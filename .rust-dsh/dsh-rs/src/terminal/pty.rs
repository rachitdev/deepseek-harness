//! A child process on a pseudo-terminal.
//!
//! The child gets the pty slave as its controlling terminal and runs in its own
//! session, so signals sent to its process group reach everything it started.
//! Output is read on a thread and handed to the async side as byte chunks; the
//! child's exit is published on a watch channel once the thread that waits for
//! it has seen it.

use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::os::unix::process::{CommandExt, ExitStatusExt};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use nix::pty::{openpty, Winsize};
use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use tokio::sync::{mpsc, watch};

nix::ioctl_write_ptr_bad!(set_window_size, libc::TIOCSWINSZ, libc::winsize);

/// How a child ended: its exit code, or `None` when a signal ended it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exit {
    pub code: Option<i32>,
}

/// What the child is doing, for deciding whether an unattended terminal is idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    /// The shell itself holds the foreground: nothing is running in it.
    Idle,
    /// Something other than the shell holds the foreground.
    Busy,
    /// The process has ended, or the foreground could not be read.
    Unknown,
}

/// A running child and its pty master.
pub struct Pty {
    pid: i32,
    master: Arc<Mutex<File>>,
    exit: watch::Receiver<Option<Exit>>,
    output: Mutex<Option<mpsc::Receiver<Vec<u8>>>>,
    reader: Mutex<Option<std::thread::JoinHandle<()>>>,
}

/// The processes `pid` started, and their descendants.
///
/// The TypeScript host walks the process tree for the same reason: a shell's
/// background job has a process group of its own, so ending the shell's group
/// leaves it running. The tree is read before the shell ends, because a job
/// that is reparented when its parent ends would otherwise escape.
fn descendants(pid: i32) -> Vec<i32> {
    let table = process_table();
    let mut children: std::collections::HashMap<i32, Vec<i32>> = std::collections::HashMap::new();
    for (process, parent) in table {
        children.entry(parent).or_default().push(process);
    }
    let mut found: Vec<i32> = Vec::new();
    let mut stack = vec![pid];
    while let Some(current) = stack.pop() {
        for child in children.get(&current).into_iter().flatten() {
            if !found.contains(child) {
                found.push(*child);
                stack.push(*child);
            }
        }
    }
    found
}

/// The processes in the session `pid` leads, on the systems that report it.
///
/// Linux knows a process's session; macOS does not, and the process tree is
/// what the TypeScript host walks there.
fn session_members(pid: i32) -> Vec<i32> {
    if !cfg!(target_os = "linux") {
        return Vec::new();
    }
    let mut members = Vec::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Ok(process) = name.parse::<i32>() else { continue };
        let Ok(stat) = std::fs::read_to_string(entry.path().join("stat")) else { continue };
        // The session is the sixth field, after the command in parentheses.
        let Some(after) = stat.rsplit(") ").nth(1) else { continue };
        let mut fields = after.split_whitespace();
        let (Some(_state), Some(_ppid), Some(_pgrp), Some(session)) = (fields.next(), fields.next(), fields.next(), fields.next()) else {
            continue;
        };
        if session.parse::<i32>().ok() == Some(pid) {
            members.push(process);
        }
    }
    members
}

/// The processes of the host, as `(pid, parent)`.
fn process_table() -> Vec<(i32, i32)> {
    let Ok(output) = std::process::Command::new("ps").args(["-o", "pid=,ppid=", "-ax"]).output() else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut table = Vec::new();
    for line in text.lines() {
        let mut fields = line.split_whitespace();
        let (Some(process), Some(parent)) = (fields.next(), fields.next()) else {
            continue;
        };
        if let (Ok(process), Ok(parent)) = (process.parse::<i32>(), parent.parse::<i32>()) {
            table.push((process, parent));
        }
    }
    table
}

/// The processes a terminal owns: the jobs it started, and on Linux the
/// members of its session.
fn owned_processes(pid: i32) -> Vec<i32> {
    let mut owned = descendants(pid);
    owned.extend(session_members(pid));
    owned.retain(|member| *member != pid);
    owned.sort_unstable();
    owned.dedup();
    owned
}

fn winsize(cols: u16, rows: u16) -> libc::winsize {
    libc::winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 }
}

impl Pty {
    /// Start `argv` in `cwd` on a pty of `cols` by `rows`.
    pub fn spawn(argv: &[String], cwd: &str, cols: u16, rows: u16, env: &[(String, String)]) -> std::io::Result<Pty> {
        let program = argv.first().ok_or_else(|| std::io::Error::other("terminal argv must contain a program"))?;
        let pty = openpty(
            Some(&Winsize { ws_row: rows, ws_col: cols, ws_xpixel: 0, ws_ypixel: 0 }),
            None,
        )
        .map_err(std::io::Error::from)?;
        let (master, slave): (OwnedFd, OwnedFd) = (pty.master, pty.slave);
        let mut command = Command::new(program);
        command.args(&argv[1..]).current_dir(cwd);
        for (key, value) in env {
            command.env(key, value);
        }
        command
            .stdin(Stdio::from(slave.try_clone()?))
            .stdout(Stdio::from(slave.try_clone()?))
            .stderr(Stdio::from(slave));
        // SAFETY: only async-signal-safe calls run between fork and exec.
        unsafe {
            command.pre_exec(|| {
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                // Make the slave (now fd 0) the controlling terminal.
                if libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command.spawn()?;
        let pid = child.id() as i32;
        let master_file = File::from(master);
        let mut reader_file = master_file.try_clone()?;
        // The queue is bounded: a process that writes faster than the screen
        // is read waits, and the pty applies the backpressure.
        let (out_tx, out_rx) = mpsc::channel(256);
        let (exit_tx, exit_rx) = watch::channel(None);
        let reader = std::thread::spawn(move || {
            let mut buffer = vec![0u8; 64 * 1024];
            loop {
                match reader_file.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if out_tx.blocking_send(buffer[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    // EIO is how a pty master reports that every slave end has closed.
                    Err(_) => break,
                }
            }
        });
        std::thread::spawn(move || {
            let status = child.wait();
            let code = match status {
                Ok(status) if status.signal().is_some() => None,
                Ok(status) => status.code(),
                Err(_) => None,
            };
            let _ = exit_tx.send(Some(Exit { code }));
        });
        Ok(Pty {
            pid,
            master: Arc::new(Mutex::new(master_file)),
            exit: exit_rx,
            output: Mutex::new(Some(out_rx)),
            reader: Mutex::new(Some(reader)),
        })
    }

    /// The child's process id.
    pub fn pid(&self) -> i32 {
        self.pid
    }

    /// The processes this terminal owns: the jobs the shell started.
    pub fn owned_processes(&self) -> Vec<i32> {
        owned_processes(self.pid)
    }

    /// The output stream; available once.
    pub fn take_output(&self) -> Option<mpsc::Receiver<Vec<u8>>> {
        self.output.lock().unwrap().take()
    }

    /// A receiver of the child's exit.
    pub fn exit(&self) -> watch::Receiver<Option<Exit>> {
        self.exit.clone()
    }

    /// Whether the child has ended.
    pub fn exited(&self) -> bool {
        self.exit.borrow().is_some()
    }

    /// Send `data` to the child's terminal input.
    pub async fn write(&self, data: Vec<u8>) -> std::io::Result<()> {
        let master = Arc::clone(&self.master);
        tokio::task::spawn_blocking(move || master.lock().unwrap().write_all(&data))
            .await
            .map_err(std::io::Error::other)?
    }

    /// Change the terminal's size.
    pub fn resize(&self, cols: u16, rows: u16) -> std::io::Result<()> {
        let master = self.master.lock().unwrap();
        let size = winsize(cols, rows);
        // SAFETY: the fd is the open pty master and `size` outlives the call.
        unsafe { set_window_size(master.as_raw_fd(), &size) }.map(|_| ()).map_err(std::io::Error::from)
    }

    /// What the child is doing now.
    ///
    /// A shell that has a job of its own is busy, as the TypeScript host
    /// reports: the foreground group alone does not tell the whole story. The
    /// process table is read without holding the pty, so a scan never keeps a
    /// write from reaching the child.
    pub async fn activity(&self) -> Activity {
        if self.exited() {
            return Activity::Unknown;
        }
        let foreground = {
            let master = self.master.lock().unwrap();
            nix::unistd::tcgetpgrp(master.as_fd()).map(|group| group.as_raw())
        };
        match foreground {
            Ok(group) if group == self.pid => {
                let pid = self.pid;
                let owned = tokio::task::spawn_blocking(move || owned_processes(pid)).await.unwrap_or_default();
                if owned.is_empty() { Activity::Idle } else { Activity::Busy }
            }
            Ok(_) => Activity::Busy,
            Err(_) => Activity::Unknown,
        }
    }

    /// End the child and everything it started.
    ///
    /// An interactive shell puts its foreground and background jobs in
    /// process groups of their own, so hanging up the shell's group leaves them
    /// running. The child leads its own session (it called `setsid`), so every
    /// process it started is a member of that session: the session's members
    /// are the range to end.
    pub async fn terminate(&self, grace: Duration) {
        let group = Pid::from_raw(self.pid);
        // The range is captured before the shell ends: a job reparented when
        // its parent ends would otherwise escape.
        let members = owned_processes(self.pid);
        if !self.exited() {
            // The shell's group first, then everything in its session.
            let _ = killpg(group, Signal::SIGHUP);
        }
        for member in &members {
            if *member != self.pid {
                let _ = nix::sys::signal::kill(Pid::from_raw(*member), Signal::SIGTERM);
            }
        }
        let mut exit = self.exit();
        let shell_ended = tokio::time::timeout(grace, exit.wait_for(Option::is_some)).await.is_ok();
        // Whatever survived the grace is ended for good, whether or not the
        // shell itself ended.
        for member in &members {
            if *member != self.pid {
                let _ = nix::sys::signal::kill(Pid::from_raw(*member), Signal::SIGKILL);
            }
        }
        let _ = killpg(group, Signal::SIGKILL);
        if !shell_ended {
            let _ = exit.wait_for(Option::is_some).await;
        }
    }

    /// Wait for the reader thread to finish, so every byte the child wrote has been delivered.
    pub async fn drained(&self) {
        let reader = self.reader.lock().unwrap().take();
        if let Some(reader) = reader {
            let _ = tokio::task::spawn_blocking(move || reader.join()).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn collect(pty: &Pty) -> String {
        let mut output = pty.take_output().unwrap();
        let mut text = Vec::new();
        while let Some(chunk) = output.recv().await {
            text.extend(chunk);
        }
        String::from_utf8_lossy(&text).to_string()
    }

    fn sh(script: &str) -> Vec<String> {
        vec!["/bin/sh".into(), "-c".into(), script.into()]
    }

    #[tokio::test]
    async fn a_program_runs_on_a_terminal_and_its_output_and_exit_arrive() {
        let pty = Pty::spawn(&sh("echo hello-$((6*7)); test -t 0 && echo tty; exit 3"), "/tmp", 80, 24, &[]).unwrap();
        let text = collect(&pty).await;
        assert!(text.contains("hello-42"), "{text:?}");
        assert!(text.contains("tty"), "stdin is a terminal: {text:?}");
        assert!(text.contains("\r\n"), "the terminal turns newlines into CRLF: {text:?}");
        let mut exit = pty.exit();
        let seen = *exit.wait_for(Option::is_some).await.unwrap();
        assert_eq!(seen, Some(Exit { code: Some(3) }));
    }

    #[tokio::test]
    async fn input_reaches_the_program_and_the_size_is_what_was_asked() {
        let pty = Pty::spawn(&sh("stty size; read line; echo got:$line"), "/tmp", 100, 30, &[]).unwrap();
        let mut output = pty.take_output().unwrap();
        pty.write(b"ping\n".to_vec()).await.unwrap();
        let mut text = String::new();
        while let Some(chunk) = output.recv().await {
            text.push_str(&String::from_utf8_lossy(&chunk));
        }
        assert!(text.contains("30 100"), "{text:?}");
        assert!(text.contains("got:ping"), "{text:?}");
    }

    #[tokio::test]
    async fn a_resize_reaches_the_program() {
        let pty = Pty::spawn(&sh("sleep 0.3; stty size"), "/tmp", 80, 24, &[]).unwrap();
        pty.resize(120, 40).unwrap();
        let text = collect(&pty).await;
        assert!(text.contains("40 120"), "{text:?}");
    }

    #[tokio::test]
    async fn the_working_directory_and_environment_are_the_ones_given() {
        let env = [("DSH_PROBE".to_string(), "yes".to_string())];
        let pty = Pty::spawn(&sh("pwd; echo $DSH_PROBE"), "/tmp", 80, 24, &env).unwrap();
        let text = collect(&pty).await;
        assert!(text.contains("tmp"), "{text:?}");
        assert!(text.contains("yes"), "{text:?}");
    }

    #[tokio::test]
    async fn a_signalled_child_has_no_exit_code() {
        let pty = Pty::spawn(&sh("kill -9 $$"), "/tmp", 80, 24, &[]).unwrap();
        let mut exit = pty.exit();
        assert_eq!(*exit.wait_for(Option::is_some).await.unwrap(), Some(Exit { code: None }));
    }

    #[tokio::test]
    async fn terminate_ends_a_child_that_ignores_hangup() {
        let pty = Pty::spawn(&sh("trap '' HUP; while true; do sleep 1; done"), "/tmp", 80, 24, &[]).unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let started = std::time::Instant::now();
        pty.terminate(Duration::from_millis(300)).await;
        assert!(pty.exited());
        assert!(started.elapsed() < Duration::from_secs(5));
        pty.drained().await;
    }

    #[tokio::test]
    async fn activity_tells_an_idle_shell_from_one_running_something() {
        let pty = Pty::spawn(&["/bin/sh".to_string(), "-i".to_string()], "/tmp", 80, 24, &[]).unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(pty.activity().await, Activity::Idle);
        pty.write(b"sleep 3\n".to_vec()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(400)).await;
        assert_eq!(pty.activity().await, Activity::Busy);
        pty.terminate(Duration::from_millis(200)).await;
        assert_eq!(pty.activity().await, Activity::Unknown);
    }

    #[tokio::test]
    async fn terminating_the_shell_ends_the_jobs_it_started() {
        // The shell puts a background job in a process group of its own, so
        // ending the shell's group alone would leave it running.
        let pty = Pty::spawn(&sh("sleep 30 & echo started:$!; wait"), "/tmp", 80, 24, &[]).unwrap();
        let mut output = pty.take_output().unwrap();
        let mut text = String::new();
        let mut child = None;
        while child.is_none() {
            let Some(chunk) = output.recv().await else { break };
            text.push_str(&String::from_utf8_lossy(&chunk));
            if let Some(at) = text.find("started:") {
                let rest = &text[at + "started:".len()..];
                let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
                if !digits.is_empty() {
                    child = digits.parse::<i32>().ok();
                }
            }
        }
        let child = child.expect("the shell reported its background job");
        assert!(nix::sys::signal::kill(Pid::from_raw(child), None).is_ok(), "the job is running");
        pty.terminate(Duration::from_millis(500)).await;
        // The job is gone: signalling it fails, because its process is ended.
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if nix::sys::signal::kill(Pid::from_raw(child), None).is_err() {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "the background job outlived the shell");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        pty.drained().await;
    }

    #[tokio::test]
    async fn a_job_control_shells_background_job_dies_with_it() {
        // An interactive shell puts a background job in a process group of its
        // own, so ending the shell's group alone leaves it running.
        let pty = Pty::spawn(&["/bin/bash".into(), "-i".into()], "/tmp", 80, 24, &[]).unwrap();
        let mut output = pty.take_output().unwrap();
        pty.write(b"sleep 60 & echo bg:$!\n".to_vec()).await.unwrap();
        let mut text = String::new();
        let mut job = None;
        for _ in 0..50 {
            let Ok(Some(chunk)) = tokio::time::timeout(Duration::from_millis(200), output.recv()).await else {
                break;
            };
            text.push_str(&String::from_utf8_lossy(&chunk));
            // The echoed command carries the text too, so the last occurrence
            // is the shell's answer.
            if let Some(at) = text.rfind("bg:") {
                let digits: String = text[at + 3..].chars().take_while(char::is_ascii_digit).collect();
                if !digits.is_empty() {
                    job = digits.parse::<i32>().ok();
                    break;
                }
            }
        }
        let job = job.unwrap_or_else(|| panic!("the shell reported its background job: {text:?}"));
        assert!(
            pty.owned_processes().contains(&job),
            "the terminal owns the job it started: {:?}",
            pty.owned_processes()
        );
        assert_eq!(pty.activity().await, Activity::Busy, "a shell with a job is busy");
        pty.terminate(Duration::from_millis(500)).await;
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while nix::sys::signal::kill(Pid::from_raw(job), None).is_ok() {
            assert!(std::time::Instant::now() < deadline, "the background job outlived the shell");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        pty.drained().await;
    }

    #[tokio::test]
    async fn a_missing_program_fails_to_start() {
        assert!(Pty::spawn(&["/no/such/shell".into()], "/tmp", 80, 24, &[]).is_err());
        assert!(Pty::spawn(&[], "/tmp", 80, 24, &[]).is_err());
    }
}
