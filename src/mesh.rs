use bevy::render::{
    mesh::{Indices, Mesh, VertexAttributeValues},
    render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
    texture::Image,
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, QuadCoordinateConfig};
use ndshape::{Shape, Shape3u32};
use num_integer::Roots;

use crate::voxel::Voxel;

pub(crate) fn palette_to_texture(palette: &[[u8; 4]]) -> (Image, u8) {
    let texture_width = palette.len().sqrt().next_power_of_two();
    let texture_size = texture_width * texture_width;
    let texture_data = {
        let mut data = Vec::with_capacity(texture_size * 4);
        for color in palette {
            data.extend_from_slice(color);
        }
        for _ in palette.len()..texture_size {
            data.extend_from_slice(&[0; 4]);
        }
        data
    };
    let texture = Image::new(
        Extent3d {
            width: texture_width as u32,
            height: texture_width as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        texture_data,
        TextureFormat::Rgba8UnormSrgb,
    );
    (texture, texture_width as u8)
}

pub(crate) fn mesh_model(
    buffer_shape: Shape3u32,
    buffer: &[Voxel],
    palette_texture_width: u8,
    quads_config: &QuadCoordinateConfig,
    _v_flip_face: bool,
) -> Mesh {
    let mut greedy_quads_buffer = GreedyQuadsBuffer::new(buffer_shape.size() as usize);

    greedy_quads(
        buffer,
        &buffer_shape,
        [0; 3],
        buffer_shape.as_array().map(|x| x - 1),
        &quads_config.faces,
        &mut greedy_quads_buffer,
    );

    let num_indices = greedy_quads_buffer.quads.num_quads() * 6;
    let num_vertices = greedy_quads_buffer.quads.num_quads() * 4;

    let mut indices = Vec::with_capacity(num_indices);
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    for (group, face) in greedy_quads_buffer
        .quads
        .groups
        .iter()
        .zip(quads_config.faces.as_ref())
    {
        for quad in group.iter() {
            indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
            positions.extend_from_slice(&face.quad_mesh_positions(quad, 1.0));
            normals.extend_from_slice(&face.quad_mesh_normals());

            let palette_index = buffer[buffer_shape.linearize(quad.minimum) as usize].0;
            let base_u = (palette_index % palette_texture_width) as f32 / (palette_texture_width as f32);
            let base_v = (palette_index / palette_texture_width) as f32 / (palette_texture_width as f32);
            uvs.extend_from_slice(&[[base_u, base_v]; 4]);
        }
    }

    render_mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    render_mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    render_mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    render_mesh
}
