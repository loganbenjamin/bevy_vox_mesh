use bevy::asset::{LoadContext, LoadedAsset};
use bevy::math::Vec4;
use bevy::pbr::{AlphaMode, StandardMaterial};
use bevy::render::{
    color::Color,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    texture::Image
};
use dot_vox::Material;
use num_integer::Roots;

// constants used in magicavoxel's material property map
const MATERIAL_TYPE: &str = "_type";
const MATERIAL_ALPHA: &str = "_alpha";
const MATERIAL_GLASS: &str = "_glass";
const MATERIAL_EMIT: &str = "_emit";

pub(crate) fn material_textures_width(palette: &[[u8; 4]]) -> usize {
    palette.len().sqrt().next_power_of_two()
}

pub(crate) fn load_material(ctx: &mut LoadContext, palette: &[[u8; 4]], materials: &[Material]) {
    if !materials.is_empty() {
        assert_eq!(palette.len(), materials.len(), "Expected a material for every palette color");
    }

    let texture_width = material_textures_width(palette);
    let texture_size = texture_width * texture_width;

    let (palette_texture, opaque) = palette_texture(palette, materials, texture_width, texture_size);
    let palette_texture_handle = ctx.set_labeled_asset(
        "base_color_texture",
        LoadedAsset::new(palette_texture)
    );

    let emissive_texture = emissive_texture(palette, materials, texture_width, texture_size);
    let emissive = emissive_texture.is_some();
    let emissive_texture_handle = emissive_texture.map(|x| {
        ctx.set_labeled_asset(
            "emissive_texture",
            LoadedAsset::new(x)
        )
    });

    ctx.set_labeled_asset("material", LoadedAsset::new(StandardMaterial {
        base_color_texture: Some(palette_texture_handle),
        emissive: if emissive { Color::WHITE } else { Color::BLACK },
        emissive_texture: emissive_texture_handle,
        alpha_mode: if opaque { AlphaMode::Opaque } else { AlphaMode::Blend },
        ..StandardMaterial::default()
    }));
}

fn palette_texture(palette: &[[u8; 4]], materials: &[Material], texture_width: usize, texture_size: usize) -> (Image, bool) {
    let mut opaque = true;

    let mut data = Vec::with_capacity(texture_size * 4);
    for (i, color) in palette.iter().enumerate() {
        if let Some(material) = materials.get(i) {
            if material.properties.get(MATERIAL_TYPE).filter(|x| *x == MATERIAL_GLASS).is_some() {
                if let Some(a) = material.properties.get(MATERIAL_ALPHA).and_then(|x| x.parse::<f32>().ok()) {
                    let alpha = (a * u8::MAX as f32).round() as u8;
                    data.extend_from_slice(&[color[0], color[1], color[2], alpha]);
                    opaque = false;
                    continue;
                }
            }
        }
        data.extend_from_slice(color);
    }
    for _ in palette.len()..texture_size {
        data.extend_from_slice(&[0; 4]);
    }

    let texture = Image::new(
        Extent3d {
            width: texture_width as u32,
            height: texture_width as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
    );
    (texture, opaque)
}

fn emissive_texture(palette: &[[u8; 4]], materials: &[Material], texture_width: usize, texture_size: usize) -> Option<Image> {
    if materials.is_empty() {
        return None;
    }

    let mut data = Vec::with_capacity(texture_size * 4);
    let mut is_lit = false;

    for (i, material) in materials.iter().enumerate() {
        if material.properties.get(MATERIAL_TYPE).filter(|x| *x == MATERIAL_EMIT).is_some() {
            if let Some(emit) = material.properties.get(MATERIAL_EMIT).and_then(|x| x.parse::<f32>().ok()) {
                let color: Vec4 = palette[i].map(|x| x as f32 / u8::MAX as f32).into();
                let emission = color.truncate() * emit;
                data.extend_from_slice(&emission.as_ref().map(|x| { (x * u8::MAX as f32).round() as u8 }));
                data.push(1);
                is_lit = true;
                continue;
            }
        }
        data.extend_from_slice(&[0; 4]);
    }
    for _ in materials.len()..texture_size {
        data.extend_from_slice(&[0; 4]);
    }

    if is_lit {
        Some(Image::new(
            Extent3d {
                width: texture_width as u32,
                height: texture_width as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
        ))
    } else {
        None
    }
}
