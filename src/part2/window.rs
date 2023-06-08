pub struct Window {
    window: minifb::Window,
    framebuffer: Framebuffer
}

pub struct Framebuffer {
    data: Vec<u32>,
    width: usize,
    height: usize
}

impl Window {
    pub fn new(name: &str, width: usize, height: usize) -> Self {
        let options = minifb::WindowOptions {
            resize: true,
            ..Default::default()
        };

        let window = minifb::Window::new(
            name,
            width,
            height,
            options
        ).expect("Failed to create window.");

        let framebuffer = Framebuffer::new(width, height);

        Window {
            window,
            framebuffer
        }
    }

    pub fn should_close(&self) -> bool {
        !self.window.is_open()
    }

    pub fn display(&mut self) {
        self.window.update_with_buffer(
            &self.framebuffer.data,
            self.framebuffer.width(),
            self.framebuffer.height()
        ).expect("Failed to update window buffer.");

        let (width, height) = self.window.get_size();
        if width != self.framebuffer.width() || height != self.framebuffer.height() {
            self.framebuffer = Framebuffer::new(width, height);
        }
    }

    pub fn framebuffer(&mut self) -> &mut Framebuffer {
        &mut self.framebuffer
    }
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            data: vec![0; width * height],
            width,
            height
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, value: u32) {
        self.data[x + y * self.width] = value;
    }

    pub fn set_pixel_f32(&mut self, x: usize, y: usize, value: f32) {
        self.data[y * self.width + x] = (value * u32::MAX as f32) as u32;
    }

    pub fn get_pixel_f32(&mut self, x: usize, y: usize) -> f32 {
        self.data[y * self.width + x] as f32 / u32::MAX as f32
    }

    pub fn clear(&mut self, value: u32) {
        for i in 0..self.data.len() {
            self.data[i] = value;
        }
    }
}