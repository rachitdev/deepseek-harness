//! Durable, content-addressed storage for files a person uploads, and the text
//! the model is shown in their place.
//!
//! A stored file is two names for one inode below the store root:
//!
//! ```text
//! <root>/file-objects/ab/<sha256>          the bytes, written once
//! <root>/files/ab/<sha256>/<display name>  a hard link whose leaf is the real name
//! <root>/objects/ab/<sha256>               a normalized image, written once
//! ```
//!
//! The session log holds only a [`FileRef`] (`sha256:<hex>`, the sanitized
//! name, the byte count); the model is shown [`file_handle_text`], which names
//! the path of the second link. This is the layout of the TypeScript
//! `attachment-local` store, so the two hosts agree on every name they derive
//! from an upload.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use cordis::Context;
use sha2::{Digest, Sha256};

use crate::types::{ContentBlock, FileRef, Message};

/// The service key the store is provided under.
pub const ATTACHMENTS_SERVICE: &str = "attachments";

/// Longest stored leaf name, in UTF-8 bytes.
const MAX_NAME_BYTES: usize = 255;

/// Why an attachment operation failed. `code` is the stable machine-routing
/// reason the page shows; `message` is safe to show; `cause` is for the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentError {
    pub code: &'static str,
    pub message: String,
    pub cause: Option<String>,
}

impl AttachmentError {
    pub(crate) fn new(code: &'static str, message: &str) -> Self {
        AttachmentError {
            code,
            message: message.to_string(),
            cause: None,
        }
    }

    pub(crate) fn io(code: &'static str, message: &str, cause: impl std::fmt::Display) -> Self {
        AttachmentError {
            cause: Some(cause.to_string()),
            ..Self::new(code, message)
        }
    }
}

/// The directory tree uploads are kept in.
#[derive(Debug, Clone)]
pub struct AttachmentStore {
    root: PathBuf,
}

impl AttachmentStore {
    /// Open the store at `root`, creating it. A relative root is refused: the
    /// path the model is shown must mean the same thing to every process.
    pub fn open(root: &Path) -> Result<Self, String> {
        if !root.is_absolute() {
            return Err(format!(
                "the attachments directory must be an absolute path, got {}",
                root.display()
            ));
        }
        fs::create_dir_all(root)
            .map_err(|error| format!("cannot create {}: {error}", root.display()))?;
        Ok(AttachmentStore {
            root: root.to_path_buf(),
        })
    }

    /// Store `data` byte for byte under `name` (sanitized), or find it already
    /// stored. Blocking: call from a blocking context.
    pub fn save_file(&self, data: &[u8], name: Option<&str>) -> Result<FileRef, AttachmentError> {
        let digest = hex(&Sha256::digest(data));
        let file = FileRef {
            attachment_id: format!("sha256:{digest}"),
            name: file_leaf_name(name),
            bytes: data.len() as u64,
        };
        let object = self.object_path(&digest);
        self.publish_object(&object, data, &digest)?;
        let alias = self.file_path(&file)?;
        self.publish_alias(&object, &alias, &digest)?;
        Ok(file)
    }

    /// Store a normalized image under its digest, or find it already stored.
    /// Blocking: call from a blocking context.
    pub fn save_image(&self, prepared: &crate::images::Prepared) -> Result<(), AttachmentError> {
        let digest = digest_of(&prepared.image.attachment_id).ok_or_else(|| {
            AttachmentError::new(
                "INVALID_ATTACHMENT_REF",
                "Attachment reference is invalid.",
            )
        })?;
        self.publish_object(&self.image_path(digest), &prepared.data, digest)
    }

    /// Read a stored image, proving it is the bytes the reference names: the
    /// digest, the type and the size all have to agree. Blocking.
    pub fn read_image(&self, reference: &crate::types::ImageRef) -> Result<Vec<u8>, AttachmentError> {
        let digest = digest_of(&reference.attachment_id).ok_or_else(|| {
            AttachmentError::new(
                "INVALID_ATTACHMENT_REF",
                "Attachment reference is invalid.",
            )
        })?;
        let path = self.image_path(digest);
        // The reference says how big the object is; a larger file is not it.
        match fs::metadata(&path) {
            Ok(meta) if meta.len() != reference.bytes => {
                return Err(AttachmentError::new(
                    "ATTACHMENT_CORRUPT",
                    "Stored attachment failed integrity verification.",
                ))
            }
            _ => {}
        }
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(AttachmentError::new(
                    "ATTACHMENT_NOT_FOUND",
                    "Attachment object is missing.",
                ))
            }
            Err(error) => {
                return Err(AttachmentError::io(
                    "ATTACHMENT_READ_FAILED",
                    "Unable to read image attachment.",
                    error,
                ))
            }
        };
        if hex(&Sha256::digest(&data)) != digest {
            return Err(AttachmentError::new(
                "ATTACHMENT_CORRUPT",
                "Stored attachment failed integrity verification.",
            ));
        }
        let (media_type, width, height) = crate::images::probe(&data).map_err(|_| corrupt_metadata())?;
        if media_type != reference.media_type
            || data.len() as u64 != reference.bytes
            || (width, height) != (reference.width, reference.height)
        {
            return Err(corrupt_metadata());
        }
        Ok(data)
    }

    /// Read a stored file, proving it is the bytes the reference names: the
    /// digest and the size both have to agree. Blocking.
    pub fn read_file(&self, file: &FileRef) -> Result<Vec<u8>, AttachmentError> {
        let digest = digest_of(&file.attachment_id).ok_or_else(|| {
            AttachmentError::new(
                "INVALID_ATTACHMENT_REF",
                "File attachment reference is invalid.",
            )
        })?;
        let path = self.file_path(file)?;
        let data = match fs::read(&path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(AttachmentError::new(
                    "ATTACHMENT_NOT_FOUND",
                    "Attachment object is missing.",
                ))
            }
            Err(error) => {
                return Err(AttachmentError::io(
                    "ATTACHMENT_READ_FAILED",
                    "Unable to read file attachment.",
                    error,
                ))
            }
        };
        if data.len() as u64 != file.bytes || hex(&Sha256::digest(&data)) != digest {
            return Err(AttachmentError::new(
                "ATTACHMENT_CORRUPT",
                "Stored attachment failed integrity verification.",
            ));
        }
        Ok(data)
    }

    fn image_path(&self, digest: &str) -> PathBuf {
        self.root.join("objects").join(&digest[..2]).join(digest)
    }

    /// The path the model is told to read `file` at.
    pub fn file_path(&self, file: &FileRef) -> Result<PathBuf, AttachmentError> {
        let digest = digest_of(&file.attachment_id)
            .filter(|_| file.name == file_leaf_name(Some(&file.name)))
            .ok_or_else(|| {
                AttachmentError::new(
                    "INVALID_ATTACHMENT_REF",
                    "File attachment reference is invalid.",
                )
            })?;
        Ok(self
            .root
            .join("files")
            .join(&digest[..2])
            .join(digest)
            .join(&file.name))
    }

    fn object_path(&self, digest: &str) -> PathBuf {
        self.root
            .join("file-objects")
            .join(&digest[..2])
            .join(digest)
    }

    fn publish_object(
        &self,
        object: &Path,
        data: &[u8],
        digest: &str,
    ) -> Result<(), AttachmentError> {
        if object.exists() {
            return verify_digest(object, digest);
        }
        let write_failed = |cause: std::io::Error| {
            AttachmentError::io(
                "ATTACHMENT_WRITE_FAILED",
                "Unable to persist attachment.",
                cause,
            )
        };
        let staging = self.root.join("tmp");
        fs::create_dir_all(&staging).map_err(write_failed)?;
        let staged = staging.join(format!("{digest}-{}", crate::plugins::unique_id()));
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staged)?;
            file.write_all(data)?;
            file.sync_all()?;
            drop(file);
            make_read_only(&staged)?;
            fs::create_dir_all(object.parent().unwrap_or(&self.root))?;
            // Publish without replacing: two uploads of the same bytes race for
            // the name, and the loser must not swap the inode an alias already
            // links to.
            match fs::hard_link(&staged, object) {
                Ok(()) => Ok(false),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(true),
                Err(error) => Err(error),
            }
        })();
        // The staging name is only a handle on the bytes; the object keeps them.
        let _ = fs::remove_file(&staged);
        match result {
            Ok(false) => Ok(()),
            Ok(true) => verify_digest(object, digest),
            Err(error) => Err(write_failed(error)),
        }
    }

    fn publish_alias(
        &self,
        object: &Path,
        alias: &Path,
        digest: &str,
    ) -> Result<(), AttachmentError> {
        let write_failed = |cause: std::io::Error| {
            AttachmentError::io(
                "ATTACHMENT_WRITE_FAILED",
                "Unable to persist attachment.",
                cause,
            )
        };
        fs::create_dir_all(alias.parent().unwrap_or(&self.root)).map_err(write_failed)?;
        match fs::hard_link(object, alias) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                verify_digest(alias, digest)
            }
            Err(error) => Err(write_failed(error)),
        }
    }
}

fn corrupt_metadata() -> AttachmentError {
    AttachmentError::new(
        "ATTACHMENT_CORRUPT",
        "Stored attachment metadata does not match its reference.",
    )
}

/// The hex digest in a `sha256:<hex>` id, when it is one.
fn digest_of(attachment_id: &str) -> Option<&str> {
    attachment_id
        .strip_prefix("sha256:")
        .filter(|hex| hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
}

/// Hash the stored file as a stream, so a large or substituted entry costs no
/// memory.
fn verify_digest(path: &Path, digest: &str) -> Result<(), AttachmentError> {
    let read_failed = |error| {
        AttachmentError::io(
            "ATTACHMENT_READ_FAILED",
            "Unable to read file attachment.",
            error,
        )
    };
    let mut file = fs::File::open(path).map_err(read_failed)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher).map_err(read_failed)?;
    if hex(&hasher.finalize()) == digest {
        return Ok(());
    }
    Err(AttachmentError::new(
        "ATTACHMENT_CORRUPT",
        "Stored attachment failed integrity verification.",
    ))
}

#[cfg(unix)]
fn make_read_only(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o400))
}

#[cfg(not(unix))]
fn make_read_only(path: &Path) -> std::io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Decode an upload's base64, refusing every form that does not re-encode to
/// itself (missing padding, whitespace, stray trailing bits).
pub fn decode_canonical_base64(data: &str) -> Option<Vec<u8>> {
    let decoded = STANDARD.decode(data).ok()?;
    (STANDARD.encode(&decoded) == data).then_some(decoded)
}

/// Reduce a caller's display name to a safe leaf name: no path, no control
/// characters, nothing Windows refuses, at most 255 bytes, never empty.
pub fn file_leaf_name(name: Option<&str>) -> String {
    let Some(name) = name else {
        return "file".to_string();
    };
    // Both separator styles are stripped by hand: a POSIX host would keep a
    // Windows client's whole local path otherwise.
    let leaf = name.rsplit(['/', '\\']).next().unwrap_or_default();
    let cleaned: String = leaf
        .chars()
        .filter(|c| !matches!(c, '\u{0}'..='\u{1f}' | '\u{7f}'))
        .map(|c| {
            if matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*') {
                '_'
            } else {
                c
            }
        })
        .collect();
    let mut clean = strip_trailing(cleaned.trim_matches(is_js_space)).to_string();
    if is_windows_device_name(&clean) {
        clean.insert(0, '_');
    }
    let clean = strip_trailing(utf8_prefix(&clean, MAX_NAME_BYTES));
    match clean {
        "" | "." | ".." => "file".to_string(),
        other => other.to_string(),
    }
}

/// JavaScript's `trim` also strips the byte-order mark.
fn is_js_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

fn strip_trailing(text: &str) -> &str {
    text.trim_end_matches(['.', ' '])
}

fn utf8_prefix(text: &str, max_bytes: usize) -> &str {
    let mut end = 0;
    for (at, c) in text.char_indices() {
        if at + c.len_utf8() > max_bytes {
            break;
        }
        end = at + c.len_utf8();
    }
    &text[..end]
}

fn is_windows_device_name(name: &str) -> bool {
    let stem = strip_trailing(name.split('.').next().unwrap_or_default()).to_ascii_lowercase();
    matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        || (stem.len() == 4
            && (stem.starts_with("com") || stem.starts_with("lpt"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

/// What the model is shown for an uploaded file: its identity, and where a
/// read-only copy lives when this process can name that place.
pub fn file_handle_text(file: &FileRef, path: Option<&Path>) -> String {
    let digest = file
        .attachment_id
        .strip_prefix("sha256:")
        .unwrap_or(&file.attachment_id);
    let digest = &digest[..digest.len().min(8)];
    let identity = format!(
        "File {} ({} bytes, sha256:{digest})",
        quoted(&file.name),
        file.bytes
    );
    match path {
        None => format!(
            "[{identity} was uploaded, but the current execution environment cannot access a readable path. Report that limitation if its contents are needed; do not claim to have read it.]"
        ),
        Some(path) => format!(
            "[{identity}: verbatim read-only copy saved at {}. Read that path with your file tools when its contents are needed; copy it to a writable location before modifying it. When delegating file work, include this saved path in the delegation prompt; only subagents sharing this execution environment can read it.]",
            quoted(&path.to_string_lossy())
        ),
    }
}

fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| format!("\"{text}\""))
}

/// The messages as a model request carries them: each file block replaced by
/// its handle text and each image block by the image itself. The log keeps the
/// references; only the request is rewritten.
///
/// A model that is not listed as accepting images is told an image was
/// attached instead of being sent it. An image whose stored bytes cannot be
/// read stays a reference block: the request names it as not shown, and the
/// cause is logged, so the model never answers as if it had seen a picture it
/// was not sent.
pub fn request_messages(
    ctx: &Context,
    messages: Vec<Message>,
    provider: &str,
    model: &str,
) -> Result<Vec<Message>, AttachmentError> {
    let has_attachment = |message: &Message| {
        message.content.iter().any(|block| {
            matches!(block, ContentBlock::File { .. } | ContentBlock::Image { .. })
        })
    };
    if !messages.iter().any(has_attachment) {
        return Ok(messages);
    }
    let store = ctx.get::<AttachmentStore>(ATTACHMENTS_SERVICE);
    let sees_images = ctx
        .get::<crate::images::ImageModels>(crate::images::IMAGE_MODELS_SERVICE)
        .is_some_and(|models| models.accepts(provider, model));
    let mut messages = messages;
    for message in &mut messages {
        for block in &mut message.content {
            match block {
                ContentBlock::File { attachment } => {
                    let path = store
                        .as_ref()
                        .and_then(|store| store.file_path(attachment).ok());
                    *block = ContentBlock::text(file_handle_text(attachment, path.as_deref()));
                }
                ContentBlock::Image { attachment } if !sees_images => {
                    *block = ContentBlock::text(format!(
                        "[Image {} was attached, but the current model cannot view images.]",
                        attachment.name.as_deref().unwrap_or("(unnamed)")
                    ));
                }
                ContentBlock::Image { attachment } => {
                    // A picture the model cannot be sent stops the request: an
                    // answer given without it would read as if it had seen it.
                    let store = store.as_ref().ok_or_else(|| {
                        AttachmentError::new(
                            "ATTACHMENT_NOT_FOUND",
                            "Attachment object is missing.",
                        )
                    })?;
                    let bytes = store.read_image(attachment)?;
                    *block = ContentBlock::ImageData {
                        attachment: attachment.clone(),
                        data: STANDARD.encode(bytes),
                    };
                }
                _ => {}
            }
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(raw: &str) -> String {
        file_leaf_name(Some(raw))
    }

    #[test]
    fn a_missing_name_is_file() {
        assert_eq!(file_leaf_name(None), "file");
    }

    #[test]
    fn both_separator_styles_are_stripped_so_a_client_path_never_leaks() {
        assert_eq!(name("/etc/passwd"), "passwd");
        assert_eq!(name("C:\\Users\\me\\secret.txt"), "secret.txt");
        assert_eq!(name("a/b\\c<d>.txt"), "c_d_.txt");
    }

    #[test]
    fn names_that_mean_nothing_become_file() {
        for raw in ["", ".", "..", "  ", "...", "dir/", "\u{0}\u{1}"] {
            assert_eq!(name(raw), "file", "{raw:?}");
        }
    }

    #[test]
    fn control_characters_and_trailing_dots_and_spaces_go() {
        assert_eq!(name("a\u{0}b"), "ab");
        assert_eq!(name("trail. ."), "trail");
        assert_eq!(name("  padded.txt  "), "padded.txt");
    }

    #[test]
    fn windows_device_names_are_prefixed() {
        assert_eq!(name("CON.txt"), "_CON.txt");
        assert_eq!(name("com1"), "_com1");
        assert_eq!(name("LPT9.log"), "_LPT9.log");
        assert_eq!(name("nul. "), "_nul");
        assert_eq!(name("com0.txt"), "com0.txt");
        assert_eq!(name("console.txt"), "console.txt");
    }

    #[test]
    fn a_long_name_is_cut_on_a_character_boundary_at_255_bytes() {
        let long = format!("{}.txt", "x".repeat(300));
        assert_eq!(name(&long).len(), 255);
        let wide = "é".repeat(200);
        let cut = name(&wide);
        assert_eq!(cut.len(), 254);
        assert!(cut.chars().all(|c| c == 'é'));
    }

    #[test]
    fn only_canonical_base64_is_accepted() {
        assert_eq!(decode_canonical_base64("aGk="), Some(b"hi".to_vec()));
        assert_eq!(decode_canonical_base64(""), Some(Vec::new()));
        for bad in [
            "aGk",
            "aGl=",
            "a Gk=",
            "aGk=\n",
            "not base64!!",
            "aGk==",
            "=aGk",
        ] {
            assert_eq!(decode_canonical_base64(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn the_handle_names_a_path_when_it_has_one_and_says_so_when_it_does_not() {
        let file = FileRef {
            attachment_id: format!("sha256:{}", "ab".repeat(32)),
            name: "n.txt".to_string(),
            bytes: 12,
        };
        let with = file_handle_text(&file, Some(Path::new("/s/files/ab/x/n.txt")));
        assert!(with.starts_with("[File \"n.txt\" (12 bytes, sha256:abababab): verbatim read-only copy saved at \"/s/files/ab/x/n.txt\"."), "{with}");
        let without = file_handle_text(&file, None);
        assert!(
            without.contains("cannot access a readable path"),
            "{without}"
        );
        assert!(without.contains("do not claim to have read it"));
    }

    #[test]
    fn a_saved_file_is_stored_once_under_both_names_and_read_back_identical() {
        let dir = tempfile::tempdir().unwrap();
        let store = AttachmentStore::open(dir.path()).unwrap();
        let file = store
            .save_file(b"hello world\n", Some("dir/notes.txt"))
            .unwrap();
        assert_eq!(file.name, "notes.txt");
        assert_eq!(file.bytes, 12);
        assert_eq!(
            file.attachment_id,
            "sha256:a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447"
        );
        let path = store.file_path(&file).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"hello world\n");
        assert!(path.ends_with(
            "files/a9/a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447/notes.txt"
        ));
        let again = store
            .save_file(b"hello world\n", Some("other.txt"))
            .unwrap();
        assert_eq!(again.attachment_id, file.attachment_id);
        let objects: Vec<_> = fs::read_dir(dir.path().join("file-objects/a9"))
            .unwrap()
            .collect();
        assert_eq!(objects.len(), 1, "the bytes are kept once");
        assert_eq!(
            fs::read(store.file_path(&again).unwrap()).unwrap(),
            b"hello world\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn concurrent_saves_of_the_same_bytes_leave_every_alias_linked_to_the_one_object() {
        use std::os::unix::fs::MetadataExt;
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(AttachmentStore::open(dir.path()).unwrap());
        let saves: Vec<_> = (0..16)
            .map(|n| {
                let store = std::sync::Arc::clone(&store);
                std::thread::spawn(move || {
                    store
                        .save_file(b"same bytes", Some(&format!("copy-{n}.txt")))
                        .unwrap()
                })
            })
            .collect();
        let files: Vec<_> = saves.into_iter().map(|save| save.join().unwrap()).collect();
        let object = fs::metadata(store.object_path(&files[0].attachment_id[7..])).unwrap();
        for file in &files {
            let alias = fs::metadata(store.file_path(file).unwrap()).unwrap();
            assert_eq!(
                alias.ino(),
                object.ino(),
                "{} shares the object's inode",
                file.name
            );
        }
        assert_eq!(
            fs::read_dir(dir.path().join("tmp")).unwrap().count(),
            0,
            "no staging file is left behind"
        );
    }

    #[test]
    fn saving_the_same_name_twice_is_idempotent_and_an_empty_file_is_valid() {
        let dir = tempfile::tempdir().unwrap();
        let store = AttachmentStore::open(dir.path()).unwrap();
        let first = store.save_file(b"", None).unwrap();
        let second = store.save_file(b"", None).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.bytes, 0);
        assert_eq!(first.name, "file");
    }

    #[test]
    fn a_tampered_object_is_reported_not_trusted() {
        let dir = tempfile::tempdir().unwrap();
        let store = AttachmentStore::open(dir.path()).unwrap();
        let file = store.save_file(b"genuine", Some("a.txt")).unwrap();
        let digest = file
            .attachment_id
            .strip_prefix("sha256:")
            .unwrap()
            .to_string();
        let object = dir
            .path()
            .join("file-objects")
            .join(&digest[..2])
            .join(&digest);
        let mut permissions = fs::metadata(&object).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        fs::set_permissions(&object, permissions).unwrap();
        fs::write(&object, b"forged!").unwrap();
        let error = store.save_file(b"genuine", Some("a.txt")).unwrap_err();
        assert_eq!(error.code, "ATTACHMENT_CORRUPT");
    }

    #[test]
    fn a_reference_that_is_not_a_digest_or_not_a_leaf_name_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = AttachmentStore::open(dir.path()).unwrap();
        let ok = format!("sha256:{}", "a".repeat(64));
        for (id, name) in [
            ("sha256:short", "n"),
            ("md5:abc", "n"),
            (ok.as_str(), "../escape"),
            (ok.as_str(), "a/b"),
            (&format!("sha256:{}", "A".repeat(64)), "n"),
        ] {
            let file = FileRef {
                attachment_id: id.to_string(),
                name: name.to_string(),
                bytes: 0,
            };
            assert_eq!(
                store.file_path(&file).unwrap_err().code,
                "INVALID_ATTACHMENT_REF",
                "{id} {name}"
            );
        }
    }

    #[test]
    fn a_relative_store_root_is_refused() {
        assert!(AttachmentStore::open(Path::new("relative/dir")).is_err());
    }
}
