#![allow(dead_code)]

pub mod chunk;
use chunk::Chunk;

use crate::render::block::Block;
use crate::render::mesh::ChunkMesh;
use crate::render::{Program, Shader, TextureAtlas, TextureAtlasConfiguration};

use nalgebra_glm::{vec3, Vec3};
use std::collections::HashMap;
use std::thread::JoinHandle;

use super::storage::STORAGE;
use crate::lock;

const DATA_FILE: &str = "data/test-world.dat";

macro_rules! foreach_in_radius {
    (($x:ident, $z:ident; $xcenter:ident, $zcenter:ident; $radius:ident) $body:expr) => {
        for $x in ($xcenter - $radius)..=($xcenter + $radius) {
            for $z in ($zcenter - $radius)..=($zcenter + $radius) {
                $body
            }
        }
    };
}

macro_rules! new_thread {
    ((size = $stack_size:expr) {$body:expr}) => {
        std::thread::Builder::new()
            .stack_size($stack_size)
            .spawn($body)
            .unwrap()
    };
}

macro_rules! size {
    ($val:expr, MiB) => {
        size!($val, KiB) * 1024
    };
    ($val:expr, KiB) => {
        $val * 1024
    };
}

macro_rules! get_block_position {
    (($x:ident, $y:ident, $z:ident, $xoffset:ident, $zoffset:ident) <= $coord:ident) => {
        $coord.x = shift_negative_coord($coord.x);
        $coord.z = shift_negative_coord($coord.z);

        let xyz_int = vec3($coord.x as isize, $coord.y as isize, $coord.z as isize);

        use crate::game::world::chunk::Chunk;

        let $xoffset = xyz_int.x / Chunk::WIDTH_ISIZE;
        let $zoffset = xyz_int.z / Chunk::WIDTH_ISIZE;

        let $x = shift_negative_block_coord(xyz_int.x % Chunk::WIDTH_ISIZE);
        let $y = xyz_int.y as usize;
        let $z = shift_negative_block_coord(xyz_int.z % Chunk::WIDTH_ISIZE);
    };
}

pub struct World {
    seed: u32,
    blocksize: f32,

    render_center: (isize, isize),
    render_radius_in_chunks: usize,

    threads: Vec<JoinHandle<()>>,
    texture_atlas: TextureAtlas,
    shader_program: Program,
}

impl World {
    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn new(seed: u32, blocksize: f32) -> Self {
        let world = Self {
            seed,
            blocksize,
            render_center: (0, 0),
            render_radius_in_chunks: 8,

            threads: vec![],

            texture_atlas: TextureAtlas::from(TextureAtlasConfiguration {
                image_path: String::from("res/images/block-texture-atlas.png"),
                square_size: 16,
            }),
            shader_program: Program::from([
                Shader::from_vertex(String::from("res/shaders/block-vert.glsl")).unwrap(),
                Shader::from_fragment(String::from("res/shaders/block-frag.glsl")).unwrap(),
            ]),
        };

        Chunk::set_seed(seed);

        let offset = blocksize * Chunk::WIDTH as f32;
        Self::init_world(&world, offset);
        world
    }

    fn init_world(world: &World, offset: f32) {
        let radius = world.render_radius_in_chunks as isize;
        for x in -radius..=radius {
            for z in -radius..=radius {
                let chunk = Chunk::create(x as f32 * offset, z as f32 * offset);

                STORAGE.lock().store_chunk(x, z, chunk);
            }
        }

        for (_, chunk) in STORAGE.lock().all_chunks() {
            let chunk = chunk.lock();
            let x = (chunk.xoffset() / offset) as isize;
            let z = (chunk.zoffset() / offset) as isize;
            lock!(STORAGE).update_mesh(
                x,
                z,
                ChunkMesh::new(
                    chunk.blocks(),
                    chunk.xoffset(),
                    chunk.zoffset(),
                    world.blocksize,
                ),
            );
        }
    }

    const RAY_DISTANCE: usize = 100;
    const DISTANCE: f32 = 0.4f32;
    pub fn destroy_block_if_possible(&mut self, player_position: &Vec3, view_ray: &Vec3) {
        for i in 0..Self::RAY_DISTANCE {
            let player_looking_to = player_position + view_ray * i as f32 * Self::DISTANCE;
            let mut xyz_normalized = player_looking_to / self.blocksize;

            //normalize camera target position
            xyz_normalized
                .iter_mut()
                .for_each(|axis| *axis = axis.floor());

            get_block_position!((x, y, z, xoffset, zoffset) <= xyz_normalized);

            if let Some(chunk) = STORAGE.lock().chunk(xoffset, zoffset) {
                let mut chunk = chunk.lock();
                let block = chunk.mut_block_at(x as usize, z as usize, y);
                if Block::is_air(*block) {
                    continue;
                }

                *block = 0;
                lock!(STORAGE).update_mesh(
                    xoffset,
                    zoffset,
                    ChunkMesh::new(
                        chunk.blocks(),
                        chunk.xoffset(),
                        chunk.zoffset(),
                        self.blocksize,
                    ),
                );

                self.rerender_neighbors(x, y, z, xoffset, zoffset);
                break;
            }
        }
    }

    fn rerender_neighbors(&mut self, x: usize, y: usize, z: usize, xoffset: isize, zoffset: isize) {
        macro_rules! update_mesh {
            ((($xoffset:expr, $zoffset:expr), $chunk:ident, $blocksize:ident) => $storage:ident) => {
                lock!($storage).update_mesh(
                    $xoffset,
                    $zoffset,
                    ChunkMesh::new(
                        $chunk.blocks(),
                        $chunk.xoffset(),
                        $chunk.zoffset(),
                        $blocksize,
                    ),
                );
            };
        }
        let blocksize = self.blocksize;
        if x == 0 {
            if let Some(chunk) = lock!(STORAGE).chunk(xoffset - 1, zoffset) {
                let chunk = lock!(chunk);
                let block = chunk.blocks()[y][Chunk::WIDTH - 1][z as usize];
                if !Block::is_air(block) {
                    update_mesh! (((xoffset - 1, zoffset), chunk, blocksize) =>  STORAGE);
                }
            }
        } else if x == Chunk::WIDTH - 1 {
            if let Some(chunk) = lock!(STORAGE).chunk(xoffset + 1, zoffset) {
                let chunk = lock!(chunk);
                let block = chunk.blocks()[y][0][z as usize];
                if !Block::is_air(block) {
                    update_mesh! (((xoffset + 1, zoffset), chunk, blocksize) =>  STORAGE);
                }
            }
        }

        if z == 0 {
            if let Some(chunk) = lock!(STORAGE).chunk(xoffset, zoffset - 1) {
                let chunk = lock!(chunk);
                let block = chunk.blocks()[y][x as usize][Chunk::WIDTH - 1];
                if !Block::is_air(block) {
                    update_mesh! (((xoffset, zoffset - 1), chunk, blocksize) =>  STORAGE);
                }
            }
        } else if z == Chunk::WIDTH - 1 {
            if let Some(chunk) = lock!(STORAGE).chunk(xoffset, zoffset + 1) {
                let chunk = lock!(chunk);
                let block = chunk.blocks()[y][x as usize][0];
                if !Block::is_air(block) {
                    update_mesh! (((xoffset, zoffset + 1), chunk, blocksize) =>  STORAGE);
                }
            }
        }
    }

    pub fn update_player_position(&mut self, player_position: &Vec3) {
        let normalized_ps = player_position * self.blocksize;
        let xplayer_pos = normalized_ps.x.floor() as isize / Chunk::WIDTH_ISIZE;
        let zplayer_pos = normalized_ps.z.floor() as isize / Chunk::WIDTH_ISIZE;
        if self.render_center.0 == xplayer_pos && self.render_center.1 == zplayer_pos {
            return;
        }

        let radius = self.render_radius_in_chunks as isize;
        let offset = self.blocksize * Chunk::WIDTH as f32;
        let blocksize = self.blocksize;

        foreach_in_radius! {
            (x, z; xplayer_pos, zplayer_pos; radius) {

                if let Some(chunk) = lock!(STORAGE).chunk(x, z) {
                    if lock!(STORAGE).get_mesh(x, z).is_some() {
                        continue;
                    }

                    let thread = new_thread! {
                        (size = size!(8, MiB)) {
                            move || {
                                let chunk = chunk.lock();
                                lock!(STORAGE).update_mesh(
                                    x,
                                    z,
                                    ChunkMesh::new(
                                        chunk.blocks(),
                                        chunk.xoffset(),
                                        chunk.zoffset(),
                                        blocksize,
                                    ),
                                );
                            }
                        }
                    };
                    self.threads.push(thread);
                } else  {
                    let thread = new_thread!{
                        (size = size!(8, MiB)) {
                            move || {
                                let xoffset = x as f32 * offset;
                                let zoffset = z as f32 * offset;
                                let chunk = Chunk::create(xoffset, zoffset);
                                let mesh =
                                    ChunkMesh::new(chunk.blocks(), xoffset, zoffset, blocksize);
                                let mut storage = STORAGE.lock();
                                storage.store_chunk(x, z, chunk);
                                storage.update_mesh(x, z, mesh);
                            }
                        }
                    };
                    self.threads.push(thread);
                }
            }
        };

        let (xcenter, zcenter) = self.render_center;

        foreach_in_radius! {
            (x, z; xcenter, zcenter; radius) {
                if (x - xplayer_pos).abs() > radius || (z - zplayer_pos).abs() > radius {
                    STORAGE.lock().destroy_mesh(x, z);
                }

            }
        };
        self.render_center = (xplayer_pos, zplayer_pos);
    }

    const MAX_ACCEPTS: usize = 5;
    pub fn update_state(&mut self) {
        let mut indices = vec![];
        for i in 0..self.threads.len() {
            if self.threads[i].is_finished() {
                indices.push(i);
            }
            if indices.len() == Self::MAX_ACCEPTS {
                break;
            }
        }

        for i in indices.iter().rev() {
            self.threads.remove(*i);
        }

        if indices.len() > 0 || self.threads.len() > 0 {
            return;
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

        for mesh in STORAGE.lock().all_mesh().clone().values() {
            mesh.render(&self.shader_program);
        }
    }
}

pub struct WorldHelper {
    blocksize: f32,
    chunks: HashMap<(isize, isize), Chunk>,
}

impl WorldHelper {
    pub fn get_block_at(xyz: &Vec3, blocksize: f32) -> u64 {
        let mut xyz_normalized = xyz / blocksize;
        get_block_position!((x, y, z, xoffset, zoffset) <= xyz_normalized);

        if let Some(chunk) = lock!(STORAGE).chunk(xoffset, zoffset) {
            lock!(chunk).blocks()[y][x as usize][z as usize]
        } else {
            let offset = Chunk::WIDTH as f32 * blocksize;
            Chunk::anticipated_block_at(x, z, y, xoffset as f32 * offset, zoffset as f32 * offset)
        }
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
