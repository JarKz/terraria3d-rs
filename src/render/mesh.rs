use super::block::*;
use crate::game::world::Chunk;

use gl::types::*;
use nalgebra_glm::*;

#[derive(Clone, Copy)]
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

pub struct ChunkMesh<'a> {
    mesh: Vec<BlockFace>,
    chunk: &'a Chunk,
    blocksize: f32,
}

impl ChunkMesh<'_> {
    pub fn update_vertices(&mut self, positions: [RenderPosition; 2], player_y: f32) {
        let blocks = self.chunk.blocks();
        for y in 0..Chunk::HEIGHT {
            self.update_floor(y, &blocks[y], &positions, player_y);
        }
    }

    fn update_floor(
        &mut self,
        y: usize,
        floor: &[[u64; Chunk::WIDTH]; Chunk::WIDTH],
        positions: &[RenderPosition; 2],
        player_y: f32,
    ) {
        for pos in positions {
            match pos {
                RenderPosition::NORTH => {
                    let z = 0;
                    for x in 0..Chunk::WIDTH {
                        self.update_block_faces(x, y, z, floor, positions, player_y)
                    }
                }
                RenderPosition::SOUTH => {
                    let z = Chunk::WIDTH;
                    for x in 0..Chunk::WIDTH {
                        self.update_block_faces(x, y, z, floor, positions, player_y)
                    }
                }
                RenderPosition::WEST => {
                    let x = 0;
                    for z in 0..Chunk::WIDTH {
                        self.update_block_faces(x, y, z, floor, positions, player_y)
                    }
                }
                RenderPosition::EAST => {
                    let x = Chunk::WIDTH;
                    for z in 0..Chunk::WIDTH {
                        self.update_block_faces(x, y, z, floor, positions, player_y)
                    }
                }
                _ => (),
            }
        }
    }

    fn update_block_faces(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        floor: &[[u64; Chunk::WIDTH]; Chunk::WIDTH],
        positions: &[RenderPosition; 2],
        player_y: f32,
    ) {
        if Self::is_air(floor[x][z]) {
            for pos in positions {
                match pos {
                    RenderPosition::NORTH => {
                        if z != 0 {
                            self.update_block_faces(x, y, z - 1, floor, positions, player_y)
                        }
                    }
                    RenderPosition::SOUTH => {
                        if z != Chunk::WIDTH {
                            self.update_block_faces(x, y, z + 1, floor, positions, player_y)
                        }
                    }
                    RenderPosition::WEST => {
                        if x != 0 {
                            self.update_block_faces(x - 1, y, z, floor, positions, player_y)
                        }
                    }
                    RenderPosition::EAST => {
                        if x != Chunk::WIDTH {
                            self.update_block_faces(x + 1, y, z, floor, positions, player_y)
                        }
                    }
                    _ => (),
                }
            }
        } else {
            let xoffset = self.chunk.xoffset();
            let zoffset = self.chunk.zoffset();
            for pos in positions {
                self.mesh.push(BlockFace::new(
                    floor[x][z],
                    xoffset,
                    zoffset,
                    y as f32,
                    self.blocksize,
                    pos,
                ));
            }
            let real_player_y = player_y / self.blocksize;
            if real_player_y < y as f32
                && (y == 0 || !Self::is_air(self.chunk.blocks()[y - 1][x][z]))
            {
                self.mesh.push(BlockFace::new(
                    floor[x][z],
                    xoffset,
                    zoffset,
                    y as f32,
                    self.blocksize,
                    &RenderPosition::BOTTOM,
                ));
            }
            if real_player_y > y as f32
                && (y == Chunk::HEIGHT || !Self::is_air(self.chunk.blocks()[y + 1][x][z]))
            {
                self.mesh.push(BlockFace::new(
                    floor[x][z],
                    xoffset,
                    zoffset,
                    y as f32,
                    self.blocksize,
                    &RenderPosition::TOP,
                ));
            }
        }
    }

    fn is_air(block: u64) -> bool {
        (block & 1) == 0
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

    const STANDARD_VAO_ATTRIBS: [VaoAttributes; 3] = [
        VaoAttributes {
            position: 0,
            size: 3,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: (8 * std::mem::size_of::<f32>()) as GLint,
            pointer: std::ptr::null(),
        },
        VaoAttributes {
            position: 1,
            size: 3,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: (8 * std::mem::size_of::<f32>()) as GLint,
            pointer: (3 * std::mem::size_of::<f32>()) as *const GLvoid,
        },
        VaoAttributes {
            position: 2,
            size: 2,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: (8 * std::mem::size_of::<f32>()) as GLint,
            pointer: (6 * std::mem::size_of::<f32>()) as *const GLvoid,
        },
    ];

    fn new(
        block: u64,
        xoffset: f32,
        zoffset: f32,
        yoffset: f32,
        blocksize: f32,
        pos: &RenderPosition,
    ) -> Self {
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
        let model = translate(
            &Mat4::identity(),
            &(vec3(xoffset, yoffset, zoffset) * blocksize),
        );

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
