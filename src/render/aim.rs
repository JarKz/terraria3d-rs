use gl::types::*;
use nalgebra_glm::{scale, vec3, Mat4, Vec3};

use super::{Program, Shader, VaoAttributes};

pub struct Aim {
    vao: GLuint,
    vbo: GLuint,
    model: Mat4,
    program: Program,
}

impl Aim {
    const VERTICES: [[f32; 3]; 4] = [
        [1., 0.0, 0.0],
        [-1., 0.0, 0.0],
        [0.0, 1., 0.0],
        [0.0, -1., 0.0],
    ];

    pub fn new(width: f32, height: f32, color: Vec3) -> Aim {
        let program = Self::initialize_shaders();
        let vbo = Self::create_vbo(width / height, color);
        let vao = Self::create_vao(vbo);
        Self {
            program,
            vbo,
            vao,
            model: scale(&Mat4::identity(), &vec3(0.02, 0.02, 1.)),
        }
    }

    fn initialize_shaders() -> Program {
        Program::from([
            Shader::from_vertex(String::from("res/shaders/aim-vert.glsl")).unwrap(),
            Shader::from_fragment(String::from("res/shaders/aim-frag.glsl")).unwrap(),
        ])
    }

    fn create_vbo(aspect_ratio: f32, color: Vec3) -> GLuint {
        let mut vertices = vec![];
        let color_array = [color.x, color.y, color.z];
        for v in Self::VERTICES.clone().iter_mut() {
            v[0] /= aspect_ratio;
            vertices.extend_from_slice(v);
            vertices.extend_from_slice(&color_array);
        }

        let mut vbo = 0;
        unsafe {
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const GLvoid,
                gl::STATIC_DRAW,
            );
        }
        vbo
    }

    const STRIDE: GLint = (6 * std::mem::size_of::<f32>()) as GLint;
    const STANDARD_VAO_ATTRIBS: [VaoAttributes; 2] = [
        VaoAttributes {
            position: 0,
            size: 3,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: Self::STRIDE,
            pointer: std::ptr::null(),
        },
        VaoAttributes {
            position: 1,
            size: 3,
            type_: gl::FLOAT,
            normalized: gl::FALSE,
            stride: Self::STRIDE,
            pointer: (3 * std::mem::size_of::<f32>()) as *const GLvoid,
        },
    ];

    fn create_vao(vbo: GLuint) -> GLuint {
        let mut vao = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

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
        }
        vao
    }

    pub fn render(&self) {
        self.program.set_used();
        self.program
            .insert_mat4(&std::ffi::CString::new("model").unwrap(), &self.model);
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::LINES, 0, 4);
        }
    }
}
