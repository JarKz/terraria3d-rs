#![allow(dead_code)]

use crate::render::{Program, Shader, TextureAtlas, TextureAtlasConfiguration};

use std::collections::HashMap;

const DATA_FILE: &str = "data/test-world.dat";

pub struct World {
    seed: u32,
    blocksize: f32,

    render_center: (isize, isize),
    render_radius_in_chunks: usize,

    chunks: HashMap<(isize, isize), Chunk>,
    texture_atlas: TextureAtlas,
    shader_program: Program,
}

impl World {
    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn new(seed: u32, blocksize: f32) -> Self {
        let offset = blocksize * Chunk::WIDTH as f32;

        let vshader = Shader::from_vertex(String::from("res/shaders/block-vert.glsl")).unwrap();
        let fshader = Shader::from_fragment(String::from("res/shaders/block-frag.glsl")).unwrap();
        let mut world = Self {
            seed,
            blocksize,
            render_center: (0, 0),
            render_radius_in_chunks: 2,
            chunks: HashMap::new(),
            texture_atlas: TextureAtlas::from(TextureAtlasConfiguration {
                image_path: String::from("res/images/block-texture-atlas.png"),
                square_size: 16,
            }),
            shader_program: Program::from([vshader, fshader]),
        };

        for x in -2..3 {
            for z in -2..3 {
                let chunk = Chunk::create(seed, x as f32 * offset, z as f32 * offset);
                world.chunks.insert((x, z), chunk);
            }
        }

        let world_helper = WorldHelper {
            blocksize: world.blocksize,
            chunks: world.chunks.clone(),
        };
        for (_, chunk) in &mut world.chunks {
            chunk.create_mesh(&world_helper, blocksize);
        }
        write_to_file(
            String::from(DATA_FILE),
            // String::from("Test-world"),
            &world.chunks,
        )
        .unwrap();

        world
    }

    pub fn update_position(&mut self, player_position: &Vec3) {
        let normalized_ps = player_position * self.blocksize;
        let xplayer_pos = normalized_ps.x as isize / Chunk::WIDTH as isize;
        let zplayer_pos = normalized_ps.z as isize / Chunk::WIDTH as isize;
        if self.render_center.0 == xplayer_pos && self.render_center.1 == zplayer_pos {
            return;
        }

        self.render_center = (xplayer_pos, zplayer_pos);

        let radius = self.render_radius_in_chunks as isize;
        let xmin = xplayer_pos - radius;
        let xmax = xplayer_pos + radius;
        let zmin = zplayer_pos - radius;
        let zmax = zplayer_pos + radius;
        for x in xmin..=xmax {
            for z in zmin..=zmax {
                if self.chunks.get(&(x, z)).is_none() {
                    let new_chunk = Chunk::create(
                        self.seed,
                        x as f32 * self.blocksize * Chunk::WIDTH as f32,
                        z as f32 * self.blocksize * Chunk::WIDTH as f32,
                    );
                    self.chunks.insert((x, z), new_chunk);
                }
            }
        }

        let world_helper = WorldHelper {
            blocksize: self.blocksize,
            chunks: self.chunks.clone(),
        };

        let mut to_remove = vec![];
        for ((i, j), chunk) in &mut self.chunks {
            if *i < xmin || xmax < *i || *j < zmin || zmax < *j {
                to_remove.push((*i, *j));
            } else {
                chunk.create_mesh(&world_helper, self.blocksize);
            }
        }

        for (i, j) in to_remove {
            self.chunks.get_mut(&(i, j)).unwrap().destroy_mesh();
        }
    }

    pub fn render(&self, projection: &Mat4, view: &Mat4) {
        self.texture_atlas.set_used();
        self.shader_program.set_used();
        self.shader_program
            .insert_mat4(&std::ffi::CString::new("projection").unwrap(), projection);
        self.shader_program
            .insert_mat4(&std::ffi::CString::new("view").unwrap(), view);
        for (_, chunk) in &self.chunks {
            chunk.render(&self.shader_program);
        }
    }
}

pub struct WorldHelper {
    blocksize: f32,
    chunks: HashMap<(isize, isize), Chunk>,
}

impl WorldHelper {
    //TODO: Fix here for negative numbers
    pub fn get_block_at(&self, xyz: &Vec3) -> Option<u64> {
        let mut xyz_normalized = xyz / self.blocksize;
        if xyz_normalized.x < 0. {
            xyz_normalized.x -= Chunk::WIDTH as f32;
        }
        if xyz_normalized.z < 0. {
            xyz_normalized.z -= Chunk::WIDTH as f32;
        }
        let xchunk_offset = xyz_normalized.x as isize / Chunk::WIDTH as isize;
        let zchunk_offset = xyz_normalized.z as isize / Chunk::WIDTH as isize;
        if let Some(chunk) = self.chunks.get(&(xchunk_offset, zchunk_offset)) {
            let mut x = xyz_normalized.x as isize % Chunk::WIDTH as isize;
            if x < 0 {
                x += Chunk::WIDTH as isize;
            }
            let y = xyz_normalized.y as usize;
            let mut z = xyz_normalized.z as isize % Chunk::WIDTH as isize;
            if z < 0 {
                z += Chunk::WIDTH as isize;
            }
            return Some(chunk.blocks[y][x as usize][z as usize]);
        }
        None
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
            mesh: None,
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

    fn create_mesh(&mut self, world: &WorldHelper, blocksize: f32) {
        self.mesh = Some(ChunkMesh::new(
            &self.blocks,
            self.xoffset,
            self.zoffset,
            blocksize,
            world,
        ));
    }

    fn destroy_mesh(&mut self) {
        self.mesh = None;
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
            //Don't clone because of destroying vao, vbo and ebo
            mesh: None,
        }
    }
}

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
            mesh: None,
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

        let blocks = chunk.blocks;

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
