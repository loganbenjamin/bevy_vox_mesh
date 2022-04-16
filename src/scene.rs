use bevy::asset::{Handle, LoadContext, LoadedAsset};
use bevy::hierarchy::{BuildWorldChildren, WorldChildBuilder};
use bevy::math::{UVec3};
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::{Mesh, Transform, World};
use bevy::scene::Scene;
use bevy::transform::TransformBundle;
use dot_vox::{Model, SceneNode};

// constants used in magicavoxel's scene graph dictionaries
const TRANSLATION: &str = "_t";

pub(crate) fn load_scene(
    ctx: &mut LoadContext,
    material: Handle<StandardMaterial>,
    models: &Vec<Model>,
    meshes: Vec<Handle<Mesh>>,
    scene: &Vec<SceneNode>,
) {
    let mut world = World::default();
    if !scene.is_empty() {
        world
            .spawn()
            .insert_bundle(TransformBundle::identity())
            .with_children(|builder| {
                let root = &scene[0];
                let transform = Transform::identity();
                traverse_scene(builder, scene, root, transform, models, &material, &meshes);
            });
    }
    ctx.set_default_asset(LoadedAsset::new(Scene::new(world)));
}

fn traverse_scene(
    builder: &mut WorldChildBuilder,
    scene: &Vec<SceneNode>,
    root: &SceneNode,
    root_transform: Transform,
    models: &Vec<Model>,
    material: &Handle<StandardMaterial>,
    meshes: &Vec<Handle<Mesh>>,
) {
    match root {
        SceneNode::Transform { frames, child, .. } => {
            if let Some(child_root) = scene.get(*child as usize) {
                let transform = frames
                    .get(0)
                    .and_then(|x| x.get(TRANSLATION))
                    .and_then(|translation| {
                        let mut components = translation.split(" ");
                        let x = components.next()?.parse::<f32>().ok()?;
                        let y = components.next()?.parse::<f32>().ok()?;
                        let z = components.next()?.parse::<f32>().ok()?;
                        if components.next() == None {
                            // note: we swizzle z and y since bevy is y-up
                            Some(Transform::from_xyz(x, z, y))
                        } else {
                            // there shouldn't be more than 3 components, bail
                            None
                        }
                    })
                    .map(|x| root_transform * x)
                    .unwrap_or(root_transform);

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
                    // note: we swizzle z and y since bevy is y-up
                    let pivot = UVec3::new(model.size.x / 2, model.size.z / 2, model.size.y / 2).as_vec3();
                    let mut transform = root_transform.clone();
                    transform.translation -= pivot;
                    builder.spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform,
                        ..PbrBundle::default()
                    });
                }
            }
        }
    }
}
