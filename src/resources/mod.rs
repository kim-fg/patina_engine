mod hdr_loader;
pub use hdr_loader::HdrLoader; // re-export for easier access

use std::io::{BufReader, Cursor};
use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use crate::{model, texture};

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("learn-wgpu") {
        origin = format!("{}/learn-wgpu", origin);
    }

    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    is_normal_map: bool,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name, is_normal_map)
}

pub async fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader, 
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        }, 
        |path| async move {
            let mat_text = load_string(&path).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        }
    ).await?;

    let mut materials = Vec::new();
    for material in obj_materials? {
        let diffuse_texture = load_texture(&material.diffuse_texture, false, device, queue).await?;
        let normal_texture = load_texture(&material.normal_texture, true, device, queue).await?;
        
        materials.push(model::Material::new(
            device,
            &material.name,
            diffuse_texture,
            normal_texture,
            layout
        ));
    }

    //todo! clean this up - way too much repetition that can be fixed.
    let meshes = models.into_iter()
        .map(|model| {
            //let mesh = &model.mesh;
            let mut vertices = (0..model.mesh.positions.len() / 3).map(|i| model::ModelVertex {
                position: [
                    model.mesh.positions[i * 3 + 0],
                    model.mesh.positions[i * 3 + 1],
                    model.mesh.positions[i * 3 + 2],
                ],
                tex_coords: [model.mesh.texcoords[i * 2], 1.0 - model.mesh.texcoords[i * 2 + 1]],
                normal: [
                    model.mesh.normals[i * 3 + 0],
                    model.mesh.normals[i * 3 + 1],
                    model.mesh.normals[i * 3 + 2],
                ],
                tangent: [0.0; 3],
                bitangent: [0.0; 3],
            }).collect::<Vec<_>>();

            let indices = &model.mesh.indices;
            let mut triangles_included = vec![0; vertices.len()];

            for chunk in indices.chunks(3) {
                let i0 = chunk[0] as usize;
                let i1 = chunk[1] as usize;
                let i2 = chunk[2] as usize;

                let v0 = vertices[i0];
                let v1 = vertices[i1];
                let v2 = vertices[i2];

                let pos0: cgmath::Vector3<_> = v0.position.into();
                let pos1: cgmath::Vector3<_> = v1.position.into();
                let pos2: cgmath::Vector3<_> = v2.position.into();

                let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
                let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
                let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

                // Calculate the edges of the triangle
                let delta_pos1 = pos1 - pos0;
                let delta_pos2 = pos2 - pos0;

                // This will give us a direction to calculate 
                // the tangent and bitangent
                let delta_uv1 = uv1 - uv0;
                let delta_uv2 = uv2 - uv0;

                //todo! figure this out.. it makes no sense to me
                // wtf is an r
                let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
                let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;

                // We flip the bitangent to enable right-handed normal
                // maps with wgpu texture coordinate system
                let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

                // We'll use the same tangent/bitangent for each vertex in the triangle
                vertices[i0].tangent = (tangent + cgmath::Vector3::from(vertices[i0].tangent)).into();
                vertices[i1].tangent = (tangent + cgmath::Vector3::from(vertices[i1].tangent)).into();
                vertices[i2].tangent = (tangent + cgmath::Vector3::from(vertices[i2].tangent)).into();

                vertices[i0].bitangent = (bitangent + cgmath::Vector3::from(vertices[i0].bitangent)).into();
                vertices[i1].bitangent = (bitangent + cgmath::Vector3::from(vertices[i1].bitangent)).into();
                vertices[i2].bitangent = (bitangent + cgmath::Vector3::from(vertices[i2].bitangent)).into();

                // used to average the tangents/bitangents
                triangles_included[i0] += 1;
                triangles_included[i1] += 1;
                triangles_included[i2] += 1;
            }

            for (i, n) in triangles_included.into_iter().enumerate() {
                let denom = 1.0 / n as f32;
                let mut v = &mut vertices[i];
                v.tangent = (cgmath::Vector3::from(v.tangent) * denom).into();
                v.bitangent = (cgmath::Vector3::from(v.bitangent) * denom).into();
            }
                
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&model.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: model.mesh.indices.len() as u32,
                material: model.mesh.material_id.unwrap_or(0),
            }
        }).collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}