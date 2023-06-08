use glam::*;
use std::ffi::CString;

#[derive(Clone, Debug)]
pub struct Texture {
    data: Vec<u8>,
    width: u32,
    height: u32,
    channel_count: usize
}

pub fn load_texture(file_path: &str) -> Texture {
    let file_path = CString::new(file_path.as_bytes()).unwrap();

    unsafe {
        let mut width = 0;
        let mut height = 0;
        let mut channel_count = 0;
        let data = stb_image::stb_image::bindgen::stbi_load(
            file_path.as_ptr(),
            &mut width,
            &mut height,
            &mut channel_count,
            0,
        );

        assert!(!data.is_null(), "Failed to load texture.");
        let data: Vec<u8> = std::slice::from_raw_parts(
            data,
            (width * height * channel_count) as usize
        ).to_vec();

        Texture {
            data,
            width: width as u32,
            height: height as u32,
            channel_count: channel_count as usize
        }
    }
}

impl Texture {
    pub fn sample_pixel(&self, x: f32, y: f32) -> Vec4 {
        let inv_dims = Vec2::new(1.0 / self.width as f32, 1.0 / self.height as f32);

        let tl = self.get_pixel(x - inv_dims.x, y - inv_dims.y);
        let bl = self.get_pixel(x - inv_dims.x, y + inv_dims.y);
        let br = self.get_pixel(x + inv_dims.x, y + inv_dims.y);
        let tr = self.get_pixel(x + inv_dims.x, y - inv_dims.y);
        
        let x = x * self.width as f32;
        let y = y * self.height as f32;
        let dx = x - ((x as i32) as f32);
        let dy = y - ((y as i32) as f32);

        let bottom = bl.lerp(br, dx);
        let top = tl.lerp(tr, dx);
        top.lerp(bottom, dy)
    }

    pub fn get_pixel(&self, x: f32, y: f32) -> Vec4 {
        let x = ((x * self.width as f32) as usize) % (self.width - 1) as usize;
        let y = ((y * self.height as f32) as usize) % (self.height - 1) as usize;

        match self.channel_count {
            4 => {
                let data: &Vec<(u8, u8, u8, u8)> = unsafe { std::mem::transmute(&self.data) };
                let pixel = &data[y * (self.width as usize) + x];
    
                Vec4::new(pixel.0 as f32 / 255.99, pixel.1 as f32 / 255.99, pixel.2 as f32 / 255.99, pixel.3 as f32 / 255.99)
            },
            3 => {
                let data: &Vec<(u8, u8, u8)> = unsafe { std::mem::transmute(&self.data) };
                let pixel = &data[y * (self.width as usize) + x];
    
                Vec4::new(pixel.0 as f32 / 255.99, pixel.1 as f32 / 255.99, pixel.2 as f32 / 255.99, 0.0)
            }
            _ => panic!("Failed to get pixel. (Unsupported channel count)")
        }
    }
}