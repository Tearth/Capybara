use super::context::RendererContext;
use crate::utils::storage::StorageItem;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Error;
use anyhow::Result;
use glow::Context;
use glow::HasContext;
use glow::Program;
use glow::UniformLocation;
use std::collections::HashMap;
use std::rc::Rc;
use std::slice;

pub const SPRITE_VERTEX_SHADER: &str = include_str!("./shaders/sprite.vert");
pub const SPRITE_FRAGMENT_SHADER: &str = include_str!("./shaders/sprite.frag");

pub const SHAPE_VERTEX_SHADER: &str = include_str!("./shaders/shape.vert");
pub const SHAPE_FRAGMENT_SHADER: &str = include_str!("./shaders/shape.frag");

pub struct Shader {
    pub id: usize,
    pub name: Option<String>,
    pub program: Program,
    pub uniforms: HashMap<String, ShaderParameter>,

    gl: Rc<Context>,
}

pub struct ShaderParameter {
    pub location: UniformLocation,
    pub r#type: u32,
}

impl Shader {
    pub fn new(renderer: &RendererContext, vertex_shader_source: &str, fragment_shader_source: &str) -> Result<Self> {
        unsafe {
            let gl = renderer.gl.clone();

            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).map_err(Error::msg)?;
            gl.shader_source(vertex_shader, preprocess_shader_source(vertex_shader_source).as_str());
            gl.compile_shader(vertex_shader);

            if !gl.get_shader_compile_status(vertex_shader) {
                bail!("Vertex shader: {}", gl.get_shader_info_log(vertex_shader));
            }

            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).map_err(Error::msg)?;
            gl.shader_source(fragment_shader, preprocess_shader_source(fragment_shader_source).as_str());
            gl.compile_shader(fragment_shader);

            if !gl.get_shader_compile_status(fragment_shader) {
                bail!("Fragment shader: {}", gl.get_shader_info_log(fragment_shader));
            }

            let program = gl.create_program().map_err(Error::msg)?;
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                bail!("Program: {}", gl.get_program_info_log(program));
            }

            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            let active_uniforms = gl.get_active_uniforms(program);
            let mut uniforms: HashMap<String, ShaderParameter> = Default::default();

            for index in 0..active_uniforms {
                let uniform = gl.get_active_uniform(program, index).ok_or_else(|| anyhow!("Uniform not found"))?;

                if uniform.size == 1 {
                    let location = gl.get_uniform_location(program, &uniform.name).ok_or_else(|| anyhow!("Uniform location not found"))?;
                    uniforms.insert(uniform.name, ShaderParameter::new(location, uniform.utype));
                } else {
                    for array_index in 0..uniform.size {
                        let name_with_index = uniform.name.replace("[0]", &format!("[{}]", array_index));
                        let location = gl.get_uniform_location(program, &name_with_index).ok_or_else(|| anyhow!("Uniform location not found"))?;
                        uniforms.insert(name_with_index, ShaderParameter::new(location, uniform.utype));
                    }
                }
            }

            Ok(Shader { id: 0, name: None, program, uniforms, gl })
        }
    }

    pub fn set_uniform<T>(&self, name: &str, data: *const T) -> Result<()>
    where
        T: Copy + Into<f32>,
    {
        unsafe {
            let parameter = match self.uniforms.get(name) {
                Some(parameter) => parameter,
                None => return Ok(()),
            };

            match parameter.r#type {
                glow::INT => {
                    (self.gl.uniform_1_i32(Some(&parameter.location), (*data).into() as i32));
                }
                glow::FLOAT => {
                    (self.gl.uniform_1_f32(Some(&parameter.location), (*data).into()));
                }
                glow::FLOAT_VEC2 => {
                    let slice = slice::from_raw_parts::<f32>(data as *const f32, 2);
                    (self.gl.uniform_2_f32_slice(Some(&parameter.location), slice));
                }
                glow::FLOAT_VEC4 => {
                    let slice = slice::from_raw_parts::<f32>(data as *const f32, 4);
                    (self.gl.uniform_4_f32_slice(Some(&parameter.location), slice));
                }
                glow::FLOAT_MAT4 => {
                    let slice = slice::from_raw_parts::<f32>(data as *const f32, 16);
                    (self.gl.uniform_matrix_4_f32_slice(Some(&parameter.location), false, slice));
                }
                _ => bail!("Invalid shader parameter type".to_string()),
            };

            Ok(())
        }
    }

    pub fn activate(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }
}

impl StorageItem for Shader {
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id;
    }

    fn get_name(&self) -> Option<String> {
        self.name.clone()
    }

    fn set_name(&mut self, name: Option<String>) {
        self.name = name;
    }
}

impl ShaderParameter {
    pub fn new(location: UniformLocation, r#type: u32) -> Self {
        Self { location, r#type }
    }
}

fn preprocess_shader_source(source: &str) -> String {
    #[cfg(any(windows, unix))]
    let version = "330 core";

    #[cfg(web)]
    let version = "300 es";

    source.replace("<version>", version)
}
