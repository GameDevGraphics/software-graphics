use std::time::SystemTime;

use glam::*;

mod window;
use window::{Window, Framebuffer};
mod model;
use model::{Model, Vertex, Material, load_model};
mod texture;
use texture::{Texture, load_texture};

fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn from_vec3_rgb(rgb: &Vec3) -> u32 {
    from_u8_rgb((rgb.x * 255.99) as u8, (rgb.y * 255.99) as u8, (rgb.z * 255.99) as u8)
}

fn edge_function(a: &Vec2, c: &Vec2, b: &Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

fn draw_triangle(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut Framebuffer,
    v0: &Vertex, v1: &Vertex, v2: &Vertex,
    mvp: &Mat4,
    inv_trans_model_matrix: &Mat4,
    material: &Material
) {
    let v0_clip_space = project(&v0.position, mvp);
    let v1_clip_space = project(&v1.position, mvp);
    let v2_clip_space = project(&v2.position, mvp);

    let screen_size = Vec2::new(framebuffer.width() as f32, framebuffer.height() as f32);
    let v0_screen_space = clip_to_screen_space(&v0_clip_space.0.xy(), &screen_size);
    let v1_screen_space = clip_to_screen_space(&v1_clip_space.0.xy(), &screen_size);
    let v2_screen_space = clip_to_screen_space(&v2_clip_space.0.xy(), &screen_size);

    let min = v0_screen_space.min(v1_screen_space.min(v2_screen_space)).max(Vec2::ZERO);
    let max = (v0_screen_space.max(v1_screen_space.max(v2_screen_space)) + 1.0).min(screen_size);

    for x in (min.x as usize)..(max.x as usize) {
        for y in (min.y as usize)..(max.y as usize) {
            let p = Vec2::new(x as f32, y as f32) + 0.5;
            
            let a0 = edge_function(&v1_screen_space, &v2_screen_space, &p);
            let a1 = edge_function(&v2_screen_space, &v0_screen_space, &p);
            let a2 = edge_function(&v0_screen_space, &v1_screen_space, &p);
            let overlaps = a0 > 0.0 && a1 > 0.0 && a2 > 0.0;
            
            if overlaps {
                let area_rep = 1.0 / edge_function(&v0_screen_space, &v1_screen_space, &v2_screen_space);
                let bary_coords = Vec3::new(a0, a1, a2) * area_rep;
                let correction = 1.0 / (bary_coords.x * v0_clip_space.1
                                            + bary_coords.y * v1_clip_space.1
                                            + bary_coords.z * v2_clip_space.1);

                let z = v0_clip_space.0.z * bary_coords.x
                        + v1_clip_space.0.z * bary_coords.y
                        + v2_clip_space.0.z * bary_coords.z;
                let depth = depth_buffer.get_pixel_f32(x, y);

                if z < depth {
                    depth_buffer.set_pixel_f32(x, y, z);

                    let n0 = *inv_trans_model_matrix * Vec4::from((v0.normal, 1.0));
                    let n1 = *inv_trans_model_matrix * Vec4::from((v1.normal, 1.0));
                    let n2 = *inv_trans_model_matrix * Vec4::from((v2.normal, 1.0));
                    let normal = ((n0 * v0_clip_space.1 * bary_coords.x
                                        + n1 * v1_clip_space.1 * bary_coords.y
                                        + n2 * v2_clip_space.1 * bary_coords.z).xyz()
                                            * correction).normalize();
                    
                    let tex_coord = (v0.tex_coord * v0_clip_space.1 * bary_coords.x
                                            + v1.tex_coord * v1_clip_space.1 * bary_coords.y
                                            + v2.tex_coord * v2_clip_space.1 * bary_coords.z) * correction;

                    let mut base_color = material.base_color;
                    if let Some(base_color_texture) = &material.base_color_texture {
                        base_color *= base_color_texture.sample_pixel(tex_coord.x, tex_coord.y);
                    }

                    let light_dir = Vec3::new(0.3, -0.8, -0.4).normalize();
                    let light_intensity = normal.dot(-light_dir);

                    let final_color = base_color * light_intensity;

                    framebuffer.set_pixel(x, y, from_vec3_rgb(&final_color.xyz()));
                }
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

fn draw_model(
    framebuffer: &mut Framebuffer,
    depth_buffer: &mut Framebuffer,
    model: &Model,
    mvp: &Mat4,
    inv_trans_model_matrix: &Mat4
) {
    for mesh in &model.meshes {
        for i in 0..(mesh.indices.len() / 3) {
            let v0 = mesh.vertices[mesh.indices[i * 3] as usize];
            let v1 = mesh.vertices[mesh.indices[i * 3 + 1] as usize];
            let v2 = mesh.vertices[mesh.indices[i * 3 + 2] as usize];

            let material = &model.materials[mesh.material_idx];

            draw_triangle(
                framebuffer,
                depth_buffer,
                &v0, &v1, &v2,
                mvp,
                inv_trans_model_matrix,
                material
            );
        }
    }
}

fn main() {
    let mut window = Window::new("3D graphics from scratch! (PART 3)", 512, 512);
    let mut depth_buffer = Framebuffer::new(window.framebuffer().width(), window.framebuffer().height());

    let model = load_model("assets/DamagedHelmet/DamagedHelmet.gltf");

    let timer = SystemTime::now();

    while !window.should_close() {
        let framebuffer = window.framebuffer();

        if framebuffer.width() != depth_buffer.width() || framebuffer.height() != depth_buffer.height() {
            depth_buffer = Framebuffer::new(framebuffer.width(), framebuffer.height());
        }

        framebuffer.clear(from_u8_rgb(20, 20, 20));
        depth_buffer.clear(u32::MAX);

        let aspect_ratio = framebuffer.width() as f32 / framebuffer.height() as f32;
        let model_matrix = Mat4::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), timer.elapsed().unwrap().as_secs_f32()) * Mat4::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), (90.0f32).to_radians());
        let view_matrix = Mat4::from_translation(Vec3::new(0.0, 0.0, -2.5));
        let proj_matrix = Mat4::perspective_rh((60.0f32).to_radians(), aspect_ratio, 0.01, 300.0);
        let mvp_matrix = proj_matrix * view_matrix * model_matrix;
        let inv_trans_model_matrix = model_matrix.inverse().transpose();

        draw_model(
            framebuffer,
            &mut depth_buffer,
            &model,
            &mvp_matrix,
            &inv_trans_model_matrix
        );

        window.display();
    }
}