use std::collections::HashMap;

use super::world::Chunk;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

pub static STORAGE: Lazy<Mutex<Storage>> = Lazy::new(|| Mutex::new(Storage::init()));

pub struct Storage {
    chunks: HashMap<(isize, isize), Arc<Mutex<Chunk>>>,
}

impl Storage {
    pub fn init() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn chunk(&self, xoffset: isize, zoffset: isize) -> Option<Arc<Mutex<Chunk>>> {
        if let Some(chunk) = self.chunks.get(&(xoffset, zoffset)) {
            return Some(chunk.clone());
        }
        None
    }

    pub fn store_chunk(&mut self, xoffset: isize, zoffset: isize, chunk: Chunk) {
        self.chunks
            .insert((xoffset, zoffset), Arc::new(Mutex::new(chunk)));
    }

    pub fn all_chunks(&self) -> &HashMap<(isize, isize), Arc<Mutex<Chunk>>> {
        &self.chunks
    }
}
