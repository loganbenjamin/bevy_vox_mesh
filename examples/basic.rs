use bevy::prelude::*;
use bevy_vox_mesh::VoxMeshPlugin;
use std::f32::consts::PI;
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugin(VoxMeshPlugin::default())
        .add_plugin(NoCameraPlayerPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut stdmats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(FlyCam);

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: stdmats.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        transform: Transform::from_scale((0.01, 0.01, 0.01).into())
            * Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
        mesh: assets.load("chicken.vox#model0"),
        material: assets.load("chicken.vox#material"),
        ..Default::default()
    });

    commands.spawn_bundle(PbrBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            scale: Vec3::splat(0.16),
            ..Default::default()
        },
        mesh: assets.load("materials.vox#model0"),
        material: assets.load("materials.vox#material"),
        ..Default::default()
    });

    commands.spawn_bundle(SceneBundle {
        transform: Transform {
            translation: Vec3::new(-1.0, 0.0, 0.0),
            scale: Vec3::splat(0.04),
            ..Default::default()
        },
        scene: assets.load("eggs.vox"),
        ..Default::default()
    });
}
