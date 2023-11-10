use std::collections::HashMap;

use super::world::chunk::Chunk;
use crate::render::mesh::ChunkMesh;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::sync::Arc;

pub static STORAGE: Lazy<Mutex<Storage>> = Lazy::new(|| Mutex::new(Storage::init()));

pub struct Storage {
    chunk_meshs: HashMap<(isize, isize), Arc<ChunkMesh>>,
    chunks: HashMap<(isize, isize), Arc<Mutex<Chunk>>>,
}

impl Storage {
    pub fn init() -> Self {
        Self {
            chunk_meshs: HashMap::new(),
            chunks: HashMap::new(),
        }
    }

    pub fn chunk(&self, xoffset: isize, zoffset: isize) -> Option<Arc<Mutex<Chunk>>> {
        self.chunks.get(&(xoffset, zoffset)).cloned()
    }

    pub fn store_chunk(&mut self, xoffset: isize, zoffset: isize, chunk: Chunk) {
        self.chunks
            .insert((xoffset, zoffset), Arc::new(Mutex::new(chunk)));
    }

    pub fn all_chunks(&self) -> &HashMap<(isize, isize), Arc<Mutex<Chunk>>> {
        &self.chunks
    }

    pub fn get_mesh(&self, xoffset: isize, zoffset: isize) -> Option<Arc<ChunkMesh>> {
        self.chunk_meshs.get(&(xoffset, zoffset)).cloned()
    }

    pub fn update_mesh(&mut self, xoffset: isize, zoffset: isize, mesh: ChunkMesh) {
        self.chunk_meshs.insert((xoffset, zoffset), Arc::new(mesh));
    }

    pub fn destroy_mesh(&mut self, xoffset: isize, zoffset: isize) {
        self.chunk_meshs.remove(&(xoffset, zoffset));
    }

    pub fn all_mesh(&self) -> &HashMap<(isize, isize), Arc<ChunkMesh>> {
        &self.chunk_meshs
    }
}

#[macro_export]
macro_rules! lock {
    ($to_lock:ident) => {{
        if $to_lock.is_locked() {
            use parking_lot::lock_api::RawMutex;
            unsafe { $to_lock.raw().unlock() }
        }
        $to_lock.lock()
    }};
}
