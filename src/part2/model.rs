use glam::*;

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: Vec3::ZERO,
            normal: Vec3::ZERO
        }
    }
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_idx: usize
}

#[derive(Clone, Debug)]
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>
}

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub base_color: Vec4
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: Vec4::ONE
        }
    }
}

pub fn load_model(file_path: &str) -> Model {
    let (document, buffers, _images) = gltf::import(file_path)
        .expect("Failed to load model.");

    let mut meshes = Vec::new();
    let mut materials = vec![Material::default(); document.materials().len()];
    if materials.is_empty() {
        materials.push(Material::default());
    }
    
    if document.nodes().len() > 0 {
        process_node(
            document.nodes().next().as_ref().unwrap(),
            &buffers,
            &mut meshes,
            &mut materials
        );
    }

    Model {
        meshes,
        materials
    }
}

fn process_node(
    node: &gltf::Node,
    buffers: &[gltf::buffer::Data],
    meshes: &mut Vec<Mesh>,
    materials: &mut [Material]
) {
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            if primitive.mode() == gltf::mesh::Mode::Triangles {
                let reader = primitive.reader(
                    |buffer| Some(&buffers[buffer.index()])
                );

                let positions = {
                    let iter = reader
                        .read_positions()
                        .expect("Failed to process mesh node. (Vertices must have positions)");

                    iter.map(|arr| -> Vec3 { Vec3::from(arr) }).collect::<Vec<_>>()
                };

                let mut vertices: Vec<Vertex> = positions
                    .into_iter()
                    .map(|position| {
                        Vertex {
                             position,
                             ..Default::default()
                        }
                }).collect();

                if let Some(normals) = reader.read_normals() {
                    for (i, normal) in normals.enumerate() {
                        vertices[i].normal = Vec3::from(normal);
                    }
                }

                let indices = reader
                    .read_indices()
                    .map(|read_indices| {
                        read_indices.into_u32().collect::<Vec<_>>()
                    }).expect("Failed to process mesh node. (Indices are required)");
                
                let prim_material = primitive.material();
                let pbr = prim_material.pbr_metallic_roughness();
                let material_idx = primitive.material().index().unwrap_or(0);

                let material = &mut materials[material_idx];
                material.base_color = Vec4::from(pbr.base_color_factor());

                meshes.push(Mesh {
                    vertices,
                    indices,
                    material_idx
                });
            }
        }
    }
}