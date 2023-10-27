#[allow(dead_code)]
pub struct World {
    seed: u32,
    blocksize: f32,

    //TODO:
    //this is temporary, need in future to save in file as bytes
    chunks: Vec<Vec<Chunk>>,
}

impl World {

    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn new(seed: u32, blocksize: f32) -> Self {
        let offset = blocksize * 16.0;
        let mut chunks = vec![vec![]; 3];
        for x in 0..3 {
            for z in 0..3 {
                chunks[x].push(Chunk::create(seed, x as f32 * offset, z as f32 * offset));
            }
        }
        Self {
            seed,
            blocksize,
            chunks,
        }
    }

    pub fn update_shaders(&self, _player_position: &Vec3) {
        //TODO:
        //Need add defining direction from player position to chunks for generating only these
        //vertices which need to display.
    }
}

pub struct Chunk {
    xoffset: f32,
    zoffset: f32,

    /// Block data stores as 3D array by yxz (height, x and z offset) and their u64 looks like:
    /// 48 bits - metatdata information
    /// 16 bits - block id
    blocks: [[[u64; Chunk::WIDTH]; Chunk::WIDTH]; Chunk::HEIGHT],
}

use nalgebra_glm::Vec3;
use noise::*;

impl Chunk {
    const WIDTH: usize = 16;
    const HEIGHT: usize = 216;

    const SURFACE_LINE: usize = 100;

    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn create(seed: u32, xoffset: f32, zoffset: f32) -> Self {
        let mut blocks = [[[0; Self::WIDTH]; Self::WIDTH]; Self::HEIGHT];

        for y in 0..Self::SURFACE_LINE {
            let floor = &mut blocks[y];

            for x in 0..Self::WIDTH {
                for z in 0..Self::WIDTH {
                    floor[x][z] = 1;
                }
            }
        }

        let perlin = noise::Perlin::new(seed);
        let xoffset_f64 = xoffset as f64;
        let yoffset_f64 = zoffset as f64;
        for y in Self::SURFACE_LINE..Self::HEIGHT {
            let floor = &mut blocks[y];

            for x in 0..Self::WIDTH {
                for z in 0..Self::WIDTH {
                    let val = perlin.get([xoffset_f64 + x as f64, yoffset_f64 + z as f64]);
                    if val < 0.5 {
                        floor[x][z] = 1;
                    } else {
                        floor[x][z] = 0;
                    }
                }
            }
        }

        Chunk {
            xoffset,
            zoffset,
            blocks,
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
