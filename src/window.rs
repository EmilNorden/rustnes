use sdl2::video::GLContext;

pub struct Window {
    window: sdl2::video::Window,
    context: GLContext,
}

impl Window {
    pub fn create(sdl: &sdl2::Sdl) -> Result<Window, String> {
        let video = sdl.video().unwrap();

        let sdl_window2 = video
            .window("test", 800, 600)
            .opengl()
            .position_centered()
            .build();

        let sdl_window = match sdl_window2 {
            Err(e) => Err(e.to_string()),
            Ok(w) => Ok(w)
        }?;

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 3);

        let gl_context = sdl_window.gl_create_context()?;
        let _gl = gl::load_with(|s| video.gl_get_proc_address(s) as *const std::os::raw::c_void);

        unsafe {
            gl::Viewport(0, 0, 512, 512);
            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
        }

        let result = Window {
            window: sdl_window,
            context: gl_context
        };
        Ok(result)
    }

    pub fn swap(&self) {
        self.window.gl_swap_window();
    }
}