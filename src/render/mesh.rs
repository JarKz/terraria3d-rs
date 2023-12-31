use super::{block::*, *};
use crate::game::world::{
    chunk::{Chunk, ChunkBlocks},
    WorldHelper,
};

use nalgebra_glm::*;
use once_cell::sync::Lazy;

use crate::foreach_block;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderPosition {
    /// Negative Z
    NORTH = 0,
    /// Positive Z
    SOUTH = 1,
    /// Negative X
    WEST = 2,
    /// Positive X
    EAST = 3,
    /// Positive Y
    TOP = 4,
    /// Negative Y
    BOTTOM = 5,
}

#[derive(Clone)]
pub struct ChunkMesh {
    mesh: Vec<BlockMesh>,
    blocksize: f32,
}

impl ChunkMesh {
    pub fn new(blocks: &ChunkBlocks, xoffset: f32, zoffset: f32, blocksize: f32) -> Self {
        let mut mesh = vec![];

        let mut offset = vec3(xoffset, 0.0, zoffset);
        foreach_block! {
            (y, offset = vec3(xoffset, offset.y + blocksize, zoffset);
            x, offset = vec3(offset.x + blocksize, offset.y, zoffset);
            z, offset.z += blocksize) {
                let block = blocks[y][x][z];
                if Block::is_air(block) {
                    offset.z += blocksize;
                    continue;
                }

                let mut positions = vec![];
                if x == 0 {
                    let b = WorldHelper::get_block_at(
                        &vec3(xoffset - blocksize, offset.y, offset.z),
                        blocksize,
                    );
                    if Block::is_air(b) {
                        positions.push(RenderPosition::WEST);
                    }
                } else if Block::is_air(blocks[y][x - 1][z]) {
                    positions.push(RenderPosition::WEST);
                }
                if x + 1 == Chunk::WIDTH {
                    let b = WorldHelper::get_block_at(
                        &vec3(offset.x + blocksize, offset.y, offset.z),
                        blocksize,
                    );
                    if Block::is_air(b) {
                        positions.push(RenderPosition::EAST);
                    }
                } else if Block::is_air(blocks[y][x + 1][z]) {
                    positions.push(RenderPosition::EAST);
                }

                if z == 0 {
                    let b = WorldHelper::get_block_at(
                        &vec3(offset.x, offset.y, zoffset - blocksize),
                        blocksize,
                    );
                    if Block::is_air(b) {
                        positions.push(RenderPosition::NORTH);
                    }
                } else if Block::is_air(blocks[y][x][z - 1]) {
                    positions.push(RenderPosition::NORTH);
                }
                if z + 1 == Chunk::WIDTH {
                    let b = WorldHelper::get_block_at(
                        &vec3(offset.x, offset.y, offset.z + blocksize),
                        blocksize,
                    );
                    if Block::is_air(b) {
                        positions.push(RenderPosition::SOUTH);
                    }
                } else if Block::is_air(blocks[y][x][z + 1]) {
                    positions.push(RenderPosition::SOUTH);
                }

                // TODO: It's temporary for increasing performance! In future must be valid logic!
                if y > 0 && Block::is_air(blocks[y - 1][x][z]) {
                    positions.push(RenderPosition::BOTTOM);
                }
                if y + 1 == Chunk::HEIGHT || Block::is_air(blocks[y + 1][x][z]) {
                    positions.push(RenderPosition::TOP);
                }

                if positions.len() > 0 {
                    mesh.push(BlockMesh::new(block, &offset, positions));
                }
        }};

        Self { mesh, blocksize }
    }

    pub fn render(&self, shader_program: &super::Program) {
        for block_mesh in &self.mesh {
            shader_program
                .insert_mat4(&std::ffi::CString::new("model").unwrap(), &block_mesh.model);
            shader_program.insert_float(
                &std::ffi::CString::new("texture_offset").unwrap(),
                block_mesh.zoffset_texture,
            );
            for face in &block_mesh.data {
                BLOCK_RENDERER.render(*face);
            }
        }
    }
}

struct BlockMesh {
    model: Mat4,
    data: Vec<usize>,
    zoffset_texture: f32,
}

impl BlockMesh {
    fn new(block: u64, offset: &Vec3, postitions: Vec<RenderPosition>) -> Self {
        let block_info = Block::from(Self::get_block_id(block));
        let data = postitions
            .iter()
            .map(|p| *p as usize)
            .collect::<Vec<usize>>();
        let model = translate(&Mat4::identity(), offset);
        BlockMesh {
            model,
            data,
            zoffset_texture: block_info.zoffset_texure() as f32,
        }
    }

    fn get_block_id(block: u64) -> usize {
        let mut bit = 1;
        let mut result = 0;
        for _ in 0..16 {
            if bit & block == bit {
                result |= bit;
            }
            bit <<= 1;
        }
        result as usize
    }
}

impl Clone for BlockMesh {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            data: self.data.clone(),
            zoffset_texture: self.zoffset_texture,
        }
    }
}

static BLOCK_RENDERER: Lazy<BlockRenderer> = Lazy::new(|| BlockRenderer::init());

pub struct BlockRenderer {
    // Corresponding to RenderPosition struct
    faces: [BlockFace; 6],
}

impl BlockRenderer {
    pub fn init() -> Self {
        let mut faces = [
            BlockFace {
                vao: 0,
                vbo: 0,
                ebo: 0,
            },
            BlockFace {
                vao: 0,
                vbo: 0,
                ebo: 0,
            },
            BlockFace {
                vao: 0,
                vbo: 0,
                ebo: 0,
            },
            BlockFace {
                vao: 0,
                vbo: 0,
                ebo: 0,
            },
            BlockFace {
                vao: 0,
                vbo: 0,
                ebo: 0,
            },
            BlockFace {
                vao: 0,
                vbo: 0,
                ebo: 0,
            },
        ];
        for (i, face) in faces.iter_mut().enumerate() {
            *face = BlockFace::new(i);
        }
        Self { faces }
    }

    fn render(&self, pos: usize) {
        self.faces[pos].render();
    }
}

#[derive(Clone)]
pub struct BlockFace {
    vbo: GLuint,
    vao: GLuint,
    ebo: GLuint,
}

impl BlockFace {
    const CUBE_VERTICES: [[f32; 3]; 8] = [
        [0., 0., 0.],
        [0., 1., 0.],
        [1., 0., 0.],
        [1., 1., 0.],
        [0., 0., 1.],
        [0., 1., 1.],
        [1., 0., 1.],
        [1., 1., 1.],
    ];

    /// NORTH  - 0
    /// SOUTH  - 1
    /// WEST   - 2
    /// EAST   - 3
    /// TOP    - 4
    /// BOTTOM - 5
    const MAPPING_VERTICES: [[usize; 4]; 6] = [
        [2, 3, 0, 1],
        [4, 5, 6, 7],
        [0, 1, 4, 5],
        [6, 7, 2, 3],
        [5, 1, 7, 3],
        [0, 4, 2, 6],
    ];

    /// NORTH  - 0
    /// SOUTH  - 1
    /// WEST   - 2
    /// EAST   - 3
    /// TOP    - 4
    /// BOTTOM - 5
    const NORM: [[f32; 3]; 6] = [
        [0., 0., -1.],
        [0., 0., 1.],
        [-1., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
        [0., -1., 0.],
    ];

    /// For EBO
    const MAPPING_VERTEX_INDICES: [GLuint; 6] = [0, 1, 2, 1, 2, 3];
    const TEXTURE_UV: [[f32; 2]; 4] = [[0., 0.], [0., 1.], [1., 0.], [1., 1.]];

    const STRIDE: usize = 8;
    const STANDARD_VAO_ATTRIBS: [VaoAttributes; 3] = [
        VaoAttributes {
            position: 0,
            size: 3,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: (Self::STRIDE * std::mem::size_of::<f32>()) as GLint,
            pointer: std::ptr::null(),
        },
        VaoAttributes {
            position: 1,
            size: 3,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: (Self::STRIDE * std::mem::size_of::<f32>()) as GLint,
            pointer: (3 * std::mem::size_of::<f32>()) as *const GLvoid,
        },
        VaoAttributes {
            position: 2,
            size: 2,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: (Self::STRIDE * std::mem::size_of::<f32>()) as GLint,
            pointer: (6 * std::mem::size_of::<f32>()) as *const GLvoid,
        },
    ];

    fn new(pos: usize) -> Self {
        let mut data = vec![];
        let map_vert = Self::MAPPING_VERTICES[pos];
        map_vert.iter().enumerate().for_each(|(i, vert_index)| {
            data.extend_from_slice(&Self::CUBE_VERTICES[*vert_index]);
            data.extend_from_slice(&Self::NORM[pos]);
            data.extend_from_slice(&Self::TEXTURE_UV[i]);
        });

        // TODO:
        // Maybe in future need to change to Dynamic in specific cases
        let vbo = Self::create_vbo(data, gl::STATIC_DRAW);
        let ebo = Self::create_ebo(gl::STATIC_DRAW);
        let vao = Self::create_vao(vbo, ebo);

        Self { vbo, vao, ebo }
    }

    fn create_vbo(data: Vec<f32>, usage: GLenum) -> GLuint {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);

            gl::BindBuffer(gl::ARRAY_BUFFER, id);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                data.as_ptr() as *const gl::types::GLvoid,
                usage,
            );
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        id
    }

    fn create_ebo(usage: GLenum) -> GLuint {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (Self::MAPPING_VERTEX_INDICES.len() * std::mem::size_of::<GLuint>())
                    as gl::types::GLsizeiptr,
                Self::MAPPING_VERTEX_INDICES.as_ptr() as *const gl::types::GLvoid,
                usage,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
        id
    }

    fn create_vao(vbo: GLuint, ebo: GLuint) -> GLuint {
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

            for attrs in Self::STANDARD_VAO_ATTRIBS {
                gl::VertexAttribPointer(
                    attrs.position,
                    attrs.size,
                    attrs.type_,
                    attrs.normalized,
                    attrs.stride,
                    attrs.pointer,
                );
                gl::EnableVertexAttribArray(attrs.position);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
        vao
    }

    fn render(&self) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, 0 as *const GLvoid);
        }
    }
}

impl Drop for BlockFace {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &mut self.vao);
            gl::DeleteBuffers(1, &mut self.vbo);
            gl::DeleteBuffers(1, &mut self.ebo);
        }
    }
}
