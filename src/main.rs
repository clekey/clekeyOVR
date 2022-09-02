mod sdl2_glium;

use crate::sdl2_glium::DisplayBuild;
use glium::backend::Facade;
use glium::texture::{DepthFormat, RawImage2d};
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use glium::{implement_vertex, Program, Surface, VertexBuffer};
use glium::framebuffer::{DepthRenderBuffer, MultiOutputFrameBuffer, RenderBuffer, SimpleFrameBuffer};
use openvr::cstr;
use sdl2::video::GLProfile;
use openvr::overlay::OwnedInVROverlay;

const WINDOW_HEIGHT: u32 = 256;
const WINDOW_WIDTH: u32 = 512;

fn main() {
    // sdl initialization
    let sdl = sdl2::init().expect("sdl initialization error");
    let sdl_video = sdl.video().expect("sdl video");
    sdl_video.gl_attr().set_double_buffer(true);
    sdl_video.gl_attr().set_context_major_version(4);
    sdl_video.gl_attr().set_context_minor_version(1);
    sdl_video.gl_attr().set_context_profile(GLProfile::Core);

    let window = sdl_video
        .window("clekeyOVR", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position(0, 0)
        .opengl()
        .build_glium()
        .expect("window creation");

    // openvr initialization

    let ovr = openvr::init(openvr::ApplicationType::Overlay).expect("ovr");

    let overlay = ovr.overlay().expect("openvr overlay must be accessible");
    let input = ovr.input().expect("openvr input must be accessible");

    input
        .set_action_manifest_path(cstr!(r"C:\Users\anata\clekey-ovr-build\actions.json"))
        .expect("");

    let action_left_stick = input.get_action_handle(cstr!("/actions/input/in/left_stick")).expect("action left_stick not found");
    let action_left_click = input.get_action_handle(cstr!("/actions/input/in/left_click")).expect("action left_click not found");
    let action_left_haptic = input.get_action_handle(cstr!("/actions/input/in/left_haptic")).expect("action left_haptic not found");
    let action_right_stick = input.get_action_handle(cstr!("/actions/input/in/right_stick")).expect("action right_stick not found");
    let action_right_click = input.get_action_handle(cstr!("/actions/input/in/right_click")).expect("action right_click not found");
    let action_right_haptic = input.get_action_handle(cstr!("/actions/input/in/right_haptic")).expect("action right_haptic not found");
    let action_set_input = input.get_action_set_handle(cstr!("/actions/input"));

    let overlay_handle = OwnedInVROverlay::new(overlay, cstr!("com.anatawa12.clekey-ovr"), cstr!("clekey-ovr")).expect("create overlay");

    overlay_handle.set_overlay_width_in_meters(2.0).expect("overlay");
    overlay_handle.set_overlay_alpha(1.0).expect("overlay");

    // gl main

    let shader_program = Program::from_source(
        &window,
        concat!(
        "#version 330 core\n",
        "layout(location = 0) in vec3 position;\n",
        "layout(location = 1) in vec4 color;\n",
        "out vec4 out_color;\n",
        "void main() {\n",
        "    gl_Position.xyz = position;\n",
        "    out_color = color;\n",
        "}\n",
        ),
        concat!(
                "#version 330 core\n",
                "in vec4 out_color;\n",
                "",
                "// Ouput data\n",
                "layout(location = 0) out vec4 color;\n",
                "\n",
                "void main() {\n",
                "    // Output color = red \n",
                "    color = out_color;\n",
        "}\n",
        ),
        None,
    ).expect("shader_err");
    let dest_texture = glium::Texture2d::empty(&window, WINDOW_WIDTH, WINDOW_HEIGHT).expect("main texture creation");
    let depth_buffer = DepthRenderBuffer::new(&window, DepthFormat::F32, WINDOW_WIDTH, WINDOW_HEIGHT).expect("depth buffer creation");
    let mut frame_buffer = SimpleFrameBuffer::with_depth_buffer(&window, &dest_texture, &depth_buffer).expect("framebuffer creation");

    let texture = {
        // for debugging, make a image
        let mut data = vec![0 as u8; (WINDOW_WIDTH * WINDOW_HEIGHT * 4) as usize];
        for rgba in data.chunks_mut(4) {
            rgba[0] = 0x80;
            rgba[1] = 0x40;
            rgba[2] = 0xC0;
            rgba[3] = 0xFF;
        }
        let data = RawImage2d::from_raw_rgba(data, (WINDOW_WIDTH, WINDOW_HEIGHT));
        let texture = glium::Texture2d::new(&window, data).expect("main texture creation");

        texture
    };
    let texture_sampled = texture
        .sampled()
        .minify_filter(MinifySamplerFilter::Linear)
        .magnify_filter(MagnifySamplerFilter::Nearest);

    frame_buffer.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 0.0);
    #[derive(Copy, Clone)]
    struct Vertex {
        position: [f32; 3],
        color: [f32; 4],
    }
    implement_vertex!(Vertex, position, color);

    let vertexbuffer = VertexBuffer::new(&window, &[
        Vertex { position: [-1.0, -1.0, 0.0], color: [1.0, 0.0, 0.0, 1.0] },
        Vertex { position: [1.0, -1.0, 0.0], color: [0.0, 1.0, 0.0, 1.0] },
        Vertex { position: [0.0, 1.0, 0.0], color: [0.0, 0.0, 1.0, 1.0] },
    ]).expect("vertex buffer creation");

    frame_buffer.draw(
        &vertexbuffer,
        glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
        &shader_program,
        &glium::uniforms::EmptyUniforms,
        &Default::default()
    ).expect("draw");

    let mut frame = window.draw();
    //frame.clear_color();

    println!("Hello, world!");
}
