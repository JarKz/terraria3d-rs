use sdl2::{video::GLContext, EventPump};

extern crate sdl2;

use once_cell::sync::Lazy;
use parking_lot::Mutex;

pub static ASPECT_RATIO: Lazy<Mutex<f32>> = Lazy::new(|| Mutex::new(1.0));

pub struct Window {
    sdl: sdl2::Sdl,
    window_context: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
}

impl Window {
    pub const WIDTH: u32 = 1280;
    pub const HEIGHT: u32 = 720;

    const MAX_WIDTH: u32 = 2560;
    const MAX_HEIGHT: u32 = 1440;

    pub fn from(title: &str, width: u32, height: u32) -> Result<Self, Box<dyn std::error::Error>> {
        assert!(width > 0 && width <= Self::MAX_WIDTH);
        assert!(height > 0 && height <= Self::MAX_HEIGHT);

        let sdl = sdl2::init()?;
        let video_subsystem = sdl.video()?;

        let attr = video_subsystem.gl_attr();
        attr.set_context_profile(sdl2::video::GLProfile::Core);
        attr.set_context_version(4, 1);
        #[cfg(target_os = "macos")]
        {
            attr.set_context_flags().forward_compatible().set();
        }

        let window_context = video_subsystem
            .window(title, width, height)
            // .fullscreen_desktop()
            .opengl()
            .allow_highdpi()
            .build()?;

        let gl_context = window_context.gl_create_context()?;
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const gl::types::GLvoid);

        sdl.mouse().set_relative_mouse_mode(true);

        let (width, height) = window_context.size();
        *ASPECT_RATIO.lock() = width as f32 / height as f32;
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
            #[cfg(target_os = "macos")]
            {
                gl::Viewport(0, 0, width as i32 * 2, height as i32 * 2);
            }
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::Enable(gl::DEPTH_TEST);
        }
        Ok(Self {
            sdl,
            window_context,
            gl_context,
        })
    }

    pub fn event_pump(&self) -> Result<EventPump, String> {
        self.sdl.event_pump()
    }

    pub fn width(&self) -> u32 {
        self.window_context.size().0
    }

    pub fn height(&self) -> u32 {
        self.window_context.size().1
    }

    pub fn gl_context(&self) -> &GLContext {
        &self.gl_context
    }

    pub fn timer(&self) -> Result<sdl2::TimerSubsystem, String> {
        self.sdl.timer()
    }

    pub fn update(&self) {
        self.window_context.gl_swap_window();
    }
}
