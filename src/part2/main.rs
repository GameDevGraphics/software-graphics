use std::time::SystemTime;

use glam::*;

mod window;
use window::{Window, Framebuffer};
mod model;
use model::{Model, load_model};

fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn edge_function(a: &Vec2, c: &Vec2, b: &Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

fn inside_triangle(a: &Vec2, b: &Vec2, c: &Vec2, p: &Vec2) -> bool {
    let a0 = edge_function(b, c, p);
    let a1 = edge_function(c, a, p);
    let a2 = edge_function(a, b, p);

    let mut overlaps = true;
    let edge0 = *c - *b;
    let edge1 = *a - *c;
    let edge2 = *b - *a;
    overlaps &= a0 > 0.0;//if a0 == 0.0 { (edge0.y == 0.0 && edge0.x > 0.0) || edge0.y > 0.0 } else { a0 > 0.0 };
    overlaps &= a1 > 0.0;//if a1 == 0.0 { (edge1.y == 0.0 && edge1.x > 0.0) || edge1.y > 0.0 } else { a1 > 0.0 };
    overlaps &= a2 > 0.0;//if a1 == 0.0 { (edge2.y == 0.0 && edge2.x > 0.0) || edge2.y > 0.0 } else { a2 > 0.0 };

    overlaps
}

fn draw_triangle(framebuffer: &mut Framebuffer, a: &Vec2, b: &Vec2, c: &Vec2, color: u32) {
    // let width = framebuffer.width();
    // let height = framebuffer.height();

    let min = a.min(b.min(*c)).max(Vec2::ZERO);// * Vec2::new(width as f32, height as f32);
    let max = (a.max(b.max(*c)) + 1.0).min(Vec2::new(framebuffer.width() as f32, framebuffer.height() as f32));// * Vec2::new(width as f32, height as f32);

    for x in (min.x as usize)..(max.x as usize) {
        for y in (min.y as usize)..(max.y as usize) {
            let p = Vec2::new(
                x as f32,// / width as f32,
                y as f32// / height as f32
            ) + 0.5;
            
            let inside = inside_triangle(a, b, c, &p);
            if inside {
                framebuffer.set_pixel(x, y, color);
            }
        }
    }
}

fn project(p: &Vec3, mvp: &Mat4) -> (Vec3, f32) {
    let proj_pos = *mvp * Vec4::from((*p, 1.0));
    let rec = 1.0 / proj_pos.w;
    let rec_pos = proj_pos * rec;
    (Vec3::new(rec_pos.x, rec_pos.y, rec_pos.z), rec)
}

fn clip_to_screen_space(clip_space: &Vec2, screen_size: &Vec2) -> Vec2 {
    (*clip_space * -0.5 + 0.5) * *screen_size
}

fn draw_model(framebuffer: &mut Framebuffer, model: &Model, mvp: &Mat4) {
    for mesh in &model.meshes {
        for i in 0..(mesh.indices.len() / 3) {
            let v0 = mesh.vertices[mesh.indices[i * 3] as usize];
            let v1 = mesh.vertices[mesh.indices[i * 3 + 1] as usize];
            let v2 = mesh.vertices[mesh.indices[i * 3 + 2] as usize];

            let v0_clip_space = project(&v0.position, mvp);
            let v1_clip_space = project(&v1.position, mvp);
            let v2_clip_space = project(&v2.position, mvp);

            let screen_size = Vec2::new(framebuffer.width() as f32, framebuffer.height() as f32);
            let v0_screen_space = clip_to_screen_space(&v0_clip_space.0.xy(), &screen_size);
            let v1_screen_space = clip_to_screen_space(&v1_clip_space.0.xy(), &screen_size);
            let v2_screen_space = clip_to_screen_space(&v2_clip_space.0.xy(), &screen_size);

            draw_triangle(
                framebuffer,
                &v0_screen_space,
                &v1_screen_space,
                &v2_screen_space,
                from_u8_rgb(200, 200, 100)
            );
        }
    }
}

fn main() {
    let mut window = Window::new("3D graphics from scratch! (PART 2)", 512, 512);

    let model = load_model("assets/DamagedHelmet/DamagedHelmet.gltf");

    let timer = SystemTime::now();

    while !window.should_close() {
        let framebuffer = window.framebuffer();

        framebuffer.clear(from_u8_rgb(20, 20, 20));

        let aspect_ratio = framebuffer.width() as f32 / framebuffer.height() as f32;
        let model_matrix = Mat4::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), timer.elapsed().unwrap().as_secs_f32());
        let view_matrix = Mat4::from_translation(Vec3::new(0.0, 0.0, -5.0));
        let proj_matrix = Mat4::perspective_rh((60.0f32).to_radians(), aspect_ratio, 0.01, 300.0);
        let mvp_matrix = proj_matrix * view_matrix * model_matrix;

        draw_model(
            framebuffer,
            &model,
            &mvp_matrix
        );

        window.display();
    }
}