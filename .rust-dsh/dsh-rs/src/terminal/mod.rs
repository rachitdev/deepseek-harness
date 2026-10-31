//! Persistent terminals: the host side of `terminal/*`.
//!
//! The TypeScript package (`@deepseek-ai/dsh-terminal-controller`) is the
//! reference. A terminal is a shell on a pty owned by one session; any number of
//! clients attach to watch it, and the most recent attachment controls its input.

pub mod follower;
pub mod pty;
pub mod screen;
pub mod service;
