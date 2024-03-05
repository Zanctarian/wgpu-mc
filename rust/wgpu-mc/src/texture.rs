use arc_swap::ArcSwap;
use std::sync::Arc;

use image::GenericImageView;
use wgpu::Extent3d;

use crate::{
    mc::{resource::ResourcePath, MinecraftState},
    render::pipeline::WmPipelines,
    WgpuState, WmRenderer,
};

pub type TextureId = u32;
pub type UV = ((u16, u16), (u16, u16));

/// Representation of a texture that has been uploaded to wgpu along with the corresponding view
#[derive(Debug)]
pub struct TextureAndView {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub format: wgpu::TextureFormat,
}

impl TextureAndView {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn from_image_file_bytes(
        wgpu_state: &WgpuState,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, anyhow::Error> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(wgpu_state, &img, Some(label))
    }

    pub fn from_image(
        wgpu_state: &WgpuState,
        img: &image::DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, anyhow::Error> {
        let rgba8 = img.to_rgba8();

        let dimensions = img.dimensions();

        Self::from_rgb_bytes(
            wgpu_state,
            &rgba8.as_raw()[..],
            Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
            label,
            wgpu::TextureFormat::Rgba8Unorm,
        )
    }

    pub fn from_rgb_bytes(
        wgpu_state: &WgpuState,
        bytes: &[u8],
        size: Extent3d,
        label: Option<&str>,
        format: wgpu::TextureFormat,
    ) -> Result<Self, anyhow::Error> {
        let texture = wgpu_state.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        if !bytes.is_empty() {
            wgpu_state.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                bytes,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * 4),
                    rows_per_image: Some(size.height),
                },
                size,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            texture,
            view,
            format,
        })
    }
}

///Texture that will be automatically resized by wgpu-mc to fit the framebuffer
#[derive(Debug, Clone)]
pub struct TextureHandle {
    pub bindable_texture: Arc<ArcSwap<BindableTexture>>,
}

///Represents a texture that has been uploaded to GPU and has an associated `BindGroup`
#[derive(Debug)]
pub struct BindableTexture {
    pub tv: Arc<TextureAndView>,
    pub bind_group: wgpu::BindGroup,
}

impl BindableTexture {
    #[must_use]
    pub fn from_tv(
        wgpu_state: &WgpuState,
        pipelines: &WmPipelines,
        tv: Arc<TextureAndView>,
        sampler: &wgpu::Sampler,
        depth: bool,
    ) -> Self {
        let bind_group = wgpu_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: pipelines
                    .bind_group_layouts
                    .read()
                    .get(if depth { "texture_depth" } else { "texture" })
                    .unwrap(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&tv.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });

        Self {
            tv,
            bind_group,
        }
    }
}