use crate::gui::Message;
use rfd::FileHandle;
use std::{future::Future, path::PathBuf, sync::mpsc};

/// A file picked by the user.
pub struct File {
    /// File content in bytes
    pub content: Vec<u8>,
    /// Path of the file. Always None on WASM.
    pub path: Option<PathBuf>,
}

/// Open a file dialog. This operation is async and returns immediately,
/// sending a [Message] once the user has picked a file.
pub fn open(sender: mpsc::Sender<Message>) {
    let task = rfd::AsyncFileDialog::new()
        .add_filter("GameGirl games", &["gb", "gbc", "gba"])
        .pick_file();

    execute(async move {
        let file = task.await;
        if let Some(file) = file {
            let path = path(&file);
            let content = file.read().await;
            sender.send(Message::FileOpen(File { content, path })).ok();
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn path(f: &FileHandle) -> Option<PathBuf> {
    Some(f.path().to_path_buf())
}

#[cfg(target_arch = "wasm32")]
fn path(_f: &FileHandle) -> Option<PathBuf> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || futures_executor::block_on(f));
}

#[cfg(target_arch = "wasm32")]
fn execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}