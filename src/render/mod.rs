#![allow(dead_code)]

use gl::types::*;
use lodepng::Bitmap;
use rgb::*;
use std::ffi::CString;

pub mod block;
pub mod mesh;

pub struct TextureAtlasConfiguration {
    pub image_path: String,
    pub square_size: usize,
}

type ImgTiles = Vec<Vec<Vec<u8>>>;

pub struct TextureAtlas {
    id: GLuint,
}

impl TextureAtlas {
    const BYTE_SIZE: usize = 4;
    fn get_atlas_mesh_from_image(image: Bitmap<RGBA<u8>>, square_size: usize) -> ImgTiles {
        if image.width == 0 || image.height == 0 {
            panic!("Image width or height must be greater than zero!");
        }
        if image.width % square_size != 0 || image.height % square_size != 0 {
            panic!("Invalid image size!");
        }

        let tiles_rows = image.height / square_size;
        let tiles_columns = image.width / square_size;
        let offset = Self::BYTE_SIZE * square_size * tiles_columns;

        let mut images: Vec<Vec<Vec<u8>>> = vec![vec![vec![]; tiles_columns]; tiles_rows];
        let bytes = image.buffer.as_bytes();

        let row_size = square_size * Self::BYTE_SIZE;
        for i in 0..tiles_rows {
            for j in 0..tiles_columns {
                let mut tile_offset = i * square_size * offset + j * row_size;
                for _ in 0..16 {
                    images[i][j].extend_from_slice(&bytes[tile_offset..(tile_offset + row_size)]);
                    tile_offset += offset;
                }
            }
        }

        images
    }

    fn create_texture_image(id: &mut GLuint, images: ImgTiles, square_size: i32) {
        let image_count = (images.len() * images[0].len()) as i32;
        unsafe {
            gl::GenTextures(1, id);

            gl::BindTexture(gl::TEXTURE_2D_ARRAY, *id);
            gl::TexImage3D(
                gl::TEXTURE_2D_ARRAY,
                0,
                gl::RGBA8 as GLint,
                square_size,
                square_size,
                image_count,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null() as *const GLvoid,
            );
            for i in 0..images.len() {
                for j in 0..images[i].len() {
                    gl::TexSubImage3D(
                        gl::TEXTURE_2D_ARRAY,
                        0,
                        0,
                        0,
                        (i * images.len() + j) as GLsizei,
                        square_size,
                        square_size,
                        1,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        images[i][j].as_ptr() as *const GLvoid,
                    );
                }
            }
            gl::GenerateMipmap(gl::TEXTURE_2D_ARRAY);
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as GLint,
            );
            gl::TexParameteri(gl::TEXTURE_2D_ARRAY, gl::TEXTURE_MAX_LEVEL, 4);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
        }
    }

    pub fn set_used(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.id);
        }
    }
}

impl From<TextureAtlasConfiguration> for TextureAtlas {
    fn from(cfg: TextureAtlasConfiguration) -> Self {
        let image =
            lodepng::decode32_file(std::path::Path::new(&cfg.image_path)).expect("Image not found in resource path!");
        let images = Self::get_atlas_mesh_from_image(image, cfg.square_size);
        let mut id = 0;
        Self::create_texture_image(&mut id, images, cfg.square_size as i32);

        Self { id }
    }
}

impl Drop for TextureAtlas {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &mut self.id) }
    }
}

pub struct Program {
    id: GLuint,
}

impl Program {
    pub fn set_used(&self) {
        unsafe { gl::UseProgram(self.id) }
    }

    pub fn insert_mat4(&self, fieldname: &CString, matrix: &nalgebra_glm::Mat4) {
        unsafe {
            gl::Uniform4fv(
                gl::GetUniformLocation(self.id, fieldname.as_ptr() as *const GLchar),
                1,
                matrix.as_ptr(),
            );
        }
    }
}

impl From<[Shader; 2]> for Program {
    fn from(shaders: [Shader; 2]) -> Self {
        let id = unsafe { gl::CreateProgram() };

        shaders
            .iter()
            .for_each(|s| unsafe { gl::AttachShader(id, s.id) });

        unsafe { gl::LinkProgram(id) }
        let mut success = 0;
        unsafe { gl::GetProgramiv(id, gl::LINK_STATUS, &mut success) }

        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            }
            panic!("{}", error.to_string_lossy().into_owned());
        }

        shaders
            .iter()
            .for_each(|s| unsafe { gl::DetachShader(id, s.id) });

        Self { id }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id) }
    }
}

pub struct Shader {
    id: GLuint,
}

use std::io::{BufRead, BufReader};

impl Shader {
    pub fn from_vertex(path: String) -> Result<Self, String> {
        Self::from(gl::VERTEX_SHADER, path)
    }

    pub fn from_fragment(path: String) -> Result<Self, String> {
        Self::from(gl::FRAGMENT_SHADER, path)
    }

    pub fn from(kind: GLenum, path: String) -> Result<Self, String> {
        let id = unsafe { gl::CreateShader(kind) };
        let source = match Self::read_from_path(path) {
            Ok(s) => s,
            Err(e) => {
                return Err(
                    "Some error in reading shader file! The error is: ".to_string()
                        + &e.to_string(),
                )
            }
        };

        unsafe {
            gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
            gl::CompileShader(id);
        }

        let mut success: GLint = 0;
        unsafe {
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: GLint = 0;
            unsafe {
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
            }
            let error = create_whitespace_cstring_with_len(len as usize);
            unsafe {
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            }
            return Err(error.to_string_lossy().into_owned());
        }

        Ok(Shader { id })
    }

    fn read_from_path(path: String) -> std::io::Result<CString> {
        let path = std::path::Path::new(&path);
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        Ok(CString::new(
            reader.lines().reduce(|l, r| {
                Ok(l? + "\n" + &r?)
            }).unwrap()?,
        )?)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { gl::DeleteShader(self.id) }
    }
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    let mut buffer = Vec::with_capacity(len as usize + 1);
    buffer.extend([b' '].iter().cycle().take(len as usize));
    unsafe { CString::from_vec_unchecked(buffer) }
}
