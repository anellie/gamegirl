use crate::system::io::cartridge::Cartridge;
use std::path::PathBuf;

/// Empty struct holding methods used for interacting with the file system,
/// for storing game save data / cartridge RAM.
/// On native, will load/store `.sav` files next to game ROM files.
/// On WASM, will load/store into browser local storage.
pub struct Storage;

impl Storage {
    /// Save the given cart's RAM to disk.
    /// Path should always be Some and point to the game ROM path,
    /// since this is on native.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(path: Option<PathBuf>, cart: &Cartridge) {
        let path = Self::get_sav_path(path.unwrap());
        std::fs::write(path, &cart.ram()).ok(); // TODO handle error
    }

    /// Load the given cart's RAM from disk, replacing existing RAM.
    /// Path should always be Some and point to the game ROM path,
    /// since this is on native.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(path: Option<PathBuf>, cart: &mut Cartridge) {
        let path = Self::get_sav_path(path.unwrap());
        if let Ok(ram) = std::fs::read(path) {
            cart.load_ram(ram);
        }
    }

    /// "hello/my/rom.gb" -> "hello/my/rom.sav"
    #[cfg(not(target_arch = "wasm32"))]
    fn get_sav_path(mut path: PathBuf) -> PathBuf {
        let base = path.file_stem().unwrap().to_str().unwrap();
        let name = format!("{base}.sav");
        path.pop();
        path.push(name);
        path
    }

    /// Save the given cart's RAM to local storage.
    /// Path will always be None, since this is WASM.
    #[cfg(target_arch = "wasm32")]
    pub fn save(_path: Option<PathBuf>, cart: &Cartridge) {
        let content = base64::encode(cart.ram());
        Self::local_storage().set(&cart.title(true), &content).ok();
    }

    /// Load the given cart's RAM from disk, replacing existing RAM.
    /// Path will always be None, since this is WASM.
    #[cfg(target_arch = "wasm32")]
    pub fn load(_path: Option<PathBuf>, cart: &mut Cartridge) {
        let base64 = Self::local_storage().get(&cart.title(true)).ok().flatten();
        if let Some(ram) = base64.and_then(|ram| base64::decode(ram).ok()) {
            cart.load_ram(ram);
        }
    }

    /// Get the browser's local storage.
    #[cfg(target_arch = "wasm32")]
    fn local_storage() -> web_sys::Storage {
        web_sys::window().unwrap().local_storage().unwrap().unwrap()
    }
}