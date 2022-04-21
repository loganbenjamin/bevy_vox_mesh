use bevy::asset::{Handle, LoadContext, LoadedAsset};
use bevy::hierarchy::{BuildWorldChildren, WorldChildBuilder};
use bevy::math::{Mat3, Quat, UVec3, Vec3, Vec4, Vec4Swizzles};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{Mesh, Transform, World};
use bevy::scene::Scene;
use bevy::transform::TransformBundle;
use dot_vox::{Dict, Model, SceneNode};

// constants used in magicavoxel's scene graph dictionaries
const ROTATION: &str = "_r";
const TRANSLATION: &str = "_t";

pub(crate) fn load_scene(
    ctx: &mut LoadContext,
    material: Handle<StandardMaterial>,
    models: &[Model],
    meshes: &[Handle<Mesh>],
    scene: &[SceneNode],
) {
    let mut world = World::default();
    if !scene.is_empty() {
        world
            .spawn()
            .insert_bundle(TransformBundle::identity())
            .with_children(|builder| {
                let root = &scene[0];
                let transform = Transform::identity();
                traverse_scene(builder, scene, root, transform, models, &material, meshes);
            });
    }
    ctx.set_default_asset(LoadedAsset::new(Scene::new(world)));
}

fn traverse_scene(
    builder: &mut WorldChildBuilder,
    scene: &[SceneNode],
    root: &SceneNode,
    root_transform: Transform,
    models: &[Model],
    material: &Handle<StandardMaterial>,
    meshes: &[Handle<Mesh>],
) {
    match root {
        SceneNode::Transform { frames, child, .. } => {
            if let Some(child_root) = scene.get(*child as usize) {
                let this_transform = Transform {
                    translation: extract_translation(frames).unwrap_or_default(),
                    rotation: extract_rotation(frames).unwrap_or_default(),
                    ..Transform::default()
                };
                let transform = root_transform * this_transform;

                traverse_scene(builder, scene, child_root, transform, models, material, meshes);
            }
        }
        SceneNode::Group { children, .. } => {
            for child in children {
                if let Some(child_root) = scene.get(*child as usize) {
                    traverse_scene(builder, scene, child_root, root_transform, models, material, meshes);
                }
            }
        }
        SceneNode::Shape { models: shape_models, .. } => {
            for model in shape_models {
                let id = model.model_id as usize;
                if let (Some(mesh), Some(model)) = (meshes.get(id), models.get(id)) {
                    // we swizzle z and y since bevy is y-up
                    let size = UVec3::new(model.size.x, model.size.z, model.size.y).as_vec3();
                    // `load_from_model` adds a 1-voxel border around the entire model, which we
                    // need to account for when calculating the pivot
                    let mut pivot = (size / 2.0).floor() + 1.0;
                    // we reverse x since MagicaVoxel's x axis is reversed
                    pivot[0] = -pivot[0];
                    let translation = root_transform.mul_vec3(-pivot).floor();
                    builder.spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Transform {
                            translation,
                            ..root_transform
                        },
                        ..PbrBundle::default()
                    });
                }
            }
        }
    }
}

fn extract_translation(frame: &[Dict]) -> Option<Vec3> {
    frame
        .get(0)
        .and_then(|x| x.get(TRANSLATION))
        .and_then(|translation| {
            let mut components = translation.split(' ');
            let x = components.next()?.parse::<f32>().ok()?;
            let y = components.next()?.parse::<f32>().ok()?;
            let z = components.next()?.parse::<f32>().ok()?;
            if components.next() == None {
                // we swizzle z and y since bevy is y-up
                // we reverse x since MagicaVoxel's x axis is reversed
                Some(Vec3::new(-x, z, y))
            } else {
                // there shouldn't be more than 3 components, bail
                None
            }
        })
}

// Based on https://github.com/jpaver/opengametools/blob/master/src/ogt_vox.h#L821
fn extract_rotation(frame: &[Dict]) -> Option<Quat> {
    frame
        .get(0)
        .and_then(|x| x.get(ROTATION))
        .and_then(|translation| {
            let packed = translation.parse::<u32>().ok()?;
            let index0 = packed & 0b11;
            let index1 = (packed >> 2u32) & 0b11;
            let index2 = (1u32 << index0 | 1u32 << index1).trailing_ones();

            #[inline(always)]
            fn negate_if(x: u32) -> f32 {
                if x == 0 { 1.0 } else { -1.0 }
            }

            let mut mat = Mat3::ZERO;
            mat.x_axis[index0 as usize] = negate_if(packed & (1 << 4));
            mat.y_axis[index1 as usize] = negate_if(packed & (1 << 5));
            mat.z_axis[index2 as usize] = negate_if(packed & (1 << 6));

            // we swizzle z and y since bevy is y-up
            Some(Quat::from_vec4(Vec4::from(Quat::from_mat3(&mat)).xzyw()))
        })
}
