#![allow(dead_code)]

use crate::render::{Program, Shader, TextureAtlas, TextureAtlasConfiguration};

pub struct World {
    seed: u32,
    blocksize: f32,

    //TODO:
    //this is temporary, need in future to save in file as bytes
    chunks: Vec<Vec<Chunk>>,
    texture_atlas: TextureAtlas,
    shader_program: Program,
}

impl World {
    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn new(seed: u32, blocksize: f32) -> Self {
        let offset = blocksize * Chunk::WIDTH as f32;
        let mut chunks = vec![vec![]; 3];
        for x in 0..1 {
            for z in 0..1 {
        // for x in 0..3 {
        //     for z in 0..3 {
                let mut chunk = Chunk::create(seed, x as f32 * offset, z as f32 * offset);
                chunk.create_mesh(blocksize);
                chunks[x].push(chunk);
            }
        }

        let vshader = Shader::from_vertex(String::from("res/shaders/block-vert.glsl")).unwrap();
        let fshader =
            Shader::from_fragment(String::from("res/shaders/block-frag.glsl")).unwrap();
        Self {
            seed,
            blocksize,
            chunks,
            texture_atlas: TextureAtlas::from(TextureAtlasConfiguration {
                image_path: String::from("res/images/block-texture-atlas.png"),
                square_size: 16,
            }),
            shader_program: Program::from([vshader, fshader]),
        }
    }

    pub fn update_shaders(&mut self, _player_position: &Vec3) {
        //TODO:
        //Need add defining direction from player position to chunks for generating only these
        //vertices which need to display.
    }

    pub fn render(&self, projection: &Mat4, view: &Mat4) {
        self.texture_atlas.set_used();
        self.shader_program.set_used();
        self.shader_program.insert_mat4(&std::ffi::CString::new("projection").unwrap(), projection);
        self.shader_program.insert_mat4(&std::ffi::CString::new("view").unwrap(), view);
        for zchunk in &self.chunks {
            for chunk in zchunk {
                chunk.render(&self.shader_program);
            }
        }
    }

    pub fn test(&self) {
        // self.texture_atlas.set_used();
        self.shader_program.set_used();
    }
}

/// Block data stores as 3D array by yxz (height, x and z offset) and their u64 looks like:
/// 48 bits - metatdata information
/// 16 bits - block id
pub type ChunkBlocks = [[[u64; Chunk::WIDTH]; Chunk::WIDTH]; Chunk::HEIGHT];

use crate::render::mesh::ChunkMesh;

pub struct Chunk {
    xoffset: f32,
    zoffset: f32,

    /// Block data stores as 3D array by yxz (height, x and z offset) and their u64 looks like:
    /// 48 bits - metatdata information
    /// 16 bits - block id
    blocks: ChunkBlocks,
    mesh: Option<ChunkMesh>,
}

use nalgebra_glm::{Mat4, Vec3};
use noise::*;

impl Chunk {
    pub const WIDTH: usize = 16;
    pub const HEIGHT: usize = 216;

    const SURFACE_LINE: usize = 100;

    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn create(seed: u32, xoffset: f32, zoffset: f32) -> Self {
        let mut blocks = [[[0; Self::WIDTH]; Self::WIDTH]; Self::HEIGHT];

        let perlin = noise::Perlin::new(seed);
        let xoffset_f64 = xoffset as f64;
        let zoffset_f64 = zoffset as f64;
        for y in 0..Self::HEIGHT {
            let floor = &mut blocks[y];

            for x in 0..Self::WIDTH {
                for z in 0..Self::WIDTH {
                    let val = perlin.get([xoffset_f64 + x as f64, zoffset_f64 + z as f64]);
                    let surface_height = (Self::SURFACE_LINE as f64 * (20.0 * val)) as usize;
                    floor[x][z] = if y < surface_height { 1 } else { 0 }
                }
            }
        }

        Chunk {
            xoffset,
            zoffset,
            blocks,
            mesh: None
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

    fn create_mesh(&mut self, blocksize: f32) {
        self.mesh = Some(ChunkMesh::new(&self.blocks, self.xoffset, self.zoffset, blocksize));
    }

    fn render(&self, shader_program: &Program) {
        if let Some(mesh) = &self.mesh {
            mesh.render(shader_program);
        }
    }
}

impl Clone for Chunk {
    fn clone(&self) -> Self {
        Self {
            xoffset: self.xoffset,
            zoffset: self.zoffset,
            blocks: self.blocks.clone(),
            mesh: self.mesh.clone(),
        }
    }
}
