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
    (($x:ident, $z:ident; $xcenter:expr, $zcenter:expr; $radius:ident) $body:expr) => {
        for $x in ($xcenter - $radius)..=($xcenter + $radius) {
            for $z in ($zcenter - $radius)..=($zcenter + $radius) {
                $body
            }
        }
    };
    (($x:ident, $y:ident, $z:ident; $xcenter:expr, $ycenter:expr, $zcenter:expr; $radius:ident) $body:expr) => {
        for $y in ($ycenter - $radius).max(0)..=($ycenter + $radius).min(crate::game::world::chunk::Chunk::HEIGHT as isize) {
            foreach_in_radius!(($x, $z; $xcenter, $zcenter; $radius) {$body});
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

    player_distance_to_block: usize,

    threads: Vec<JoinHandle<()>>,
    texture_atlas: TextureAtlas,
    shader_program: Program,
}

impl World {
    //TODO:
    //This generation is temporary, need in future change to normal generation!
    pub fn new(seed: u32, blocksize: f32) -> Self {
        #[allow(unused_assignments)]
        let mut render_radius_in_chunks = 8;
        #[cfg(target_os = "macos")]
        {
            render_radius_in_chunks = 4;
        }
        let world = Self {
            seed,
            blocksize,

            render_center: (0, 0),
            render_radius_in_chunks,

            player_distance_to_block: 4,

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
        foreach_in_radius! {
            (x, z; 0, 0; radius) {
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

    pub fn destroy_block_if_possible(&mut self, player_position: &Vec3, view_ray: &Vec3) {
        if let Some(CoordinateInSpace {
            x,
            y,
            z,
            xoffset,
            zoffset,
            ..
        }) = self.find_nearest_block_in_ray(player_position, view_ray)
        {
            if let Some(chunk) = STORAGE.lock().chunk(xoffset, zoffset) {
                let mut chunk = chunk.lock();
                let block = chunk.mut_block_at(x, z, y);

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
            }
        }
    }

    pub fn place_block_if_possible(&mut self, player_position: &Vec3, view_ray: &Vec3) {
        if let Some(CoordinateInSpace {
            mut x,
            mut y,
            mut z,
            mut xoffset,
            mut zoffset,
            intersection_point,
        }) = self.find_nearest_block_in_ray(player_position, view_ray)
        {
            let block_pos = vec3(
                x as f32 + (xoffset * Chunk::WIDTH_ISIZE) as f32,
                y as f32,
                z as f32 + (zoffset * Chunk::WIDTH_ISIZE) as f32,
            );
            let diff = intersection_point - block_pos;
            macro_rules! shift_to_need_axis {
                ($diff:ident, $axis:ident, $axisoffset:ident, $max:expr) => {
                    if $diff.$axis == 1.0 {
                        if $axis + 1 == $max {
                            $axisoffset += 1;
                            $axis = 0;
                        } else {
                            $axis += 1;
                        }
                    } else if $diff.$axis == 0.0 {
                        if $axis == 0 {
                            $axisoffset -= 1;
                            $axis = $max - 1;
                        } else {
                            $axis -= 1;
                        }
                    }
                };
                ($diff:ident, $axis:ident, $max:expr) => {
                    let mut _stub = 0;
                    shift_to_need_axis!($diff, $axis, _stub, $max);
                };
            }
            shift_to_need_axis!(diff, x, xoffset, Chunk::WIDTH);
            shift_to_need_axis!(diff, z, zoffset, Chunk::WIDTH);
            shift_to_need_axis!(diff, y, Chunk::HEIGHT);

            // TODO:
            // Need to define normal collision of player and block
            //
            // let dist = vec3(
            //     ((x as f32 + (xoffset * Chunk::WIDTH_ISIZE) as f32) - player_position.x)
            //         .abs(),
            //     (y as f32) - player_position.y.floor(),
            //     ((z as f32 + (zoffset * Chunk::WIDTH_ISIZE) as f32) - player_position.z)
            //         .abs(),
            // );
            // if (dist.x / self.blocksize < self.blocksize && dist.z / self.blocksize < self.blocksize)
            //     && ((dist.y < 0.0 && dist.y.abs() / (self.blocksize * 2.0) < 1.0)
            //         || (dist.y >= 0.0 && dist.y / self.blocksize < 1.0))
            // {
            //     return;
            // }

            if let Some(chunk) = STORAGE.lock().chunk(xoffset, zoffset) {
                let mut chunk = chunk.lock();
                let block = chunk.mut_block_at(x, z, y);

                if !Block::is_air(*block) {
                    return;
                }

                *block = 1;
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
            }
        }
    }

    fn find_nearest_block_in_ray(
        &self,
        player_position: &Vec3,
        view_ray: &Vec3,
    ) -> Option<CoordinateInSpace> {
        let xcenter = player_position.x.floor() as isize;
        let ycenter = player_position.y.floor() as isize;
        let zcenter = player_position.z.floor() as isize;
        let radius = self.player_distance_to_block as isize;

        let mut coord: Option<CoordinateInSpace> = None;
        let mut distance = f32::MAX;
        foreach_in_radius! {
            (x, y, z; xcenter, ycenter, zcenter; radius) {
                let (tmin, tmax) = self.get_t_values_ray_intersection(x, y, z, player_position, view_ray);
                if tmax < 0.0 || tmin > tmax {
                    continue;
                }

                let mut block_position = vec3(x as f32, y as f32, z as f32);
                get_block_position!((xblock, yblock, zblock, xoffset, zoffset) <= block_position);
                if let Some(chunk) = STORAGE.lock().chunk(xoffset, zoffset) {
                    let chunk = chunk.lock();
                    let block = chunk.block_at(xblock, zblock, yblock);
                    if Block::is_air(block) {
                        continue;
                    }

                    let intersection_point = player_position + view_ray * tmin;
                    let dist_to_block = player_position.metric_distance(&intersection_point);
                    if dist_to_block < distance {
                        distance = dist_to_block;
                        coord = Some(CoordinateInSpace{
                            x: xblock, y: yblock, z: zblock, xoffset, zoffset, intersection_point}
                            );
                    }
                }
            }
        };
        coord
    }

    fn get_t_values_ray_intersection(
        &self,
        x: isize,
        y: isize,
        z: isize,
        origin: &Vec3,
        direction: &Vec3,
    ) -> (f32, f32) {
        let block_start = vec3(x as f32, y as f32, z as f32);
        let block_end = block_start.add_scalar(self.blocksize);
        let mut min = block_start - origin;
        min.x /= direction.x;
        min.y /= direction.y;
        min.z /= direction.z;
        let mut max = block_end - origin;
        max.x /= direction.x;
        max.y /= direction.y;
        max.z /= direction.z;

        let tmin = min.x.min(max.x).max(min.y.min(max.y)).max(min.z.min(max.z));
        let tmax = min.x.max(max.x).min(min.y.max(max.y)).min(min.z.max(max.z));
        (tmin, tmax)
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

struct CoordinateInSpace {
    x: usize,
    y: usize,
    z: usize,
    xoffset: isize,
    zoffset: isize,
    intersection_point: Vec3,
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
