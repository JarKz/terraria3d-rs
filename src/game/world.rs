#![allow(dead_code)]
use crate::render::block::Block;
use crate::render::{Program, Shader, TextureAtlas, TextureAtlasConfiguration};

use std::collections::HashMap;
use std::thread::JoinHandle;

use parking_lot::lock_api::RawMutex;

use super::storage::STORAGE;

const DATA_FILE: &str = "data/test-world.dat";

pub struct World {
    seed: u32,
    blocksize: f32,

    render_center: (isize, isize),
    render_radius_in_chunks: usize,

    creating_chunks: Vec<JoinHandle<()>>,
    to_create_mesh: Vec<(isize, isize)>,
    rerendering_chunks: Vec<JoinHandle<()>>,

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
        let world = Self {
            seed,
            blocksize,
            render_center: (0, 0),
            render_radius_in_chunks: 4,

            creating_chunks: vec![],
            to_create_mesh: vec![],
            rerendering_chunks: vec![],

            texture_atlas: TextureAtlas::from(TextureAtlasConfiguration {
                image_path: String::from("res/images/block-texture-atlas.png"),
                square_size: 16,
            }),
            shader_program: Program::from([vshader, fshader]),
        };

        let radius = world.render_radius_in_chunks as isize;
        for x in -radius..=radius {
            for z in -radius..=radius {
                let chunk = Chunk::create(seed, x as f32 * offset, z as f32 * offset);

                if STORAGE.is_locked() {
                    unsafe {STORAGE.raw().unlock()}
                }
                STORAGE.lock().store_chunk(x, z, chunk);
            }
        }

        if STORAGE.is_locked() {
            unsafe {STORAGE.raw().unlock()}
        }
        for (_, chunk) in STORAGE.lock().all_chunks() {
            chunk.lock().create_mesh(blocksize);
        }

        // write_to_file(
        //     String::from(DATA_FILE),
        //     // String::from("Test-world"),
        //     &world.chunks,
        // )
        // .unwrap();

        world
    }

    const RAY_DISTANCE: usize = 100;
    const DISTANCE: f32 = 0.4f32;
    pub fn destroy_block_if_possible(&mut self, player_position: &Vec3, view_ray: &Vec3) {
        for i in 0..Self::RAY_DISTANCE {
            let player_looking_to = player_position + view_ray * i as f32 * Self::DISTANCE;
            let mut xyz_normalized = player_looking_to / self.blocksize;

            //normalize camera target position
            xyz_normalized = vec3(
                xyz_normalized.x.floor(),
                xyz_normalized.y.floor(),
                xyz_normalized.z.floor(),
            );

            xyz_normalized.x = shift_negative_coord(xyz_normalized.x);
            xyz_normalized.z = shift_negative_coord(xyz_normalized.z);

            let xyz_normalized = vec3(
                xyz_normalized.x as isize,
                xyz_normalized.y as isize,
                xyz_normalized.z as isize,
            );

            let xchunk_offset = xyz_normalized.x / Chunk::WIDTH_ISIZE;
            let zchunk_offset = xyz_normalized.z / Chunk::WIDTH_ISIZE;

            if STORAGE.is_locked() {
                unsafe {STORAGE.raw().unlock()}
            }
            if let Some(chunk) = STORAGE.lock().chunk(xchunk_offset, zchunk_offset) {
                let x = shift_negative_block_coord(xyz_normalized.x % Chunk::WIDTH_ISIZE);
                let y = xyz_normalized.y as usize;
                let z = shift_negative_block_coord(xyz_normalized.z % Chunk::WIDTH_ISIZE);

                if chunk.is_locked() {
                    unsafe {chunk.raw().unlock()}
                }

                let block = &mut chunk.lock().blocks[y][x as usize][z as usize];
                if Block::is_air(*block) {
                    continue;
                }

                *block = 0;

                if chunk.is_locked() {
                    unsafe {chunk.raw().unlock()}
                }
                chunk.lock().create_mesh(self.blocksize);

                self.rerender_neighbors(x, y, z, xchunk_offset, zchunk_offset);
                break;
            }
        }
    }

    fn rerender_neighbors(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        xchunk_offset: isize,
        zchunk_offset: isize,
    ) {
        if x == 0 {
            if STORAGE.is_locked() {
                unsafe {STORAGE.raw().unlock()}
            }
            if let Some(chunk) = STORAGE.lock().chunk(xchunk_offset - 1, zchunk_offset) {
                if chunk.is_locked() {
                    unsafe {chunk.raw().unlock()}
                }
                let block = chunk.lock().blocks[y][Chunk::WIDTH - 1][z as usize];
                if !Block::is_air(block) {
                    chunk.lock().create_mesh(self.blocksize);
                }
            }
        } else if x == Chunk::WIDTH - 1 {
            if STORAGE.is_locked() {
                unsafe {STORAGE.raw().unlock()}
            }
            if let Some(chunk) = STORAGE.lock().chunk(xchunk_offset + 1, zchunk_offset) {
                if chunk.is_locked() {
                    unsafe {chunk.raw().unlock()}
                }
                let block = chunk.lock().blocks[y][0][z as usize];
                if !Block::is_air(block) {
                    chunk.lock().create_mesh(self.blocksize);
                }
            }
        }

        if z == 0 {
            if STORAGE.is_locked() {
                unsafe {STORAGE.raw().unlock()}
            }
            if let Some(chunk) = STORAGE.lock().chunk(xchunk_offset, zchunk_offset - 1) {
                if chunk.is_locked() {
                    unsafe {chunk.raw().unlock()}
                }
                let block = chunk.lock().blocks[y][x as usize][Chunk::WIDTH - 1];
                if !Block::is_air(block) {
                    chunk.lock().create_mesh(self.blocksize);
                }
            }
        } else if z == Chunk::WIDTH - 1 {
            if STORAGE.is_locked() {
                unsafe {STORAGE.raw().unlock()}
            }
            if let Some(chunk) = STORAGE.lock().chunk(xchunk_offset, zchunk_offset + 1) {
                if chunk.is_locked() {
                    unsafe {chunk.raw().unlock()}
                }
                let block = chunk.lock().blocks[y][x as usize][0];
                if !Block::is_air(block) {
                    chunk.lock().create_mesh(self.blocksize);
                }
            }
        }
    }

    pub fn update_player_position(&mut self, player_position: &Vec3) {
        let normalized_ps = player_position * self.blocksize;
        let xplayer_pos = normalized_ps.x.round() as isize / Chunk::WIDTH_ISIZE;
        let zplayer_pos = normalized_ps.z.round() as isize / Chunk::WIDTH_ISIZE;
        if self.render_center.0 == xplayer_pos && self.render_center.1 == zplayer_pos {
            return;
        }

        let radius = self.render_radius_in_chunks as isize;
        let xmin = xplayer_pos - radius;
        let xmax = xplayer_pos + radius;
        let zmin = zplayer_pos - radius;
        let zmax = zplayer_pos + radius;
        let offset = self.blocksize * Chunk::WIDTH as f32;
        for x in xmin..=xmax {
            for z in zmin..=zmax {

                if STORAGE.is_locked() {
                    unsafe {STORAGE.raw().unlock()}
                }
                if STORAGE.lock().chunk(x, z).is_none() {
                    let seed = self.seed;
                    self.creating_chunks.push(
                        std::thread::Builder::new()
                            .stack_size(8 * 1024 * 1024)
                            .spawn(move || {
                                if STORAGE.is_locked() {
                                    unsafe {STORAGE.raw().unlock()}
                                }
                                STORAGE.lock().store_chunk(
                                    x,
                                    z,
                                    Chunk::create(seed, x as f32 * offset, z as f32 * offset),
                                );
                            })
                            .unwrap(),
                    );
                }
                self.to_create_mesh.push((x, z));
            }
        }

        let (xcenter, zcenter) = self.render_center;
        let old_xmin = xcenter - radius;
        let old_xmax = xcenter + radius;
        let old_zmin = zcenter - radius;
        let old_zmax = zcenter + radius;

        for x in old_xmin..=old_xmax {
            for z in old_zmin..=old_zmax {
                if x < xmin || xmax < x || z < zmin || zmax < z {
                    if STORAGE.is_locked() {
                        unsafe {STORAGE.raw().unlock()}
                    }
                    if let Some(chunk) = STORAGE.lock().chunk(x, z) {
                        chunk.lock().destroy_mesh();
                    }
                }
            }
        }
        self.render_center = (xplayer_pos, zplayer_pos);
    }

    const MAX_ACCEPTS: usize = 5;
    pub fn update_state(&mut self) {
        let mut indices = vec![];
        for i in 0..self.creating_chunks.len() {
            if self.creating_chunks[i].is_finished() {
                indices.push(i);
            }
            if indices.len() == Self::MAX_ACCEPTS {
                break;
            }
        }

        for i in indices.iter().rev() {
            self.creating_chunks.remove(*i);
        }

        if indices.len() > 0 || self.creating_chunks.len() > 0 {
            return;
        }

        let mut counter = 0;
        while let Some((x, z)) = self.to_create_mesh.pop() {
            if STORAGE.is_locked() {
                unsafe {STORAGE.raw().unlock()}
            }
            if let Some(chunk) = STORAGE.lock().chunk(x, z) {
                let chunk = chunk.clone();
                let blocksize = self.blocksize;
                self.rerendering_chunks.push(
                    std::thread::Builder::new()
                        .stack_size(8 * 1024 * 1024)
                        .spawn(move || {
                            if chunk.is_locked() {
                                unsafe {chunk.raw().unlock()}
                            }
                            chunk.lock().create_mesh(blocksize);
                        })
                        .unwrap(),
                );
            }
            counter += 1;
            if counter == Self::MAX_ACCEPTS {
                return;
            }
        }

        if counter > 0 || self.to_create_mesh.len() > 0 {
            return;
        }

        let mut indices = vec![];
        for i in 0..self.rerendering_chunks.len() {
            if self.rerendering_chunks[i].is_finished() {
                indices.push(i);
            }
            if indices.len() == Self::MAX_ACCEPTS {
                break;
            }
        }
        for i in indices.into_iter().rev() {
            self.rerendering_chunks.remove(i);
        }
    }

    pub fn render(&self, player: &super::player::Player) {
        self.texture_atlas.set_used();
        self.shader_program.set_used();

        self.shader_program.insert_mat4(
            &std::ffi::CString::new("projection").unwrap(),
            player.projection(),
        );
        self.shader_program
            .insert_mat4(&std::ffi::CString::new("view").unwrap(), &player.look_at());

        self.shader_program.insert_vec3(
            &std::ffi::CString::new("camera_position").unwrap(),
            player.position(),
        );
        self.shader_program.insert_vec3(
            &std::ffi::CString::new("fog_color").unwrap(),
            &vec3(0.3, 0.3, 0.5),
        );

        self.shader_program
            .insert_float(&std::ffi::CString::new("fog_min_dist").unwrap(), 35.);
        self.shader_program
            .insert_float(&std::ffi::CString::new("fog_max_dist").unwrap(), 40.);

        if STORAGE.is_locked() {
            unsafe {STORAGE.raw().unlock()}
        }
        for (_, chunk) in STORAGE.lock().all_chunks() {
            chunk.lock().render(&self.shader_program);
        }
    }
}

pub struct WorldHelper {
    blocksize: f32,
    chunks: HashMap<(isize, isize), Chunk>,
}

impl WorldHelper {
    pub fn get_block_at(xyz: &Vec3, blocksize: f32) -> Option<u64> {
        let mut xyz_normalized = xyz / blocksize;

        xyz_normalized.x = shift_negative_coord(xyz_normalized.x);
        xyz_normalized.z = shift_negative_coord(xyz_normalized.z);

        let xyz_normalized = vec3(
            xyz_normalized.x as isize,
            xyz_normalized.y as isize,
            xyz_normalized.z as isize,
        );

        let xchunk_offset = xyz_normalized.x / Chunk::WIDTH_ISIZE;
        let zchunk_offset = xyz_normalized.z / Chunk::WIDTH_ISIZE;

        if STORAGE.is_locked() {
            unsafe {STORAGE.raw().unlock()}
        }
        if let Some(chunk) = STORAGE.lock().chunk(xchunk_offset, zchunk_offset) {
            let x = shift_negative_block_coord(xyz_normalized.x % Chunk::WIDTH_ISIZE);
            let y = xyz_normalized.y as usize;
            let z = shift_negative_block_coord(xyz_normalized.z % Chunk::WIDTH_ISIZE);

            if chunk.is_locked() {
                unsafe { chunk.raw().unlock() }
            }
            return Some(chunk.lock().blocks[y][x as usize][z as usize]);
        }
        None
    }
}

fn shift_negative_block_coord(mut coord: isize) -> usize {
    if coord < 0 {
        coord += Chunk::WIDTH_ISIZE;
    }
    coord as usize
}

fn shift_negative_coord(mut coord: f32) -> f32 {
    if coord < 0. && coord % Chunk::WIDTH as f32 != 0. {
        coord -= Chunk::WIDTH as f32;
    }
    coord
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

use nalgebra_glm::{vec3, Vec3};
use noise::*;

impl Chunk {
    pub const WIDTH_ISIZE: isize = 16;

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

    fn create_mesh(&mut self, blocksize: f32) {
        self.mesh = Some(ChunkMesh::new(
            &self.blocks,
            self.xoffset,
            self.zoffset,
            blocksize,
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
            // Clonning this may ruin performance
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
