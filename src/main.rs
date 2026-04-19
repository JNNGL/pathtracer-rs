pub mod camera;
pub mod pbrt;
pub mod scene;

use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use crate::scene::MeshVertex;

struct SceneData {
    scene: scene::Scene,
    scene_bind_group: wgpu::BindGroup,
}

struct RenderPipeline {
    view_bind_group_layout: wgpu::BindGroupLayout,
    view_bind_group: wgpu::BindGroup,
    scene_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    surface_configured: bool,
    scene_data: Option<SceneData>,
    blitter: wgpu::util::TextureBlitter,
    render_pipeline: RenderPipeline,
    render_buffer_view: wgpu::TextureView,
    camera_buffer: wgpu::Buffer,
    camera: camera::Camera,
}

fn create_render_buffer(device: &wgpu::Device, format: wgpu::TextureFormat, width: u32, height: u32) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width, height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    texture.create_view(&Default::default())
}

fn create_view_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    camera_buffer: &wgpu::Buffer,
    render_buffer: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(render_buffer),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: camera_buffer.as_entire_binding()
            }
        ],
    })
}

struct Struct {
    x: i32,
    b: bool,
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<State> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::EXPERIMENTAL_RAY_QUERY,
                experimental_features: unsafe { wgpu::ExperimentalFeatures::enabled() },
                required_limits: wgpu::Limits::default().using_minimum_supported_acceleration_structure_values(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("render.wgsl"));
        let view_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let scene_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::AccelerationStructure {
                        vertex_return: false,
                    },
                    count: None,
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(&view_bind_group_layout),
                Some(&scene_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader_module,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        let render_buffer_view = create_render_buffer(&device, wgpu::TextureFormat::Rgba16Float, size.width, size.height);

        let camera = camera::Camera::new(glam::Vec3::ZERO, 0.0, 0.0, size.width as f32 / size.height as f32);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera.uniform_data()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let view_bind_group = create_view_group(&device, &view_bind_group_layout, &camera_buffer, &render_buffer_view);

        let blitter = wgpu::util::TextureBlitter::new(&device, config.format);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            surface_configured: false,
            scene_data: None,
            blitter,
            render_pipeline: RenderPipeline {
                view_bind_group_layout,
                view_bind_group,
                scene_bind_group_layout,
                pipeline,
            },
            render_buffer_view,
            camera_buffer,
            camera,
        })
    }

    fn update_camera_buffer(&self) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera.uniform_data()]));
    }

    fn recreate_view_group(&mut self) {
        self.render_buffer_view = create_render_buffer(&self.device, wgpu::TextureFormat::Rgba16Float, self.config.width, self.config.height);

        self.render_pipeline.view_bind_group = create_view_group(
            &self.device, &self.render_pipeline.view_bind_group_layout,
            &self.camera_buffer, &self.render_buffer_view);
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.surface_configured = true;

            self.camera.set_viewport(width, height);
            self.recreate_view_group();
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&Default::default());

        if self.scene_data.is_some() {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.render_pipeline.pipeline);
            pass.set_bind_group(0, &self.render_pipeline.view_bind_group, &[]);
            pass.set_bind_group(1, &self.scene_data.as_ref().unwrap().scene_bind_group, &[]);

            const WORKGROUP_SIZE: (u32, u32) = (8, 8);
            pass.dispatch_workgroups(
                self.config.width.div_ceil(WORKGROUP_SIZE.0),
                self.config.height.div_ceil(WORKGROUP_SIZE.1),
                1
            );
        }

        self.blitter.copy(&self.device, &mut encoder, &self.render_buffer_view, &view);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn update(&mut self) {
        self.update_camera_buffer()
    }

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    fn load_scene(&mut self, scene: scene::Scene) {
        self.camera.set_transform(&scene.camera_transformation);

        log::info!("creating acceleration structures...");

        let mut blases: Vec<Vec<_>> = Vec::new();

        for object in &scene.objects {
            blases.push(object.meshes.iter().map(|mesh| {
                let geometry_desc = wgpu::BlasTriangleGeometrySizeDescriptor {
                    vertex_format: wgpu::VertexFormat::Float32x3,
                    vertex_count: mesh.vertices.len() as u32,
                    index_format: Some(wgpu::IndexFormat::Uint16),
                    index_count: Some(mesh.indices.len() as u32),
                    flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
                };

                (mesh, geometry_desc.clone(), self.device.create_blas(
                    &wgpu::CreateBlasDescriptor {
                        label: None,
                        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
                    },
                    wgpu::BlasGeometrySizeDescriptors::Triangles {
                        descriptors: vec![geometry_desc],
                    },
                ))
            }).collect());
        }

        let mut tlas = self.device.create_tlas(&wgpu::CreateTlasDescriptor {
            label: None,
            flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            max_instances: scene.instances.iter().map(|i| blases[i.object].len() as u32).sum()
        });

        let mut store_index = 0;
        for instance in &scene.instances {
            for (_, _, blas) in &blases[instance.object] {
                tlas[store_index] = Some(wgpu::TlasInstance::new(
                    blas,
                    instance.transform.transpose().to_cols_array()[..12].try_into().unwrap(),
                    0, 0xFF
                ));
                store_index += 1;
            }
        }

        let mut encoder = self.device.create_command_encoder(&Default::default());

        for (mesh, geometry_desc, blas) in blases.iter().flat_map(std::convert::identity) {
            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::BLAS_INPUT,
            });

            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::BLAS_INPUT,
            });

            encoder.build_acceleration_structures(std::iter::once(&wgpu::BlasBuildEntry {
                blas,
                geometry: wgpu::BlasGeometries::TriangleGeometries(vec![
                    wgpu::BlasTriangleGeometry {
                        size: geometry_desc,
                        vertex_buffer: &vertex_buffer,
                        first_vertex: 0,
                        vertex_stride: size_of::<MeshVertex>() as u64,
                        index_buffer: Some(&index_buffer),
                        first_index: Some(0),
                        transform_buffer: None,
                        transform_buffer_offset: None,
                    }
                ]),
            }), std::iter::empty());
        }

        encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&tlas));

        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        log::info!("built {} blases and {} tlas instances.", blases.iter().map(Vec::len).sum::<usize>(), scene.instances.len());

        let scene_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.render_pipeline.scene_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::AccelerationStructure(&tlas),
                }
            ],
        });

        self.scene_data = Some(SceneData {
            scene,
            scene_bind_group,
        });
    }
}

struct Application {
    state: Option<State>,
}

impl Application {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<State> for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(State::new(window)).unwrap());

        let mut tokenizer = pbrt::Tokenizer::create_from_file(std::path::Path::new(
            "/Users/jnngl/Desktop/pbrt-v4-scenes/bmw-m6/bmw-m6.pbrt",
        ))
            .unwrap();
        let mut state = pbrt::parser::ParseState {
            working_directory: tokenizer.directory.clone(),
            ..Default::default()
        };

        let scene = state.parse(&mut tokenizer).unwrap();

        self.state.as_mut().unwrap().load_scene(scene);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) | Err(wgpu::SurfaceError::Occluded) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render: {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }

    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = Application::new();

    event_loop.run_app(&mut app)?;

    Ok(())
}
