pub mod camera;
pub mod pbrt;
pub mod scene;

use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};
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
    camera_controller: camera::CameraController,
    last_update: Instant,
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
                required_limits: wgpu::Limits {
                    max_buffer_size: 1 << 32,
                    ..wgpu::Limits::default()
                }.using_minimum_supported_acceleration_structure_values(),
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

        let camera = camera::Camera::new(glam::Vec3::ZERO, 0.0, 0.0, 0.0, size.width as f32 / size.height as f32);
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
            camera_controller: camera::CameraController::new(6.0, 0.0025),
            last_update: Instant::now(),
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
        let now = Instant::now();
        let delta_time = now - self.last_update;
        self.last_update = now;

        self.camera_controller.update_camera(&mut self.camera, delta_time);
        self.update_camera_buffer()
    }

    fn set_mouse_capture(&mut self, captured: bool) {
        if captured {
            let grab_result = self
                .window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));

            if let Err(error) = grab_result {
                log::warn!("unable to grab cursor: {}", error);
                return;
            }
        } else if let Err(error) = self.window.set_cursor_grab(CursorGrabMode::None) {
            log::warn!("unable to release cursor grab: {}", error);
        }

        self.window.set_cursor_visible(!captured);
        self.camera_controller.set_mouse_captured(captured);
    }

    fn handle_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.camera_controller
            .process_mouse_motion(&mut self.camera, delta_x, delta_y);
    }

    fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left && state == ElementState::Pressed {
            self.set_mouse_capture(true);
        }
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if self.camera_controller.set_key_pressed(code, is_pressed) {
            return;
        }

        if !is_pressed {
            return;
        }

        match code {
            KeyCode::Escape if self.camera_controller.mouse_captured() => self.set_mouse_capture(false),
            KeyCode::Escape => event_loop.exit(),
            _ => {}
        }
    }

    fn load_scene(&mut self, scene: scene::Scene) {
        struct MeshBuildInput {
            size: wgpu::BlasTriangleGeometrySizeDescriptor,
            vertex_buffer: wgpu::Buffer,
            index_buffer: wgpu::Buffer,
            blas: wgpu::Blas,
            object_index: usize,
        }

        self.camera.set_transform(&scene.camera_transformation);
        self.camera.set_fov(scene.camera_fov);
        self.last_update = Instant::now();

        log::info!("creating acceleration structures...");

        let mut mesh_build_inputs = Vec::new();

        for (object_index, object) in scene.objects.iter().enumerate() {
            for mesh in &object.meshes {
                let size = wgpu::BlasTriangleGeometrySizeDescriptor {
                    vertex_format: wgpu::VertexFormat::Float32x3,
                    vertex_count: mesh.vertices.len() as u32,
                    index_format: Some(wgpu::IndexFormat::Uint32),
                    index_count: Some(mesh.indices.len() as u32),
                    flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
                };

                let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("BLAS Vertex Buffer"),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::BLAS_INPUT,
                });

                let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("BLAS Index Buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::BLAS_INPUT,
                });

                let blas = self.device.create_blas(
                    &wgpu::CreateBlasDescriptor {
                        label: None,
                        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
                        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
                    },
                    wgpu::BlasGeometrySizeDescriptors::Triangles {
                        descriptors: vec![size.clone()],
                    },
                );

                mesh_build_inputs.push(MeshBuildInput {
                    size,
                    vertex_buffer,
                    index_buffer,
                    blas,
                    object_index,
                });
            }
        }

        let mut tlas = self.device.create_tlas(&wgpu::CreateTlasDescriptor {
            label: None,
            flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
            update_mode: wgpu::AccelerationStructureUpdateMode::Build,
            max_instances: scene.instances.iter().map(
                |instance| mesh_build_inputs.iter()
                    .filter(|input| input.object_index == instance.object)
                    .count() as u32).sum(),
        });

        let mut store_index = 0;
        for instance in &scene.instances {
            for build_input in mesh_build_inputs
                .iter()
                .filter(|build_input| build_input.object_index == instance.object)
            {
                tlas[store_index] = Some(wgpu::TlasInstance::new(
                    &build_input.blas,
                    instance.transform.transpose().to_cols_array()[..12].try_into().unwrap(),
                    0, 0xFF
                ));
                store_index += 1;
            }
        }

        let mut encoder = self.device.create_command_encoder(&Default::default());
        let error_scope = self.device.push_error_scope(wgpu::ErrorFilter::Validation);

        let blas_build_entries: Vec<_> = mesh_build_inputs
            .iter()
            .map(|build_input| wgpu::BlasBuildEntry {
                blas: &build_input.blas,
                geometry: wgpu::BlasGeometries::TriangleGeometries(vec![
                    wgpu::BlasTriangleGeometry {
                        size: &build_input.size,
                        vertex_buffer: &build_input.vertex_buffer,
                        first_vertex: 0,
                        vertex_stride: size_of::<MeshVertex>() as u64,
                        index_buffer: Some(&build_input.index_buffer),
                        first_index: Some(0),
                        transform_buffer: None,
                        transform_buffer_offset: None,
                    }
                ]),
            })
            .collect();

        encoder.build_acceleration_structures(blas_build_entries.iter(), std::iter::once(&tlas));

        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

        if let Some(error) = pollster::block_on(error_scope.pop()) {
            log::error!("acceleration structure build failed validation: {}", error);
            return;
        }

        log::info!(
            "built {} blases and {} tlas instances.",
            mesh_build_inputs.len(),
            scene.instances.len()
        );

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
            "/Users/jnngl/Desktop/pbrt-v4-scenes/kroken/camera-1.pbrt",
        )).unwrap();
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
            WindowEvent::Focused(false) => {
                state.camera_controller.clear_input();
                state.set_mouse_capture(false);
            }
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
            WindowEvent::MouseInput { state: mouse_state, button, .. } => {
                state.handle_mouse_button(button, mouse_state)
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        if let DeviceEvent::MouseMotion { delta } = event {
            state.handle_mouse_motion(delta.0, delta.1);
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