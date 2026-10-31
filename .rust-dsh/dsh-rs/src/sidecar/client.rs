//! The sidecar client: the host's half of the bridge.
//!
//! The host starts the sidecar with a connected socket, greets it with a
//! token, and asks it what it is. After that the host calls the sidecar's
//! endpoints and the sidecar answers; a call the sidecar cannot serve comes
//! back as the failure the TypeScript package raised, with its code and its
//! message.
//!
//! The sidecar may also call the host — a plugin asking for a file — and the
//! host answers over the same socket.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::{oneshot, Mutex};

use super::protocol::{frame, take_frame, Message};
use crate::gui::wire::{codes, Failure};

/// How a call ends.
type Answer = Result<Value, Failure>;

/// The sidecar's configuration.
#[derive(Clone, Debug, Default)]
pub struct SidecarConfig {
    /// The command that starts the sidecar: the program and its arguments.
    pub command: Vec<String>,
    /// The TypeScript checkout the sidecar imports its packages from.
    pub checkout: Option<String>,
    /// How long the handshake may take.
    pub handshake_timeout_ms: u64,
    /// How long a call may take.
    pub call_timeout_ms: u64,
}

impl SidecarConfig {
    /// Validate the configuration before the plugin starts.
    pub fn validate(&self) -> Result<(), String> {
        if self.command.is_empty() {
            return Err("gui: sidecar.command must name a program".to_string());
        }
        if self.handshake_timeout_ms == 0 || self.call_timeout_ms == 0 {
            return Err("gui: sidecar timeouts must be positive".to_string());
        }
        Ok(())
    }
}

/// Why the sidecar could not be started or reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidecarError {
    /// The sidecar could not be started.
    Start(String),
    /// The handshake did not complete in time, or the sidecar refused the token.
    Handshake(String),
}

impl std::fmt::Display for SidecarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SidecarError::Start(reason) => write!(f, "the sidecar could not start: {reason}"),
            SidecarError::Handshake(reason) => write!(f, "the sidecar did not greet the host: {reason}"),
        }
    }
}

/// The host's half of the bridge.
pub struct Sidecar {
    /// The socket's writing half.
    writer: Mutex<OwnedWriteHalf>,
    /// The socket's reading half.
    reader: Mutex<Option<OwnedReadHalf>>,
    /// The calls that are waiting for their answer.
    pending: Mutex<HashMap<String, oneshot::Sender<Answer>>>,
    /// The next frame's id.
    next_id: AtomicU64,
    /// The sidecar's answer to the handshake.
    handshake: Mutex<Value>,
    /// The reader task, so the host can stop it.
    reader_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    /// The sidecar's process.
    child: Mutex<Option<tokio::process::Child>>,
}

impl Sidecar {
    /// Start the sidecar and complete the handshake.
    pub async fn start(config: &SidecarConfig) -> Result<Arc<Self>, SidecarError> {
        let (host, sidecar) = tokio::net::UnixStream::pair()
            .map_err(|error| SidecarError::Start(format!("the socket pair could not be made: {error}")))?;
        let token = token();
        let mut command = tokio::process::Command::new(&config.command[0]);
        command.args(&config.command[1..]);
        command.env("DSH_SIDECAR_TOKEN", &token);
        if let Some(checkout) = &config.checkout {
            command.env("DSH_SIDECAR_CHECKOUT", checkout);
        }
        // The sidecar reads the socket on file descriptor 0: its standard input.
        let sidecar = sidecar
            .into_std()
            .map_err(|error| SidecarError::Start(format!("the socket could not be handed over: {error}")))?;
        let sidecar = std::os::unix::io::OwnedFd::from(sidecar);
        command.stdin(std::process::Stdio::from(sidecar));
        let mut child = command
            .spawn()
            .map_err(|error| SidecarError::Start(format!("the command could not be started: {error}")))?;
        let (read, write) = host.into_split();
        let sidecar = Arc::new(Sidecar {
            writer: Mutex::new(write),
            reader: Mutex::new(Some(read)),
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(0),
            handshake: Mutex::new(Value::Null),
            reader_task: Mutex::new(None),
            child: Mutex::new(Some(child)),
        });
        let reader = tokio::spawn(sidecar.clone().read_loop());
        *sidecar.reader_task.lock().await = Some(reader);

        let deadline = Duration::from_millis(config.handshake_timeout_ms);
        sidecar
            .exchange(Message::Hello { token }, deadline)
            .await
            .map_err(SidecarError::Handshake)?;
        sidecar
            .exchange(Message::Handshake, deadline)
            .await
            .map_err(SidecarError::Handshake)?;
        Ok(sidecar)
    }

    /// The sidecar's answer to the handshake: its protocol version, the
    /// inventory's digest and the versions of the packages it loaded.
    pub async fn handshake(&self) -> Value {
        self.handshake.lock().await.clone()
    }

    /// The sidecar's protocol version.
    pub async fn protocol(&self) -> u64 {
        self.handshake.lock().await["protocol"].as_u64().unwrap_or_default()
    }

    /// Call one of the sidecar's endpoints.
    ///
    /// The failure is the sidecar's own: its code and its message reach the
    /// client unchanged.
    pub async fn call(&self, method: &str, args: Value, timeout: Duration) -> Answer {
        let id = self.next_id();
        let (sender, receiver) = oneshot::channel();
        self.pending.lock().await.insert(id.clone(), sender);
        let message = Message::Call { id: id.clone(), method: method.to_string(), args };
        if let Err(reason) = self.write(&message).await {
            self.pending.lock().await.remove(&id);
            return Err(Failure {
                code: codes::INTERNAL.to_string(),
                message: format!("the sidecar could not be reached: {reason}"),
                details: json!({}),
            });
        }
        match tokio::time::timeout(timeout, receiver).await {
            Ok(Ok(answer)) => answer,
            Ok(Err(_)) => Err(Failure {
                code: codes::INTERNAL.to_string(),
                message: "the sidecar ended before it answered".to_string(),
                details: json!({}),
            }),
            Err(_) => {
                // The call is cancelled: the sidecar is told to stop it.
                let _ = self.write(&Message::Cancel { id: id.clone() }).await;
                self.pending.lock().await.remove(&id);
                Err(Failure {
                    code: codes::INTERNAL.to_string(),
                    message: "the sidecar did not answer in time".to_string(),
                    details: json!({}),
                })
            }
        }
    }

    /// Send a message and wait for the answer the sidecar sends for it.
    async fn exchange(&self, message: Message, deadline: Duration) -> Result<Value, String> {
        let id = self.next_id();
        let (sender, receiver) = oneshot::channel();
        self.pending.lock().await.insert(id.clone(), sender);
        let message = match message {
            Message::Hello { token } => Message::Hello { token },
            other => other,
        };
        if let Err(reason) = self.write(&message).await {
            self.pending.lock().await.remove(&id);
            return Err(reason);
        }
        match tokio::time::timeout(deadline, receiver).await {
            Ok(Ok(Ok(value))) => Ok(value),
            Ok(Ok(Err(failure))) => Err(failure.message),
            Ok(Err(_)) => Err("the sidecar ended".to_string()),
            Err(_) => Err("the sidecar did not answer the handshake in time".to_string()),
        }
    }

    fn next_id(&self) -> String {
        format!("c{}", self.next_id.fetch_add(1, Ordering::SeqCst))
    }

    async fn write(&self, message: &Message) -> Result<(), String> {
        let payload = message.encode();
        let mut writer = self.writer.lock().await;
        writer.write_all(&frame(&payload)).await.map_err(|error| error.to_string())
    }

    /// Read frames until the socket closes.
    async fn read_loop(self: Arc<Self>) {
        let mut reader = match self.reader.lock().await.take() {
            Some(reader) => reader,
            None => return,
        };
        let mut buffer: Vec<u8> = Vec::new();
        loop {
            let mut chunk = [0u8; 16 * 1024];
            let read = match reader.read(&mut chunk).await {
                Ok(0) | Err(_) => break,
                Ok(read) => read,
            };
            buffer.extend_from_slice(&chunk[..read]);
            loop {
                let (payload, rest) = match take_frame(&buffer) {
                    Ok(Some(taken)) => taken,
                    Ok(None) => break,
                    Err(reason) => {
                        self.fail_all(reason).await;
                        return;
                    }
                };
                buffer = rest.to_vec();
                match Message::decode(&payload) {
                    Ok(message) => self.route(message).await,
                    Err(reason) => {
                        self.fail_all(reason).await;
                        return;
                    }
                }
            }
        }
        self.fail_all("the sidecar closed the connection".to_string()).await;
    }

    /// Hand a frame to whoever is waiting for it.
    async fn route(&self, message: Message) {
        match message {
            Message::Result { id, value } => self.complete(id, Ok(value)).await,
            Message::Failure { id, error } => {
                let failure = Failure {
                    code: error["code"].as_str().unwrap_or(codes::INTERNAL).to_string(),
                    message: error["message"].as_str().unwrap_or("the sidecar failed the call").to_string(),
                    details: error["details"].clone(),
                };
                self.complete(id, Err(failure)).await;
            }
            Message::HandshakeResult { id, protocol, inventory, packages } => {
                let value = json!({ "protocol": protocol, "inventory": inventory, "packages": packages });
                *self.handshake.lock().await = value.clone();
                self.complete(id, Ok(value)).await;
            }
            Message::Accepted { id } => self.complete(id, Ok(Value::Null)).await,
            Message::Cancelled { id } => self
                .complete(
                    id,
                    Err(Failure {
                        code: codes::INTERNAL.to_string(),
                        message: "the sidecar cancelled the call".to_string(),
                        details: json!({}),
                    }),
                )
                .await,
            Message::Unknown { kind } => {
                self.fail_all(format!("the sidecar sent an unknown frame: {kind}")).await;
            }
            Message::Hello { .. } | Message::Handshake | Message::Cancel { .. } | Message::Call { .. } => {}
        }
    }

    async fn complete(&self, id: String, answer: Answer) {
        if let Some(sender) = self.pending.lock().await.remove(&id) {
            let _ = sender.send(answer);
        }
    }

    /// Fail every waiting call.
    async fn fail_all(&self, reason: String) {
        let waiting: Vec<oneshot::Sender<Answer>> = self.pending.lock().await.drain().map(|(_, sender)| sender).collect();
        for sender in waiting {
            let _ = sender.send(Err(Failure {
                code: codes::INTERNAL.to_string(),
                message: reason.clone(),
                details: json!({}),
            }));
        }
    }

    /// Stop the sidecar.
    pub async fn stop(&self) {
        if let Some(reader) = self.reader_task.lock().await.take() {
            reader.abort();
        }
        if let Some(mut child) = self.child.lock().await.take() {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
    }
}

/// The handshake's token: 32 bytes of the host's randomness.
fn token() -> String {
    use base64::Engine as _;
    let mut bytes = [0u8; 32];
    let _ = read_random(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// The host's randomness. The operating system is the source; the crate's
/// dependencies carry no random number generator of their own.
fn read_random(bytes: &mut [u8]) -> Result<(), String> {
    let mut file = std::fs::File::open("/dev/urandom").map_err(|error| error.to_string())?;
    std::io::Read::read_exact(&mut file, bytes).map_err(|error| error.to_string())
}
