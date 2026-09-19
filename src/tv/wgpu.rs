use std::sync::Arc;

use thiserror::Error;
use wgpu::{
    Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, Color, CreateSurfaceError,
    CurrentSurfaceTexture::{Lost, Occluded, Outdated, Suboptimal, Success, Timeout, Validation},
    Device, Extent3d, FilterMode, FragmentState, Instance, LoadOp, Operations, PipelineLayout,
    PipelineLayoutDescriptor, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterError, RequestAdapterOptions,
    RequestDeviceError, SamplerBindingType, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StoreOp, Surface, SurfaceConfiguration, TexelCopyBufferLayout, Texture,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDimension,
    VertexState,
    wgt::{DeviceDescriptor, SamplerDescriptor, TextureDescriptor},
};
use winit::{dpi::PhysicalSize, window::Window};

use crate::{
    frame::{FRAME_HEIGHT, FRAME_WIDTH, Frame},
    tv::TV,
};

pub type WgpuResult<T> = Result<T, WgpuError>;

#[derive(Debug, Error)]
pub enum WgpuError {
    #[error("failed to create rendering surface: {0}")]
    CreateSurface(#[from] CreateSurfaceError),

    #[error("failed to request GPU adapter: {0}")]
    RequestAdapter(#[from] RequestAdapterError),

    #[error("failed to request GPU device: {0}")]
    RequestDevice(#[from] RequestDeviceError),

    #[error("no supported surface configuration")]
    UnsupportedSurfaceConfiguration,

    #[error("no rendering surface is available")]
    MissingSurface,

    #[error("surface acquisition failed validation")]
    SurfaceValidation,
}

pub struct WgpuTV {
    window: Arc<Window>,
    surface: Option<Surface<'static>>,
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    shader: ShaderModule,

    frame_texture: Texture,
    frame_pixels: Vec<u8>,
    frame_bind_group: BindGroup,
    frame_pipeline_layout: PipelineLayout,
    frame_pipeline: RenderPipeline,
}

impl WgpuTV {
    pub async fn new(window: Window) -> WgpuResult<Self> {
        let frame_pixels = vec![255; FRAME_WIDTH * FRAME_HEIGHT * 4];
        let window = Arc::new(window);

        let instance = Instance::default();
        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("NES display"),
                ..Default::default()
            })
            .await?;

        let size = window.inner_size();

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or(WgpuError::UnsupportedSurfaceConfiguration)?;

        if size.width > 0 && size.height > 0 {
            surface.configure(&device, &config);
        }

        let frame_texture = device.create_texture(&TextureDescriptor {
            label: Some("NES frame"),
            size: Extent3d {
                width: FRAME_WIDTH as u32,
                height: FRAME_HEIGHT as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let frame_view = frame_texture.create_view(&Default::default());

        let frame_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("NES frame sampler"),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let frame_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("NES frame bindings"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let frame_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("NES frame"),
            layout: &frame_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&frame_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&frame_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("NES frame shader"),
            source: ShaderSource::Wgsl(include_str!("frame.wgsl").into()),
        });

        let frame_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("NES frame pipeline layout"),
            bind_group_layouts: &[Some(&frame_bind_group_layout)],
            ..Default::default()
        });
        let frame_pipeline =
            Self::create_frame_pipeline(&device, &shader, &frame_pipeline_layout, config.format);

        Ok(Self {
            window,
            surface: Some(surface),
            instance,
            adapter,
            device,
            queue,
            config,
            shader,

            frame_texture,
            frame_pixels,
            frame_bind_group,
            frame_pipeline_layout,
            frame_pipeline,
        })
    }

    pub fn window(&self) -> &Window {
        self.window.as_ref()
    }

    fn create_frame_pipeline(
        device: &Device,
        shader: &ShaderModule,
        layout: &PipelineLayout,
        format: TextureFormat,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("NES frame pipeline"),
            layout: Some(layout),
            vertex: VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: shader,
                entry_point: Some(if format.is_srgb() {
                    "fs_main"
                } else {
                    "fs_unorm"
                }),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            multiview_mask: None,
            cache: None,
        })
    }

    fn upload_frame(&mut self, frame: &Frame) {
        for (rgb, rgba) in frame
            .data()
            .chunks_exact(3)
            .zip(self.frame_pixels.chunks_exact_mut(4))
        {
            rgba[..3].copy_from_slice(rgb);
            rgba[3] = 255;
        }

        self.queue.write_texture(
            self.frame_texture.as_image_copy(),
            &self.frame_pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((FRAME_WIDTH * 4) as u32),
                rows_per_image: Some(FRAME_HEIGHT as u32),
            },
            self.frame_texture.size(),
        );
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;

        if size.width == 0 || size.height == 0 {
            return;
        }

        let Some(surface) = &self.surface else {
            return;
        };

        surface.configure(&self.device, &self.config);
    }

    fn recreate_surface(&mut self) -> WgpuResult<()> {
        drop(self.surface.take());

        let surface = self.instance.create_surface(Arc::clone(&self.window))?;
        let size = self.window.inner_size();
        let config = surface
            .get_default_config(&self.adapter, size.width, size.height)
            .ok_or(WgpuError::UnsupportedSurfaceConfiguration)?;

        if size.width > 0 && size.height > 0 {
            surface.configure(&self.device, &config);
        }

        if config.format != self.config.format {
            self.frame_pipeline = Self::create_frame_pipeline(
                &self.device,
                &self.shader,
                &self.frame_pipeline_layout,
                config.format,
            );
        }

        self.config = config;
        self.surface = Some(surface);

        Ok(())
    }
}

impl TV for WgpuTV {
    type Error = WgpuError;

    fn present(&mut self, frame: &Frame) -> WgpuResult<()> {
        let surface = self.surface.as_ref().ok_or(WgpuError::MissingSurface)?;

        if self.config.width == 0 || self.config.height == 0 {
            return Ok(());
        }

        let (output, reconfigure) = match surface.get_current_texture() {
            Success(output) => (output, false),
            Suboptimal(output) => (output, true),
            Timeout | Occluded => return Ok(()),
            Outdated => {
                self.resize(self.window.inner_size());
                return Ok(());
            }
            Lost => {
                self.recreate_surface()?;
                return Ok(());
            }
            Validation => {
                return Err(WgpuError::SurfaceValidation);
            }
        };

        self.upload_frame(frame);

        let view = output.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        let window_width = self.config.width as f32;
        let window_height = self.config.height as f32;

        let fit_scale =
            (window_width / FRAME_WIDTH as f32).min(window_height / FRAME_HEIGHT as f32);

        let scale = if fit_scale >= 1.0 {
            fit_scale.floor()
        } else {
            fit_scale
        };

        let width = FRAME_WIDTH as f32 * scale;
        let height = FRAME_HEIGHT as f32 * scale;

        let x = ((window_width - width) / 2.0).floor();
        let y = ((window_height - height) / 2.0).floor();

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("NES display"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            pass.set_viewport(x, y, width, height, 0.0, 1.0);
            pass.set_pipeline(&self.frame_pipeline);
            pass.set_bind_group(0, &self.frame_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        self.window.pre_present_notify();
        self.queue.present(output);

        if reconfigure {
            drop(view);
            self.resize(self.window.inner_size());
        }

        Ok(())
    }
}
