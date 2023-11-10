/// Block data stores as 3D array by yxz (height, x and z offset) and their u64 looks like:
/// 48 bits - metatdata information
/// 16 bits - block id
pub type ChunkBlocks = [[[u64; Chunk::WIDTH]; Chunk::WIDTH]; Chunk::HEIGHT];

pub struct Chunk {
    xoffset: f32,
    zoffset: f32,

    /// Block data stores as 3D array by yxz (height, x and z offset) and their u64 looks like:
    /// 48 bits - metatdata information
    /// 16 bits - block id
    blocks: ChunkBlocks,
}

use noise::*;
use parking_lot::Mutex;

static SEED: Mutex<u32> = Mutex::new(0);

impl Chunk {
    pub const WIDTH_ISIZE: isize = 16;

    pub const WIDTH: usize = 16;
    pub const HEIGHT: usize = 216;

    const SURFACE_LINE: usize = 100;

    pub fn set_seed(seed: u32) {
        *SEED.lock() = seed;
    }

    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn create(xoffset: f32, zoffset: f32) -> Self {
        let mut blocks = [[[0; Self::WIDTH]; Self::WIDTH]; Self::HEIGHT];

        let perlin = noise::Perlin::new(*SEED.lock());
        let xoffset_f64 = xoffset as f64;
        let zoffset_f64 = zoffset as f64;
        for x in 0..Self::WIDTH {
            for z in 0..Self::WIDTH {
                let val = perlin.get([
                    (xoffset_f64 + x as f64) / 100.,
                    (zoffset_f64 + z as f64) / 100.,
                ]);
                let s_val = Self::SURFACE_LINE as f64 + (20.0 * val);
                let surface_height = s_val as usize;

                for y in 0..Self::HEIGHT {
                    let floor = &mut blocks[y];
                    floor[x][z] = if y < surface_height { 1 } else { 0 }
                }
            }
        }

        Chunk {
            xoffset,
            zoffset,
            blocks,
        }
    }

    pub fn xoffset(&self) -> f32 {
        self.xoffset
    }

    pub fn zoffset(&self) -> f32 {
        self.zoffset
    }

    pub fn blocks(&self) -> &ChunkBlocks {
        &self.blocks
    }

    pub fn mut_block_at(&mut self, x: usize, z: usize, y: usize) -> &mut u64 {
        &mut self.blocks[y][x][z]
    }

    pub fn block_at(&self, x: usize, z: usize, y: usize) -> u64 {
        self.blocks[y][x][z]
    }

    pub fn set_block_at(&mut self, x: usize, z: usize, y: usize, new_block: u64) {
        self.blocks[y][x][z] = new_block;
    }

    pub fn anticipated_block_at(x: usize, z: usize, y: usize, xoffset: f32, zoffset: f32) -> u64 {
        let perlin = noise::Perlin::new(*SEED.lock());
        let xoffset_f64 = xoffset as f64;
        let zoffset_f64 = zoffset as f64;
        let val = perlin.get([
            (xoffset_f64 + x as f64) / 100.,
            (zoffset_f64 + z as f64) / 100.,
        ]);
        let s_val = Self::SURFACE_LINE as f64 + (20.0 * val);
        let surface_height = s_val as usize;
        if y < surface_height {
            1
        } else {
            0
        }
    }
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        Self {
            xoffset: self.xoffset,
            zoffset: self.zoffset,
            blocks: self.blocks.clone(),
        }
    }
}

#[macro_export]
macro_rules! foreach_block {
    (($y:ident; $x:ident; $z:ident) $body:expr) => {
        foreach_block!(($y, {}; $x, {}; $z, {}) $body)
    };
    (($y:ident, $yafter:expr; $x:ident, $xafter:expr; $z:ident, $zafter:expr) $body:expr) => {
        for $y in 0..crate::game::world::chunk::Chunk::HEIGHT {
            for $x in 0..crate::game::world::chunk::Chunk::WIDTH {
                for $z in 0..crate::game::world::chunk::Chunk::WIDTH {
                    $body
                    $zafter
                }
                $xafter
            }
            $yafter
        }
    };
}

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};

fn read_from_file(
    filepath: String,
    blocksize: f32,
) -> std::io::Result<HashMap<(isize, isize), Chunk>> {
    let file = File::open(filepath)?;
    let mut reader = BufReader::new(file);
    let mut chunks = HashMap::new();
    let mut i_b = [0; std::mem::size_of::<isize>()];
    while let Ok(n) = reader.read(&mut i_b) {
        if n == 0 {
            break;
        }
        let mut j_b = [0; std::mem::size_of::<isize>()];
        reader.read(&mut j_b)?;

        let mut blocks = [[[0; Chunk::WIDTH]; Chunk::WIDTH]; Chunk::HEIGHT];
        let mut block = [0; std::mem::size_of::<u64>()];

        for y in 0..Chunk::HEIGHT {
            let floor = &mut blocks[y];

            for x in 0..Chunk::WIDTH {
                let zarray = &mut floor[x];

                for z in 0..Chunk::WIDTH {
                    reader.read(&mut block)?;
                    zarray[z] = u64::from_be_bytes(block);
                }
            }
        }

        let i = isize::from_be_bytes(i_b);
        let j = isize::from_be_bytes(j_b);
        let chunk = Chunk {
            xoffset: i as f32 * blocksize * Chunk::WIDTH as f32,
            zoffset: j as f32 * blocksize * Chunk::WIDTH as f32,
            blocks,
            // mesh: None,
        };
        chunks.insert((i, j), chunk);
    }
    Ok(chunks)
}

fn write_to_file(
    filepath: String,
    // worldname: String,
    chunks: &HashMap<(isize, isize), Chunk>,
) -> std::io::Result<()> {
    let mut file = File::create(filepath)?;
    // file.write(worldname.as_bytes())?;

    for ((i, j), chunk) in chunks {
        file.write(&i.to_be_bytes())?;
        file.write(&j.to_be_bytes())?;

        let blocks = chunk.blocks();

        for y in 0..Chunk::HEIGHT {
            let floor = &blocks[y];

            for x in 0..Chunk::WIDTH {
                let zarray = &floor[x];

                for z in 0..Chunk::WIDTH {
                    file.write(&zarray[z].to_be_bytes())?;
                }
            }
        }
    }
    Ok(())
}
