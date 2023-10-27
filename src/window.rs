use sdl2::{EventPump, video::GLContext};

extern crate sdl2;

pub struct Window {
    sdl: sdl2::Sdl,
    window_context: sdl2::video::Window,
    gl_context: sdl2::video::GLContext,
    width: u32,
    height: u32,
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
            .opengl()
            .allow_highdpi()
            .build()?;

        let gl_context = window_context.gl_create_context()?;
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const gl::types::GLvoid);

        Ok(Self { sdl, window_context, gl_context, width, height })
    }

    pub fn event_pump(&self) -> Result<EventPump, String> {
        self.sdl.event_pump()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
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
