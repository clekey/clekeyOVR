use gl::types::{GLenum, GLint, GLsizei, GLuint};
use pathfinder_color::ColorF;
use pathfinder_geometry::transform2d::Transform2F;

pub fn gl_clear(color: ColorF) {
    unsafe {
        gl::ClearColor(color.r(), color.g(), color.b(), color.a());
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

/// Base renderer for renderers that performs most thing in the fragment shader code
///
/// This renderer will generate rectangle with -1,-1 to 1,1 as v2f_uv.
struct ShaderRenderer {
    shader_program: GLuint,
    points_vao: GLuint,
    transform_uniform: GLint,
}

impl ShaderRenderer {
    const POINTS: usize = 6;

    fn new(fragment_shader: &str) -> ShaderRenderer {
        unsafe {
            // shader
            let vs = compile_shader(
                gl::VERTEX_SHADER,
                "#version 400\n\
                layout(location = 0) in vec2 in_pos;\n\
                uniform mat2x3 transform;\n\
                out vec2 v2f_uv;\n\
                void main() {\n\
                    v2f_uv = in_pos;\n\
                    gl_Position.xy = vec3(in_pos, 1) * transform;\n\
                    gl_Position.zw = vec2(0, 1);\n\
                }\n",
            );

            let fs = compile_shader(gl::FRAGMENT_SHADER, fragment_shader);
            let shader_program = link_shader(&[fs, vs]);
            let in_pos_attrib = gl::GetAttribLocation(shader_program, c"in_pos".as_ptr()) as GLuint;
            let transform_uniform = gl::GetUniformLocation(shader_program, c"transform".as_ptr());
            assert!(in_pos_attrib != -1 as GLint as GLuint, "in_pos not found");
            assert!(transform_uniform != -1, "transform not found");

            let mut points_vbo = 0;
            gl::GenBuffers(1, &mut points_vbo);

            type Point = [f32; 2]; // x, y, u, v
            let a = [-1., -1.];
            let b = [1., -1.];
            let c = [-1., 1.];
            let d = [1., 1.];
            let points: [Point; Self::POINTS] = [a, b, c, c, b, d];

            gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val::<[_]>(&points) as isize,
                points.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            let mut points_vao = 0;
            gl::GenVertexArrays(1, &mut points_vao);
            gl::BindVertexArray(points_vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
            gl::EnableVertexAttribArray(in_pos_attrib as _);
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<Point>() as _,
                std::ptr::without_provenance(0),
            );

            Self {
                shader_program,
                points_vao,
                transform_uniform,
            }
        }
    }

    /// Renders glyphs in specified color.
    fn draw(&self, transform: Transform2F, set_uniforms: impl FnOnce() -> ()) {
        unsafe {
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::UseProgram(self.shader_program);

            gl::BindVertexArray(self.points_vao);

            gl::UniformMatrix2x3fv(
                self.transform_uniform,
                1,
                gl::FALSE,
                [
                    transform.matrix.m11(),
                    transform.matrix.m12(),
                    transform.vector.x(),
                    transform.matrix.m21(),
                    transform.matrix.m22(),
                    transform.vector.y(),
                ]
                .as_ptr(),
            );

            set_uniforms();

            gl::DrawArrays(gl::TRIANGLES, 0, Self::POINTS as _);
            gl::Disable(gl::BLEND);
        }
    }
}

pub struct CircleRenderer {
    base: ShaderRenderer,
    color_uniform: GLint,
}

impl CircleRenderer {
    pub fn new() -> CircleRenderer {
        let base = ShaderRenderer::new(
            "#version 400\n\
                \n\
                in vec2 v2f_uv;\n\
                out vec4 out_color;\n\
                \n\
                uniform vec4 color;\n\
                \n\
                void main() {\n\
                    float r = v2f_uv.x * v2f_uv.x + v2f_uv.y * v2f_uv.y;\n\
                    float a = step(r, 1.0);\n\
                    out_color.rgb = color.rgb;\n\
                    out_color.a = a * color.a;\n\
                }\n",
        );
        unsafe {
            let color_uniform = gl::GetUniformLocation(base.shader_program, c"color".as_ptr());
            assert!(color_uniform != -1, "color not found");

            Self {
                base,
                color_uniform,
            }
        }
    }

    /// Renders glyphs in specified color.
    pub fn draw(&self, transform: Transform2F, color: ColorF) {
        self.base.draw(transform, || unsafe {
            gl::Uniform4f(
                self.color_uniform,
                color.r(),
                color.g(),
                color.b(),
                color.a(),
            );
        });
    }
}

pub struct BaseBackgroundRenderer {
    base: ShaderRenderer,
    center_color_uniform: GLint,
    bg_color_uniform: GLint,
    line_color_uniform: GLint,
}

impl BaseBackgroundRenderer {
    pub fn new() -> BaseBackgroundRenderer {
        let base = ShaderRenderer::new(
            "#version 400\n\
                const float PI = 3.1415926535897932384626433832795;\n\
                const float LINE_W = 0.04;\n\
                const float CENTER = 0.5;\n\
                \n\
                in vec2 v2f_uv;\n\
                out vec4 out_color;\n\
                \n\
                uniform vec3 center_color;\n\
                uniform vec3 bg_color;\n\
                uniform vec3 line_color;\n\
                \n\
                float d_from_line(float angle_deg) {\n\
                    float angle_rad = angle_deg * PI / 180;\n\
                    float a = sin(angle_rad);\n\
                    float b = cos(angle_rad);\n\
                    return abs(a * v2f_uv.x + b * v2f_uv.y);\n\
                }\n\
                \n\
                void main() {\n\
                    float rsq = v2f_uv.x * v2f_uv.x + v2f_uv.y * v2f_uv.y;\n\
                    \n\
                    float d1 = d_from_line(22.5 * 1);\n\
                    float d2 = d_from_line(22.5 * 3);\n\
                    float d3 = d_from_line(22.5 * -1);\n\
                    float d4 = d_from_line(22.5 * -3);\n\
                    \n\
                    float min_line_d = min(min(d1, d2), min(d3, d4));\n\
                    \n\
                    out_color.rgb = rsq < CENTER * CENTER ? center_color \
                            : min_line_d < (LINE_W / 2) || (rsq > ((1 - LINE_W) * (1 - LINE_W))) ? line_color \
                            : bg_color;\n\
                    \n\
                    out_color.a = step(rsq, 1.0);\n\
                }\n\
                \n\
                ",
        );
        unsafe {
            let center_color_uniform =
                gl::GetUniformLocation(base.shader_program, c"center_color".as_ptr());
            let bg_color_uniform =
                gl::GetUniformLocation(base.shader_program, c"bg_color".as_ptr());
            let line_color_uniform =
                gl::GetUniformLocation(base.shader_program, c"line_color".as_ptr());
            assert!(bg_color_uniform != -1, "bg_color not found");
            assert!(line_color_uniform != -1, "line_color not found");

            Self {
                base,
                center_color_uniform,
                bg_color_uniform,
                line_color_uniform,
            }
        }
    }

    /// Renders glyphs in specified color.
    pub fn draw(
        &self,
        transform: Transform2F,
        center_color: ColorF,
        bg_color: ColorF,
        line_color: ColorF,
    ) {
        self.base.draw(transform, || unsafe {
            gl::Uniform3f(
                self.center_color_uniform,
                center_color.r(),
                center_color.g(),
                center_color.b(),
            );

            gl::Uniform3f(
                self.bg_color_uniform,
                bg_color.r(),
                bg_color.g(),
                bg_color.b(),
            );

            gl::Uniform3f(
                self.line_color_uniform,
                line_color.r(),
                line_color.g(),
                line_color.b(),
            );
        });
    }
}

pub unsafe fn compile_shader(type_: GLenum, script: &str) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(type_);
        gl::ShaderSource(
            shader,
            1,
            &script.as_ptr().cast::<i8>(),
            &(script.len() as GLint),
        );
        gl::CompileShader(shader);

        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut info = Vec::<u8>::new();
            let mut len: GLsizei = 512;
            while info.capacity() < (len as usize) {
                info.reserve(len as usize);
                gl::GetShaderInfoLog(
                    shader,
                    info.capacity() as _,
                    &mut len,
                    info.as_mut_ptr().cast(),
                );
            }
            info.set_len(len as usize);
            panic!(
                "compile error: (0x{:x}): {}",
                success,
                String::from_utf8_unchecked(info)
            );
        }

        shader
    }
}

pub unsafe fn link_shader(shaders: &[GLuint]) -> GLuint {
    unsafe {
        let shader_program = gl::CreateProgram();
        for shader in shaders {
            gl::AttachShader(shader_program, *shader);
        }
        gl::LinkProgram(shader_program);

        let mut success = 0;
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);

        if success == 0 {
            let mut info = Vec::<u8>::new();
            let mut len: GLsizei = 512;
            while info.capacity() < (len as usize) {
                info.reserve(len as usize);
                gl::GetProgramInfoLog(
                    shader_program,
                    info.capacity() as _,
                    &mut len,
                    info.as_mut_ptr().cast(),
                );
            }
            info.set_len(len as usize);
            panic!(
                "link error: (0x{:x}): {}",
                success,
                String::from_utf8_unchecked(info)
            );
        }

        shader_program
    }
}
