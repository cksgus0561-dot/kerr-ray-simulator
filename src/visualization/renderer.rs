//! Persistent wgpu buffers: camera/playback write only a small uniform buffer.
use super::{
    camera,
    detector_view::DetectorView,
    geometry::{self, Segment},
};
use crate::{
    session::{DetectorMode, PlaybackMode, SessionConfig},
    simulation::SimulationData,
};
use eframe::{
    egui,
    egui_wgpu::{self, wgpu},
};
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniform {
    matrix: [[f32; 4]; 4],
    viewport: [f32; 4],
    times: [f32; 4],
    flags: [u32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayVertex {
    position: [f32; 3],
    uv: [f32; 2],
}
struct Batch {
    buffer: wgpu::Buffer,
    count: u32,
}
impl Batch {
    fn new<T: bytemuck::Pod>(device: &wgpu::Device, label: &str, values: &[T]) -> Self {
        let buffer = if values.is_empty() {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: 16,
                usage: wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        } else {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(values),
                usage: wgpu::BufferUsages::VERTEX,
            })
        };
        Self {
            buffer,
            count: values.len() as u32,
        }
    }
}
pub struct Renderer {
    pub state: egui_wgpu::RenderState,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    line_pipeline: wgpu::RenderPipeline,
    tip_pipeline: wgpu::RenderPipeline,
    surface_pipeline: wgpu::RenderPipeline,
    overlay_pipeline: wgpu::RenderPipeline,
    lines: Batch,
    rays: Batch,
    hits: Batch,
    field: Batch,
    free_fall: Batch,
    free_fall_capacity: usize,
    surface: Batch,
    overlay_vertices: Batch,
    color: wgpu::Texture,
    color_view: wgpu::TextureView,
    depth: wgpu::TextureView,
    pub texture_id: egui::TextureId,
    pub size: [u32; 2],
    overlay: wgpu::Texture,
    overlay_bind: wgpu::BindGroup,
    overlay_layout: wgpu::BindGroupLayout,
    pub detector_texture_id: egui::TextureId,
    overlay_size: [usize; 2],
    scene: Option<Arc<SimulationData>>,
    ray_key: (usize, Option<usize>),
    grid_key: (u32, u32),
    field_key: String,
    hit_key: String,
    pub rendered_ids: Vec<usize>,
    pub bounds: (glam::Vec3, glam::Vec3),
    pub grid_values: [f32; 2],
    pub max_omega: f64,
    pub ray_uploads: u64,
    pub geometry_uploads: u64,
    pub last_prepare_ms: f64,
}
impl Renderer {
    pub fn new(state: egui_wgpu::RenderState) -> Self {
        let device = &state.device;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Kerr coordinate scene (no physics in shader)"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scene.wgsl").into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene uniforms"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera and physical display time"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let overlay_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("count texture"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let texture_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout), Some(&overlay_layout)],
            immediate_size: 0,
        });
        let segment_attrs =
            wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x4,3=>Uint32x4];
        let surface_attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Uint32,2=>Float32x4];
        let overlay_attrs = wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x2];
        let pipeline = |entry: &str,
                        fragment: &str,
                        stride: u64,
                        attributes: &[wgpu::VertexAttribute],
                        instanced: bool,
                        depth_write: bool,
                        textured: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(entry),
                layout: Some(if textured {
                    &texture_layout
                } else {
                    &pipeline_layout
                }),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: stride,
                        step_mode: if instanced {
                            wgpu::VertexStepMode::Instance
                        } else {
                            wgpu::VertexStepMode::Vertex
                        },
                        attributes,
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(fragment),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let line_pipeline = pipeline(
            "vs_line",
            "fs_color",
            64,
            &segment_attrs,
            true,
            false,
            false,
        );
        let tip_pipeline = pipeline("vs_tip", "fs_color", 64, &segment_attrs, true, false, false);
        let surface_pipeline = pipeline(
            "vs_surface",
            "fs_color",
            32,
            &surface_attrs,
            false,
            true,
            false,
        );
        let overlay_pipeline = pipeline(
            "vs_overlay",
            "fs_overlay",
            20,
            &overlay_attrs,
            false,
            false,
            true,
        );
        let color = texture(
            device,
            [16, 16],
            wgpu::TextureFormat::Rgba8Unorm,
            "scene framebuffer",
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
        );
        let color_view = color.create_view(&Default::default());
        let depth = texture(
            device,
            [16, 16],
            wgpu::TextureFormat::Depth32Float,
            "scene depth",
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        )
        .create_view(&Default::default());
        let overlay = texture(
            device,
            [1, 1],
            wgpu::TextureFormat::Rgba8Unorm,
            "all-ray detector counts",
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        );
        let overlay_view = overlay.create_view(&Default::default());
        let overlay_bind = texture_bind(device, &overlay_layout, &overlay_view);
        let texture_id = state.renderer.write().register_native_texture(
            device,
            &color_view,
            wgpu::FilterMode::Linear,
        );
        let detector_texture_id = state.renderer.write().register_native_texture(
            device,
            &overlay_view,
            wgpu::FilterMode::Nearest,
        );
        let empty = || Batch::new::<Segment>(device, "empty", &[]);
        Self {
            uniform,
            bind,
            line_pipeline,
            tip_pipeline,
            surface_pipeline,
            overlay_pipeline,
            lines: empty(),
            rays: empty(),
            hits: empty(),
            field: empty(),
            free_fall: empty(),
            free_fall_capacity: 0,
            surface: empty(),
            overlay_vertices: empty(),
            color,
            color_view,
            depth,
            texture_id,
            size: [16, 16],
            overlay,
            overlay_bind,
            overlay_layout,
            detector_texture_id,
            overlay_size: [1, 1],
            scene: None,
            ray_key: (0, None),
            grid_key: (0, 0),
            field_key: String::new(),
            hit_key: String::new(),
            rendered_ids: vec![],
            bounds: (glam::Vec3::ZERO, glam::Vec3::ONE),
            grid_values: [1., 1.],
            max_omega: 0.,
            ray_uploads: 0,
            geometry_uploads: 0,
            last_prepare_ms: 0.,
            state,
        }
    }
    pub fn prepare(
        &mut self,
        data: Arc<SimulationData>,
        cfg: &SessionConfig,
        selected: Option<usize>,
    ) {
        let start = std::time::Instant::now();
        let new = self
            .scene
            .as_ref()
            .is_none_or(|old| !Arc::ptr_eq(old, &data));
        let ray_key = (cfg.view.max_rendered_trajectories, selected);
        if new || ray_key != self.ray_key {
            self.rendered_ids = geometry::subset(&data, ray_key.0, selected);
            self.bounds = geometry::bounds(&data, &self.rendered_ids);
            self.rays = Batch::new(
                &self.state.device,
                "cached actual geodesic segments",
                &geometry::ray_segments(&data, &self.rendered_ids),
            );
            self.ray_key = ray_key;
            self.ray_uploads += 1;
        }
        let extent = if cfg.view.grid_auto {
            self.bounds.0.abs().max(self.bounds.1.abs()).max_element()
        } else {
            cfg.view.grid_extent
        };
        let spacing = if cfg.view.grid_auto {
            geometry::nice_spacing(extent)
        } else {
            cfg.view.grid_spacing.max(extent / 20.)
        };
        self.grid_values = [extent, spacing];
        let grid_key = (extent.to_bits(), spacing.to_bits());
        if new || grid_key != self.grid_key {
            let (lines, surface) = geometry::static_geometry(&data, extent, spacing);
            self.lines = Batch::new(
                &self.state.device,
                "cached coordinate grid planes launches",
                &lines,
            );
            self.surface = Batch::new(&self.state.device, "exact horizon geometry", &surface);
            self.grid_key = grid_key;
            self.geometry_uploads += 1;
        }
        let field_key = format!(
            "{} {} {} {} {}",
            data.experiment.spin,
            cfg.view.frame_dragging_density,
            cfg.view.field_3d,
            cfg.view.field_extent,
            cfg.view.field_time_scale
        );
        if new || self.field_key != field_key {
            let (lines, maximum) = geometry::field_segments(
                crate::physics::kerr::Kerr::new(data.experiment.spin).unwrap(),
                &cfg.view,
            );
            self.field = Batch::new(&self.state.device, "metric-derived ZAMO field", &lines);
            self.max_omega = maximum;
            self.field_key = field_key;
            self.geometry_uploads += 1;
        }
        let hit_key = serde_json::to_string(&cfg.experiment.postprocess).unwrap();
        if (new || hit_key != self.hit_key)
            && let Ok(bins) = cfg.experiment.postprocess.resolve(&data.events)
        {
            self.hits = Batch::new(
                &self.state.device,
                "all detector hit events",
                &geometry::hit_segments(&data, bins),
            );
            self.hit_key = hit_key;
        }
        if new {
            let c = geometry::plane_corners(&data.experiment.detector.plane);
            let corner = [
                OverlayVertex {
                    position: c[0].to_array(),
                    uv: [0., 1.],
                },
                OverlayVertex {
                    position: c[1].to_array(),
                    uv: [1., 1.],
                },
                OverlayVertex {
                    position: c[2].to_array(),
                    uv: [1., 0.],
                },
                OverlayVertex {
                    position: c[3].to_array(),
                    uv: [0., 0.],
                },
            ];
            self.overlay_vertices = Batch::new(
                &self.state.device,
                "detector world quad",
                &[
                    corner[0], corner[1], corner[2], corner[0], corner[2], corner[3],
                ],
            );
            self.geometry_uploads += 1;
        }
        self.scene = Some(data);
        self.last_prepare_ms = start.elapsed().as_secs_f64() * 1000.;
    }
    /// Animated marker vertices are interpolated from the CPU cache. Reuse the
    /// allocation; cameras/uniforms never rebuild either cache or this buffer.
    pub fn update_free_fall(&mut self, segments: &[Segment]) {
        if segments.len() > self.free_fall_capacity {
            self.free_fall_capacity = segments.len().next_power_of_two();
            self.free_fall.buffer = self.state.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("cached free-fall lattice topology / animated vertices"),
                size: (self.free_fall_capacity * std::mem::size_of::<Segment>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        self.free_fall.count = segments.len() as u32;
        if !segments.is_empty() {
            self.state.queue.write_buffer(
                &self.free_fall.buffer,
                0,
                bytemuck::cast_slice(segments),
            );
        }
    }
    pub fn update_detector(&mut self, view: &DetectorView) {
        if self.overlay_size != view.resolution {
            self.overlay = texture(
                &self.state.device,
                view.resolution.map(|v| v as u32),
                wgpu::TextureFormat::Rgba8Unorm,
                "all-ray detector counts",
                wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            );
            let tv = self.overlay.create_view(&Default::default());
            self.overlay_bind = texture_bind(&self.state.device, &self.overlay_layout, &tv);
            self.state
                .renderer
                .write()
                .update_egui_texture_from_wgpu_texture(
                    &self.state.device,
                    &tv,
                    wgpu::FilterMode::Nearest,
                    self.detector_texture_id,
                );
            self.overlay_size = view.resolution;
        }
        self.state.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.overlay,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &view.rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(view.resolution[0] as u32 * 4),
                rows_per_image: Some(view.resolution[1] as u32),
            },
            wgpu::Extent3d {
                width: view.resolution[0] as u32,
                height: view.resolution[1] as u32,
                depth_or_array_layers: 1,
            },
        );
    }
    pub fn render(
        &mut self,
        size: [u32; 2],
        cfg: &SessionConfig,
        time: f64,
        bin: [f64; 2],
        selected: Option<usize>,
    ) {
        let size = size.map(|s| s.clamp(16, 8192));
        let device = &self.state.device;
        if self.size != size {
            self.color = texture(
                device,
                size,
                wgpu::TextureFormat::Rgba8Unorm,
                "scene framebuffer",
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            );
            self.color_view = self.color.create_view(&Default::default());
            self.depth = texture(
                device,
                size,
                wgpu::TextureFormat::Depth32Float,
                "scene depth",
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
            .create_view(&Default::default());
            self.state
                .renderer
                .write()
                .update_egui_texture_from_wgpu_texture(
                    device,
                    &self.color_view,
                    wgpu::FilterMode::Linear,
                    self.texture_id,
                );
            self.size = size;
        }
        let mask = cfg
            .view
            .visibility
            .iter()
            .enumerate()
            .fold(0, |m, (i, &b)| m | ((b as u32) << i))
            | ((cfg.view.free_fall.visible as u32) << 12);
        let uniform = Uniform {
            matrix: camera::matrix(&cfg.camera, size[0] as f32 / size[1] as f32).to_cols_array_2d(),
            viewport: [
                size[0] as f32,
                size[1] as f32,
                cfg.view.trajectory_thickness,
                3.0,
            ],
            times: [
                time as f32,
                bin[0] as f32,
                bin[1] as f32,
                match cfg.view.playback_mode {
                    PlaybackMode::Static => 0.,
                    PlaybackMode::Propagation => 1.,
                    PlaybackMode::Detector => 2.,
                },
            ],
            flags: [
                mask,
                selected.map_or(u32::MAX, |id| id as u32),
                match cfg.view.detector_mode {
                    DetectorMode::Accumulated => 0,
                    DetectorMode::Instantaneous => 1,
                    DetectorMode::Cumulative => 2,
                },
                0,
            ],
        };
        self.state
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&uniform));
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Kerr scene"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D depth + batched lines"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.color_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.018,
                            g: 0.028,
                            b: 0.047,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_bind_group(0, &self.bind, &[]);
            pass.set_pipeline(&self.surface_pipeline);
            if self.surface.count > 0 {
                pass.set_vertex_buffer(0, self.surface.buffer.slice(..));
                pass.draw(0..self.surface.count, 0..1);
            }
            pass.set_pipeline(&self.line_pipeline);
            for (batch, visible) in [
                (&self.lines, true),
                (&self.rays, cfg.view.visibility[8]),
                (&self.hits, cfg.view.visibility[9]),
                (&self.field, cfg.view.visibility[10]),
                (&self.free_fall, cfg.view.free_fall.visible),
            ] {
                if visible && batch.count > 0 {
                    pass.set_vertex_buffer(0, batch.buffer.slice(..));
                    pass.draw(0..6, 0..batch.count);
                }
            }
            if self.rays.count > 0
                && cfg.view.visibility[11]
                && cfg.view.playback_mode != PlaybackMode::Static
            {
                pass.set_pipeline(&self.tip_pipeline);
                pass.set_vertex_buffer(0, self.rays.buffer.slice(..));
                pass.draw(0..6, 0..self.rays.count);
            }
            if cfg.view.detector_overlay
                && cfg.view.visibility[7]
                && self.overlay_vertices.count > 0
            {
                pass.set_pipeline(&self.overlay_pipeline);
                pass.set_bind_group(1, &self.overlay_bind, &[]);
                pass.set_vertex_buffer(0, self.overlay_vertices.buffer.slice(..));
                pass.draw(0..6, 0..1);
            }
        }
        self.state.queue.submit([encoder.finish()]);
    }
}
fn texture(
    device: &wgpu::Device,
    size: [u32; 2],
    format: wgpu::TextureFormat,
    label: &str,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}
fn texture_bind(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("nearest count sampling"),
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}
