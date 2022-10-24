use gl::types::*;
use log::error;
use nalgebra_glm as glm;

use std::{
    error::Error,
    ffi::{c_void, CString},
    fmt, ptr,
};

use glfw::{Action, Context, Key};

#[derive(Debug)]
pub enum OpenGlError {
    CreateWindowError,
    ShaderCompilationError { reason: String },
    ProgramCompilationError { reason: String },
}

impl Error for OpenGlError {}
impl fmt::Display for OpenGlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

struct Shader {
    name: GLuint,
}

impl Shader {
    fn load(t: GLenum, source: &str) -> Result<Self, Box<dyn Error>> {
        unsafe {
            let shader = gl::CreateShader(t);
            let c_str = CString::new(source.as_bytes())?;
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

            if status != (gl::TRUE as GLint) {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetShaderInfoLog(
                    shader,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );

                gl::DeleteShader(shader);

                return Err(Box::new(OpenGlError::ShaderCompilationError {
                    reason: std::str::from_utf8(&buf)?.to_owned(),
                }));
            }
            Ok(Self { name: shader })
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.name);
        }
    }
}

struct Program {
    pub name: GLuint,
}

impl Program {
    fn load(shaders: Vec<Shader>) -> Result<Self, Box<dyn Error>> {
        unsafe {
            let program = gl::CreateProgram();

            for shader in shaders {
                gl::AttachShader(program, shader.name);
            }

            gl::LinkProgram(program);
            // Get the link status
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

            // Fail on error
            if status != (gl::TRUE as GLint) {
                let mut len: GLint = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buf = Vec::with_capacity(len as usize);
                buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
                gl::DeleteProgram(program);
                return Err(Box::new(OpenGlError::ProgramCompilationError {
                    reason: std::str::from_utf8(&buf)?.to_owned(),
                }));
            }
            Ok(Self { name: program })
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.name);
        }
    }
}

const POS_ATTRIB: u32 = 0;
const COL_ATTRIB: u32 = 1;
const VTX_BINDING_POS: u32 = 0;
const VTX_BINDING_COL: u32 = 1;

type Point = (f32, f32, f32);

struct VertexData {
    vao_name: GLuint,
    pos_buffer_name: GLuint,
    col_buffer_name: GLuint,
}

impl VertexData {
    fn create() -> Self {
        unsafe {
            let mut vao: GLuint = 0;
            let mut pos_buffer: GLuint = 0;
            let mut col_buffer: GLuint = 0;
            gl::CreateVertexArrays(1, &mut vao);
            gl::CreateBuffers(1, &mut pos_buffer);
            gl::CreateBuffers(1, &mut col_buffer);
            gl::EnableVertexArrayAttrib(vao, POS_ATTRIB);
            gl::EnableVertexArrayAttrib(vao, COL_ATTRIB);
            gl::VertexArrayAttribFormat(vao, POS_ATTRIB, 3, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayAttribFormat(vao, COL_ATTRIB, 3, gl::FLOAT, gl::FALSE, 0);
            gl::VertexArrayVertexBuffer(
                vao,
                VTX_BINDING_POS,
                pos_buffer,
                0,
                std::mem::size_of::<glm::Vec3>() as i32,
            );
            gl::VertexArrayVertexBuffer(
                vao,
                VTX_BINDING_COL,
                col_buffer,
                0,
                std::mem::size_of::<glm::Vec3>() as i32,
            );
            gl::VertexArrayAttribBinding(vao, POS_ATTRIB, VTX_BINDING_POS);
            gl::VertexArrayAttribBinding(vao, COL_ATTRIB, VTX_BINDING_COL);
            Self {
                vao_name: vao,
                pos_buffer_name: pos_buffer,
                col_buffer_name: col_buffer,
            }
        }
    }

    fn load_buffer(&self, points: &Vec<glm::Vec3>) {
        unsafe {
            gl::NamedBufferData(
                self.pos_buffer_name,
                (std::mem::size_of::<glm::Vec3>() * points.len()) as isize,
                points.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::NamedBufferData(
                self.col_buffer_name,
                (std::mem::size_of::<glm::Vec3>() * points.len()) as isize,
                ptr::null(),
                gl::DYNAMIC_DRAW,
            );
        }
    }

    fn update_col_buffer(&self, colors: &Vec<glm::Vec3>) {
        unsafe {
            gl::NamedBufferSubData(
                self.col_buffer_name,
                0,
                (std::mem::size_of::<glm::Vec3>() * colors.len()) as isize,
                colors.as_ptr() as *const c_void,
            );
        }
    }
}

impl Drop for VertexData {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.pos_buffer_name);
            gl::DeleteBuffers(1, &self.col_buffer_name);
            gl::DeleteVertexArrays(1, &self.vao_name);
        }
    }
}

extern "system" fn error_callback(
    _source: GLenum,
    _gltype: GLenum,
    _id: GLuint,
    _severity: GLenum,
    _length: GLsizei,
    message: *const GLchar,
    _user_param: *mut c_void,
) {
    unsafe {
        let message = std::ffi::CStr::from_ptr(message);
        error!(target: "OpenGL", "{}", message.to_str().unwrap());
    }
}

pub fn visualise(
    points: Vec<Point>,
    colors_recv: std::sync::mpsc::Receiver<Vec<(f32, f32, f32)>>,
) -> Result<(), Box<dyn Error>> {
    let points = points
        .into_iter()
        .map(|(x, y, z)| glm::vec3(x, y, z))
        .collect::<Vec<_>>();

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS)?;

    glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGl));
    glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

    let (mut window, events) = glfw
        .create_window(2560, 1440, "Rustmas Visualiser", glfw::WindowMode::Windowed)
        .ok_or(OpenGlError::CreateWindowError)?;

    window.set_key_polling(true);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s) as *const _);

    unsafe {
        gl::DebugMessageCallback(Some(error_callback), ptr::null());
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
    }

    let vs = Shader::load(gl::VERTEX_SHADER, include_str!("shaders/shader.vert"))?;
    let fs = Shader::load(gl::FRAGMENT_SHADER, include_str!("shaders/shader.frag"))?;
    let program = Program::load(vec![vs, fs])?;

    let vdata = VertexData::create();
    vdata.load_buffer(&points);

    let projection_matrix = glm::perspective(1024.0 / 768.0, 45.0_f32.to_radians(), 0.1, 100.0);
    let mut model_matrix = glm::identity::<_, 4>();

    let mvp_location = unsafe {
        gl::BindVertexArray(vdata.vao_name);
        gl::UseProgram(program.name);
        gl::PointSize(30.0);

        let c_str = CString::new("mvp".as_bytes())?;
        gl::GetUniformLocation(program.name, c_str.as_ptr())
    };

    let camera_pos = glm::vec3(2.0, 0.0, 0.0);

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true)
                }
                glfw::WindowEvent::Key(Key::W, _, Action::Press, _) => {
                    model_matrix = glm::scale(&model_matrix, &glm::vec3(1.1, 1.1, 1.1));
                }
                glfw::WindowEvent::Key(Key::S, _, Action::Press, _) => {
                    model_matrix = glm::scale(&model_matrix, &glm::vec3(0.9, 0.9, 0.9));
                }
                glfw::WindowEvent::Key(Key::A, _, Action::Press, _) => {
                    model_matrix = glm::rotate(
                        &model_matrix,
                        -10.0_f32.to_radians(),
                        &glm::vec3(0.0, 1.0, 0.0),
                    );
                }
                glfw::WindowEvent::Key(Key::D, _, Action::Press, _) => {
                    model_matrix = glm::rotate(
                        &model_matrix,
                        10.0_f32.to_radians(),
                        &glm::vec3(0.0, 1.0, 0.0),
                    );
                }
                _ => {}
            }
        }

        let view_matrix = glm::look_at(
            &camera_pos,
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );

        let mvp = projection_matrix * view_matrix * model_matrix;

        match colors_recv.try_recv() {
            Ok(new_colors) => vdata.update_col_buffer(
                &new_colors
                    .into_iter()
                    .map(|(x, y, z)| glm::vec3(x, y, z))
                    .collect(),
            ),
            _ => (),
        };

        unsafe {
            gl::Flush();
            gl::ClearColor(0.3, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UniformMatrix4fv(mvp_location, 1, gl::FALSE, mvp.as_ptr());

            gl::DrawArrays(gl::POINTS, 0, points.len() as i32);
        }

        window.swap_buffers();
    }
    Ok(())
}
