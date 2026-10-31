//! Skills plugin (`skills`).
//!
//! Discovers `<dir>/<skill>/SKILL.md` for each configured skills directory
//! (default `~/.dsh/skills`), exposes a compact catalog in the system prompt,
//! and provides a `skill` tool that loads one skill's full instructions by
//! exact name. The tool only serves names found at discovery time, so the
//! argument can never address another path.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use cordis::Plugin;
use serde::Deserialize;
use serde_json::{json, Value};

use super::{register_tools, tool_spec};
use crate::api::services::SystemPromptService;
use crate::types::ToolExecutionResult;

/// Largest SKILL.md body returned in one tool call.
const MAX_BODY_CHARS: usize = 60_000;

/// Largest `SKILL.md` read from disk; a larger file is skipped.
const MAX_MANIFEST_BYTES: u64 = 512 * 1024;

/// Read a skill manifest: it must be a regular file (not a symlink to
/// somewhere else) no larger than [`MAX_MANIFEST_BYTES`]. Skill directories
/// may themselves be symlinks, which is how installed skill sets are linked
/// into place; the manifest inside them may not be.
fn read_manifest(manifest: &Path) -> Result<String, String> {
    use std::io::Read;
    let shown = manifest.display();
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // O_NOFOLLOW refuses a symlink as the final component; O_NONBLOCK keeps
        // opening a FIFO from blocking (it is rejected below as not a regular file).
        const O_NONBLOCK: i32 = 0x0004; // macOS/BSD; Linux is 0o4000 (handled next)
        const O_NOFOLLOW: i32 = 0x0100; // macOS/BSD
        #[cfg(target_os = "linux")]
        options.custom_flags(0o4000 | 0o400000);
        #[cfg(not(target_os = "linux"))]
        options.custom_flags(O_NONBLOCK | O_NOFOLLOW);
    }
    let file = options
        .open(manifest)
        .map_err(|e| format!("{shown}: {e}"))?;
    // Checked on the opened handle, so a swap after the open cannot change what is read.
    let meta = file.metadata().map_err(|e| format!("{shown}: {e}"))?;
    if !meta.is_file() {
        return Err(format!("{shown}: not a regular file"));
    }
    let mut text = String::new();
    file.take(MAX_MANIFEST_BYTES + 1)
        .read_to_string(&mut text)
        .map_err(|e| format!("{shown}: {e}"))?;
    if text.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(format!("{shown}: larger than {MAX_MANIFEST_BYTES} bytes"));
    }
    Ok(text)
}

/// Configuration for the `skills` plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SkillsConfig {
    /// Directories scanned for skills; later entries do not override earlier ones.
    pub dirs: Vec<String>,
    /// Add the catalog to the system prompt.
    pub catalog: bool,
    /// Characters of each description kept in the catalog.
    pub catalog_desc_chars: usize,
    /// Order of the system-prompt section.
    pub section_order: i32,
}

impl Default for SkillsConfig {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_default();
        let root = std::env::var("DSH_HOME").unwrap_or_else(|_| format!("{home}/.dsh"));
        SkillsConfig {
            dirs: vec![format!("{root}/skills")],
            catalog: true,
            catalog_desc_chars: 160,
            section_order: 50,
        }
    }
}

/// One discovered skill.
#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    /// Directory holding `SKILL.md` and the skill's resources.
    pub dir: PathBuf,
}

/// The skills the `skills` plugin found, shared so other surfaces (the GUI's
/// `skills/list`) report what the model can load instead of re-scanning.
#[derive(Debug, Clone, Default)]
pub struct SkillCatalog {
    pub skills: Vec<Skill>,
}

/// Split `---` frontmatter from the body. Returns `(frontmatter, body)`.
pub fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text
        .strip_prefix("---")?
        .strip_prefix('\n')
        .or_else(|| text.strip_prefix("---\r\n"))?;
    let end = rest.find("\n---")?;
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']);
    Some((&rest[..end], body))
}

/// Read one scalar key from simple YAML frontmatter: plain, quoted, or a
/// `>` / `|` block scalar (folded to a single line).
pub fn frontmatter_value(frontmatter: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = frontmatter.lines().collect();
    let prefix = format!("{key}:");
    let index = lines.iter().position(|l| l.starts_with(&prefix))?;
    let first = lines[index][prefix.len()..].trim();
    if matches!(first.chars().next(), Some('>' | '|')) {
        let block: Vec<&str> = lines[index + 1..]
            .iter()
            .take_while(|l| l.is_empty() || l.starts_with(' ') || l.starts_with('\t'))
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        return Some(block.join(" "));
    }
    let unquoted = if first.len() >= 2
        && ((first.starts_with('"') && first.ends_with('"'))
            || (first.starts_with('\'') && first.ends_with('\'')))
    {
        &first[1..first.len() - 1]
    } else {
        first
    };
    Some(unquoted.replace("\\\"", "\"").replace("''", "'"))
}

/// Discover skills under `dir`, one level deep. Unreadable or malformed
/// entries are skipped and reported in the second element.
pub fn discover(dir: &Path) -> (Vec<Skill>, Vec<String>) {
    let mut skills = Vec::new();
    let mut skipped = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return (
            skills,
            vec![format!("{}: cannot read directory", dir.display())],
        );
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        let manifest = path.join("SKILL.md");
        if std::fs::symlink_metadata(&manifest).is_err() {
            continue; // no SKILL.md here
        }
        let text = match read_manifest(&manifest) {
            Ok(text) => text,
            Err(reason) => {
                skipped.push(reason);
                continue;
            }
        };
        let Some((frontmatter, _)) = split_frontmatter(&text) else {
            skipped.push(format!("{}: no frontmatter", manifest.display()));
            continue;
        };
        let fallback = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let name = frontmatter_value(frontmatter, "name")
            .filter(|n| !n.is_empty())
            .unwrap_or(fallback);
        let description = frontmatter_value(frontmatter, "description").unwrap_or_default();
        skills.push(Skill {
            name,
            description,
            dir: path,
        });
    }
    (skills, skipped)
}

/// Compact catalog text for the system prompt.
pub fn catalog_text(skills: &[Skill], desc_chars: usize) -> String {
    let mut out = String::from(
        "Skills are installed. When a task matches one, call the `skill` tool with its exact name to load the full instructions before acting.\nAvailable skills:\n",
    );
    for skill in skills {
        let desc = super::snippet(&skill.description, desc_chars);
        out.push_str(&format!("- {}: {}\n", skill.name, desc));
    }
    out
}

fn expand(path: &str) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    match path.strip_prefix("~/") {
        Some(rest) => PathBuf::from(home).join(rest),
        None => PathBuf::from(path),
    }
}

/// Load the skill body, bounded by [`MAX_BODY_CHARS`].
fn load_body(skill: &Skill) -> Result<String, String> {
    let manifest = skill.dir.join("SKILL.md");
    let text = read_manifest(&manifest)?;
    // A manifest that lost its frontmatter since discovery is not the skill we listed.
    let Some((_, body)) = split_frontmatter(&text) else {
        return Err(format!(
            "{}: frontmatter missing; reinstall the skill",
            manifest.display()
        ));
    };
    let mut out = format!(
        "Skill: {}\nBase directory: {}\n\n",
        skill.name,
        skill.dir.display()
    );
    if body.chars().count() > MAX_BODY_CHARS {
        out.extend(body.chars().take(MAX_BODY_CHARS));
        out.push_str(&format!(
            "\n\n[truncated at {MAX_BODY_CHARS} characters; read the rest from the base directory]"
        ));
    } else {
        out.push_str(body);
    }
    Ok(out)
}

/// The `skills` plugin. Requires `tools` and `systemPrompt`.
pub fn skills_plugin() -> Arc<dyn Plugin> {
    cordis::plugin_with(
        "skills",
        vec![
            cordis::Injection::new(crate::api::TOOLS_SERVICE.to_string()),
            cordis::Injection::new(crate::api::SYSTEM_PROMPT_SERVICE.to_string()),
        ],
        |ctx, config: SkillsConfig| async move {
            let mut by_name: BTreeMap<String, Skill> = BTreeMap::new();
            for dir in &config.dirs {
                let root = expand(dir);
                let (found, skipped) =
                    tokio::task::spawn_blocking(move || discover(&root))
                        .await
                        .map_err(|e| cordis::Error::msg(format!("skills discovery failed: {e}")))?;
                for note in skipped {
                    eprintln!("skills: skipped {note}");
                }
                for skill in found {
                    by_name.entry(skill.name.clone()).or_insert(skill);
                }
            }
            let skills: Vec<Skill> = by_name.into_values().collect();
            ctx.provide(
                crate::api::SKILLS_SERVICE,
                SkillCatalog {
                    skills: skills.clone(),
                },
            )
            .await?;
            let known = Arc::new(
                skills
                    .iter()
                    .cloned()
                    .map(|s| (s.name.clone(), s))
                    .collect::<BTreeMap<_, _>>(),
            );

            if config.catalog && !skills.is_empty() {
                let prompt =
                    ctx.require::<SystemPromptService>(crate::api::SYSTEM_PROMPT_SERVICE)?;
                let text = catalog_text(&skills, config.catalog_desc_chars);
                prompt.section("skills-catalog", config.section_order, &text, false);
                // The catalog tells the model to call `skill`; take it back when the
                // plugin unloads so a stale instruction cannot outlive its tool.
                ctx.effect("skills catalog", async move {
                    let disposer: cordis::Disposer = Box::new(move || {
                        Box::pin(async move {
                            prompt.remove_section("skills-catalog");
                        })
                    });
                    Ok(Some(disposer))
                })
                .await?;
            }

            let spec = tool_spec(
                "skill",
                "Load the full instructions of an installed skill by its exact name. Use when a task matches a skill in the catalog.",
                json!({
                    "type": "object",
                    "properties": { "name": { "type": "string", "description": "Exact skill name from the catalog." } },
                    "required": ["name"]
                }),
                move |args| {
                    let known = Arc::clone(&known);
                    async move {
                        let Some(name) = args.get("name").and_then(Value::as_str) else {
                            return ToolExecutionResult::error("BAD_ARGS", "name is required");
                        };
                        let Some(skill) = known.get(name) else {
                            return ToolExecutionResult::error("UNKNOWN_SKILL", format!("no installed skill named \"{name}\""));
                        };
                        let skill = skill.clone();
                        let skill_name = skill.name.clone();
                        // Blocking file I/O runs off the async worker threads.
                        let loaded = tokio::task::spawn_blocking(move || load_body(&skill))
                            .await
                            .unwrap_or_else(|e| Err(format!("skill load task failed: {e}")));
                        match loaded {
                            Ok(text) => ToolExecutionResult::success_text(text, json!({ "name": skill_name })),
                            Err(e) => ToolExecutionResult::error("SKILL_READ", e),
                        }
                    }
                },
            );
            register_tools(&ctx, "skills tools", vec![spec]).await?;
            eprintln!("skills: {} installed", skills.len());
            Ok(())
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_skill(root: &Path, dir: &str, content: &str) {
        let path = root.join(dir);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("SKILL.md"), content).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_and_special_manifests_are_refused() {
        let root =
            std::env::temp_dir().join(format!("dsh-rs-skill-sym-{}", crate::plugins::unique_id()));
        std::fs::create_dir_all(root.join("evil")).unwrap();
        let secret = root.join("secret.txt");
        std::fs::write(
            &secret,
            "---\nname: evil\ndescription: leaked\n---\nTOP SECRET",
        )
        .unwrap();
        std::os::unix::fs::symlink(&secret, root.join("evil/SKILL.md")).unwrap();
        write_skill(
            &root,
            "good",
            "---\nname: good\ndescription: fine\n---\nbody",
        );
        let (skills, skipped) = discover(&root);
        assert_eq!(
            skills.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["good"]
        );
        assert!(skipped.iter().any(|s| s.contains("evil")), "{skipped:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn oversized_manifests_are_skipped() {
        let root =
            std::env::temp_dir().join(format!("dsh-rs-skill-big-{}", crate::plugins::unique_id()));
        let big = format!(
            "---\nname: big\ndescription: d\n---\n{}",
            "x".repeat(MAX_MANIFEST_BYTES as usize)
        );
        write_skill(&root, "big", &big);
        let (skills, skipped) = discover(&root);
        assert!(
            skills.is_empty() && skipped.iter().any(|s| s.contains("larger than")),
            "{skipped:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn frontmatter_handles_plain_quoted_and_block_values() {
        let plain = "name: a\ndescription: plain text here\n";
        assert_eq!(
            frontmatter_value(plain, "description").as_deref(),
            Some("plain text here")
        );
        let quoted = "name: a\ndescription: \"say \\\"hi\\\"\"\n";
        assert_eq!(
            frontmatter_value(quoted, "description").as_deref(),
            Some("say \"hi\"")
        );
        let block = "name: a\ndescription: >\n  first line\n  second line\nlicense: MIT\n";
        assert_eq!(
            frontmatter_value(block, "description").as_deref(),
            Some("first line second line")
        );
        assert_eq!(frontmatter_value(block, "license").as_deref(), Some("MIT"));
    }

    #[test]
    fn discover_reads_skills_and_reports_malformed_ones() {
        let root = std::env::temp_dir().join(format!("dsh-skills-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        write_skill(
            &root,
            "alpha",
            "---\nname: alpha\ndescription: does alpha\n---\n# Alpha\nbody\n",
        );
        write_skill(&root, "beta", "no frontmatter here");
        write_skill(&root, "gamma-dir", "---\ndescription: unnamed\n---\nbody");
        std::fs::create_dir_all(root.join("empty")).unwrap();

        let (skills, skipped) = discover(&root);
        let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            vec!["alpha", "gamma-dir"],
            "unnamed falls back to its directory name"
        );
        assert_eq!(skipped.len(), 1);
        assert!(load_body(&skills[0]).unwrap().contains("# Alpha"));
        assert!(!load_body(&skills[0]).unwrap().contains("description:"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn catalog_truncates_descriptions() {
        let skills = vec![Skill {
            name: "s".into(),
            description: "x".repeat(50),
            dir: PathBuf::new(),
        }];
        let text = catalog_text(&skills, 10);
        assert!(text.contains("- s: xxxxxxxxxx…"));
    }
}
