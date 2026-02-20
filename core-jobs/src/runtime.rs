use std::future::Future;
use tokio::sync::{mpsc, oneshot};

/// Spawn a lightweight asynchronous task (goroutine-style helper).
pub fn go<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future)
}

/// Create a buffered channel for task-to-task communication.
pub fn channel<T>(buffer: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::channel(buffer)
}

/// Create an unbounded channel for fire-and-forget events.
pub fn unbounded_channel<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
    mpsc::unbounded_channel()
}

/// Create a one-shot channel for request/response style handoff.
pub fn oneshot_channel<T>() -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
    oneshot::channel()
}
