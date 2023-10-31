use super::block::*;
// use crate::game::world::Chunk;

use gl::types::*;
use nalgebra_glm::*;

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

pub struct ChunkMesh {
    mesh: Vec<BlockFace>,
    // chunk: &'a Chunk,
    blocksize: f32,
    shader_program: super::Program,
}


impl ChunkMesh {
    fn is_air(block: u64) -> bool {
        (block & 1) == 0
    }

    pub fn render(&self, shader_program: &super::Program) {
        for block_face in &self.mesh {
            shader_program.insert_mat4(&std::ffi::CString::new("model").unwrap(), &block_face.model);
            block_face.render();
        }
    }
}

pub struct VaoAttributes {
    pub position: GLuint,
    pub size: GLint,
    pub type_: GLenum,
    pub normalized: GLboolean,
    pub stride: GLsizei,
    pub pointer: *const GLvoid,
}

struct BlockFace {
    vbo: GLuint,
    vao: GLuint,
    ebo: GLuint,
    zoffset_texture: f32,
    model: Mat4,
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
    const MAPPING_VERTEX_INDICES: [usize; 6] = [0, 1, 2, 1, 2, 3];
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

    fn new(block: u64, offset: Vec3, pos: &RenderPosition) -> Self {
        let block_info = Block::from(Self::get_block_id(block));

        let mut data = vec![];
        let map_vert = Self::MAPPING_VERTICES[*pos as usize];
        map_vert.iter().enumerate().for_each(|(i, vert_index)| {
            data.extend_from_slice(&Self::CUBE_VERTICES[*vert_index]);
            data.extend_from_slice(&Self::NORM[*pos as usize]);
            data.extend_from_slice(&Self::TEXTURE_UV[i]);
        });

        // TODO:
        // Maybe in future need to change to Dynamic in specific cases
        let vbo = Self::create_vbo(data, gl::STATIC_DRAW);
        let ebo = Self::create_ebo(gl::STATIC_DRAW);
        let vao = Self::create_vao(vbo, ebo);
        let model = translate(&Mat4::identity(), &offset);

        Self {
            vbo,
            vao,
            ebo,
            zoffset_texture: block_info.zoffset_texure() as f32,
            model,
        }
    }

    fn get_block_id(block: u64) -> usize {
        let mut bit = 1;
        let mut result = 0;
        for _ in 0..=16 {
            if bit & block == 1 {
                result = result | bit;
            }
            bit <<= 1;
        }
        result as usize
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
                (Self::MAPPING_VERTEX_INDICES.len() * std::mem::size_of::<gl::types::GLuint>())
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
            if ebo != 0 {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            }

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
            gl::DrawElements(gl::TRIANGLES, 4, gl::FLOAT, 0 as *const GLvoid);
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
