use bevy::render::{
    mesh::{Indices, Mesh, VertexAttributeValues},
    render_resource::PrimitiveTopology,
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, QuadCoordinateConfig};
use ndshape::{RuntimeShape, Shape};

use crate::voxel::Voxel;

pub(crate) fn mesh_model(
    buffer_shape: RuntimeShape<u32, 3>,
    buffer: &[Voxel],
    palette: &[[f32; 4]],
    quads_config: &QuadCoordinateConfig,
    v_flip_face: bool,
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
    let mut colors = Vec::with_capacity(num_vertices);

    let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let offset_center = buffer_shape
        .as_array()
        .map(|p| ((p - 2) as f32 * 0.5).floor());

    for (group, face) in greedy_quads_buffer
        .quads
        .groups
        .iter()
        .zip(quads_config.faces.as_ref())
    {
        for quad in group.iter() {
            // we reverse x since MagicaVoxel's x axis is reversed
            indices.extend_from_slice(&{
                let mut next = face.quad_mesh_indices(positions.len() as u32);
                next.swap(0, 2);
                next.swap(3, 5);
                next
            });

            let translate_x = |mut vec: [f32; 3]| {
                vec[0] = vec[0] - offset_center[0];
                vec[1] = vec[1] - offset_center[1];
                vec[2] = vec[2] - offset_center[2];
                vec[0] = -vec[0];
                vec
            };

            let negate_x = |mut vec: [f32; 3]| {
                vec[0] = -vec[0];
                vec
            };

            positions.extend_from_slice(
                &face
                    .quad_mesh_positions(quad, 1.0)
                    .map(|position| position.map(|x| x - 1.0)) // corrects the 1 offset introduced by the meshing.
                    .map(translate_x),
            );

            normals.extend_from_slice(&face.quad_mesh_normals().map(negate_x));

            let palette_index = buffer[buffer_shape.linearize(quad.minimum) as usize].0;
            colors.extend_from_slice(&[palette[palette_index as usize]; 4]);
            uvs.extend_from_slice(&face.tex_coords(quads_config.u_flip_face, v_flip_face, quad));
        }
    }

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    render_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));

    render_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        VertexAttributeValues::Float32x4(colors),
    );

    render_mesh.set_indices(Some(Indices::U32(indices.clone())));

    render_mesh
}
