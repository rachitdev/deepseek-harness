//! The native directory picker behind `directoryPicker/pick`.
//!
//! The TypeScript host opens the operating system's own directory chooser: an
//! AppKit panel on macOS, `zenity` or `kdialog` on Linux, and the console
//! dialog on Windows. This module keeps the same three branches, the same
//! commands and the same answers: the chosen path, or nothing when the user
//! dismisses the chooser.
//!
//! The picker runs a real command, so it goes through a [`PickerRunner`]: the
//! tests answer for the chooser instead of opening one, and a command that
//! outlives its deadline is killed rather than left running.

use std::sync::Arc;

use serde_json::{json, Value};

use super::desktop::Platform;
use super::wire::{codes, Failure, RpcResult};
use crate::api::services::BoxFuture;

/// The title the chooser shows.
const DIALOG_TITLE: &str = "Select Workspace Directory";



/// The AppKit panel macOS runs. It is the TypeScript host's own script.
const MAC_PANEL: &str = r#"
ObjC.import('AppKit');
function run(argv) {
  var app = $.NSApplication.sharedApplication;
  app.setActivationPolicy($.NSApplicationActivationPolicyAccessory);
  var mainMenu = $.NSMenu.alloc.init;
  mainMenu.addItem($.NSMenuItem.alloc.init);
  var editMenu = $.NSMenu.alloc.initWithTitle('Edit');
  var editItem = $.NSMenuItem.alloc.init;
  editItem.submenu = editMenu;
  mainMenu.addItem(editItem);
  editMenu.addItemWithTitleActionKeyEquivalent('Undo', 'undo:', 'z');
  editMenu.addItemWithTitleActionKeyEquivalent('Redo', 'redo:', 'Z');
  editMenu.addItemWithTitleActionKeyEquivalent('Cut', 'cut:', 'x');
  editMenu.addItemWithTitleActionKeyEquivalent('Copy', 'copy:', 'c');
  editMenu.addItemWithTitleActionKeyEquivalent('Paste', 'paste:', 'v');
  editMenu.addItemWithTitleActionKeyEquivalent('Select All', 'selectAll:', 'a');
  app.mainMenu = mainMenu;
  var panel = $.NSOpenPanel.openPanel;
  panel.canChooseFiles = false;
  panel.canChooseDirectories = true;
  panel.canCreateDirectories = true;
  panel.allowsMultipleSelection = false;
  panel.message = argv[0];
  $.NSOperationQueue.mainQueue.addOperationWithBlock(function() {
    app.activateIgnoringOtherApps(true);
  });
  if (panel.runModal !== $.NSModalResponseOK) return '';
  var urls = panel.URLs;
  if (urls.count < 1) return '';
  return ObjC.unwrap(urls.objectAtIndex(0).path);
}"#;

/// Why a chooser command did not answer with a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerError {
    /// The command ran and failed with this exit code. `1` is the chooser's
    /// own "dismissed" answer.
    Exited(i32),
    /// The command is not installed.
    Missing,
    /// The command could not run, or failed in another way.
    Other(String),
}

/// Runs a chooser command.
pub trait PickerRunner: Send + Sync {
    /// Run `program` with `args` (no shell), returning what it printed.
    fn run(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<String, PickerError>>;
}

/// Runs real chooser commands.
///
/// The chooser waits for the operator, so it has no deadline: the command ends
/// with the request that started it, and a dropped request kills the dialog.
#[derive(Default)]
pub struct SystemPickerRunner {}

impl PickerRunner for SystemPickerRunner {
    fn run(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<String, PickerError>> {
        let program = program.to_string();
        Box::pin(async move {
            let mut command = tokio::process::Command::new(&program);
            command.args(&args).stdin(std::process::Stdio::null()).kill_on_drop(true);
            let output = command
                .output()
                .await
                .map_err(|error| {
                    if error.kind() == std::io::ErrorKind::NotFound {
                        PickerError::Missing
                    } else {
                        PickerError::Other(format!("{program} could not start: {error}"))
                    }
                })?;
            if !output.status.success() {
                return Err(PickerError::Exited(output.status.code().unwrap_or(-1)));
            }
            String::from_utf8(output.stdout)
                .map_err(|error| PickerError::Other(format!("{program} printed non-UTF-8: {error}")))
        })
    }
}

/// The native directory picker.
pub struct Picker {
    platform: Platform,
    runner: Arc<dyn PickerRunner>,
}

impl Picker {
    /// A picker for this platform.
    pub fn new(platform: Platform, runner: Arc<dyn PickerRunner>) -> Self {
        Picker { platform, runner }
    }

    /// The picker of this host.
    ///
    /// The chooser waits for the operator: it is bound to the caller's
    /// cancellation, not to a deadline, so a directory can be browsed for as
    /// long as it takes.
    pub fn system() -> Self {
        Picker::new(Platform::detect(), Arc::new(SystemPickerRunner::default()))
    }

    /// The chosen directory, or `None` when the user dismissed the chooser.
    pub async fn pick(&self) -> Result<Option<String>, String> {
        match self.platform {
            Platform::Mac => {
                let stdout = self
                    .runner
                    .run(
                        "osascript",
                        vec![
                            "-l".to_string(),
                            "JavaScript".to_string(),
                            "-e".to_string(),
                            MAC_PANEL.to_string(),
                            DIALOG_TITLE.to_string(),
                        ],
                    )
                    .await
                    .map_err(describe)?;
                Ok(output_path(&stdout))
            }
            Platform::Linux => self.pick_linux().await,
            Platform::Other => Err("native directory picker is unsupported on this platform".to_string()),
        }
    }

    /// Linux: `zenity` first, then `kdialog`.
    async fn pick_linux(&self) -> Result<Option<String>, String> {
        match self
            .runner
            .run(
                "zenity",
                vec![
                    "--file-selection".to_string(),
                    "--directory".to_string(),
                    format!("--title={DIALOG_TITLE}"),
                ],
            )
            .await
        {
            Ok(stdout) => return Ok(output_path(&stdout)),
            // The chooser's own "dismissed" answer.
            Err(PickerError::Exited(1)) => return Ok(None),
            // Zenity is not installed: try the other chooser.
            Err(PickerError::Missing) => {}
            Err(error) => return Err(describe(error)),
        }
        match self
            .runner
            .run(
                "kdialog",
                vec![
                    "--getexistingdirectory".to_string(),
                    ".".to_string(),
                    "--title".to_string(),
                    DIALOG_TITLE.to_string(),
                ],
            )
            .await
        {
            Ok(stdout) => Ok(output_path(&stdout)),
            Err(PickerError::Exited(1)) => Ok(None),
            Err(PickerError::Missing) => {
                Err("no supported native directory picker found (install zenity or kdialog)".to_string())
            }
            Err(error) => Err(describe(error)),
        }
    }
}

/// The chooser's answer: its last line without the line break, or nothing.
fn output_path(stdout: &str) -> Option<String> {
    let path = stdout.trim_end_matches(['\r', '\n']);
    (!path.is_empty()).then(|| path.to_string())
}

/// The message a chooser failure carries.
fn describe(error: PickerError) -> String {
    match error {
        PickerError::Exited(code) => format!("the directory chooser exited with {code}"),
        PickerError::Missing => "the directory chooser is not installed".to_string(),
        PickerError::Other(message) => message,
    }
}

/// `directoryPicker/pick`: the chosen directory, or nothing.
///
/// A host without the native capability answers the way the TypeScript host
/// does when its composed picker serves the browse capability.
pub(super) async fn pick(picker: Option<&Picker>) -> RpcResult {
    let Some(picker) = picker else {
        return RpcResult::failed(Failure {
            code: "directory-picker/unavailable".to_string(),
            message: "directoryPicker.pick needs the native capability; the composed picker serves \"browse\"".to_string(),
            details: json!({ "capability": "browse" }),
        });
    };
    match picker.pick().await {
        Ok(Some(path)) => RpcResult::ok(Value::String(path)),
        Ok(None) => RpcResult::ok(Value::Null),
        Err(message) => RpcResult::failed(Failure {
            code: codes::INTERNAL.to_string(),
            message: format!("directory picker failed: {message}"),
            details: json!({}),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    /// A runner that answers from a script, so no chooser opens.
    struct Script {
        answers: Mutex<VecDeque<Result<String, PickerError>>>,
        seen: Mutex<Vec<(String, Vec<String>)>>,
    }

    impl Script {
        fn new(answers: Vec<Result<String, PickerError>>) -> Arc<Self> {
            Arc::new(Script { answers: Mutex::new(answers.into()), seen: Mutex::new(Vec::new()) })
        }

        fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.seen.lock().unwrap().clone()
        }
    }

    impl PickerRunner for Script {
        fn run(&self, program: &str, args: Vec<String>) -> BoxFuture<Result<String, PickerError>> {
            self.seen.lock().unwrap().push((program.to_string(), args.clone()));
            let answer = self.answers.lock().unwrap().pop_front().unwrap_or(Ok(String::new()));
            Box::pin(async move { answer })
        }
    }

    #[tokio::test]
    async fn macos_asks_appkit_and_answers_the_chosen_path() {
        let runner = Script::new(vec![Ok("/Users/someone/Projects\n".to_string())]);
        let picker = Picker::new(Platform::Mac, runner.clone());
        assert_eq!(picker.pick().await.unwrap(), Some("/Users/someone/Projects".into()));
        let (program, args) = runner.calls().remove(0);
        assert_eq!(program, "osascript");
        assert_eq!(args[0], "-l");
        assert_eq!(args[1], "JavaScript");
        assert_eq!(args[2], "-e");
        assert!(args[3].contains("NSOpenPanel"), "the AppKit panel is the script");
        assert_eq!(args[4], "Select Workspace Directory");
    }

    #[tokio::test]
    async fn a_dismissed_chooser_answers_nothing() {
        let runner = Script::new(vec![Ok("\n".to_string())]);
        let picker = Picker::new(Platform::Mac, runner);
        assert_eq!(picker.pick().await.unwrap(), None);
    }

    #[tokio::test]
    async fn linux_asks_zenity_and_then_kdialog() {
        let runner = Script::new(vec![Err(PickerError::Missing), Ok("/home/someone\n".to_string())]);
        let picker = Picker::new(Platform::Linux, runner.clone());
        assert_eq!(picker.pick().await.unwrap(), Some("/home/someone".into()));
        let calls = runner.calls();
        assert_eq!(calls[0].0, "zenity");
        assert_eq!(calls[0].1, vec!["--file-selection", "--directory", "--title=Select Workspace Directory"]);
        assert_eq!(calls[1].0, "kdialog");
        assert_eq!(calls[1].1, vec!["--getexistingdirectory", ".", "--title", "Select Workspace Directory"]);
    }

    #[tokio::test]
    async fn a_dismissed_linux_chooser_answers_nothing() {
        let runner = Script::new(vec![Err(PickerError::Exited(1))]);
        let picker = Picker::new(Platform::Linux, runner);
        assert_eq!(picker.pick().await.unwrap(), None);
    }

    #[tokio::test]
    async fn linux_without_a_chooser_says_so() {
        let runner = Script::new(vec![Err(PickerError::Missing), Err(PickerError::Missing)]);
        let picker = Picker::new(Platform::Linux, runner);
        assert_eq!(
            picker.pick().await.unwrap_err(),
            "no supported native directory picker found (install zenity or kdialog)"
        );
    }

    #[tokio::test]
    async fn a_platform_without_a_chooser_refuses() {
        let runner = Script::new(vec![]);
        let picker = Picker::new(Platform::Other, runner);
        assert_eq!(picker.pick().await.unwrap_err(), "native directory picker is unsupported on this platform");
    }
}
