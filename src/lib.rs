mod texture;
mod camera;
mod camera_controller;
mod instance;
mod model;
mod resources;
mod light;
mod vertex;

use light::LightUniform;
use vertex::Vertex;
use camera_controller::CameraController;
use instance::{Instance, InstanceRaw};
use model::{DrawLight, DrawModel, Material, ModelVertex};

use wgpu::util::DeviceExt;
use cgmath::prelude::*;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{WindowBuilder, Window},
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
    render_pipeline: wgpu::RenderPipeline,
    camera: camera::Camera,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    depth_texture: texture::Texture,
    obj_model: model::Model,
    light_uniform: light::LightUniform,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { 
            #[cfg(not(target_arch="wasm32"))]
            backends: wgpu::Backends::PRIMARY, 
            #[cfg(target_arch="wasm32")]
            backends: wgpu::Backends::GL, 
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        multisampled: false, 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { 
                        multisampled: false, 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                        view_dimension: wgpu::TextureViewDimension::D2, 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let camera = camera::Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov_y: 45.0,
            near_clip_plane: 0.1,
            far_clip_plane: 100.0,
        };

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera_buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
            ],
        });

        let light_uniform = LightUniform {
            position: [2.0, 2.0, 2.0],
            _padding: 0,
            color: [1.0, 1.0, 1.0],
            _padding2: 0,
        };

        // We'll want to update our lights position, so we use COPY_DST
        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("light_vb"),
                contents: bytemuck::cast_slice(&[light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_bind_group_layout = device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: None 
                    },
                    count: None,
                },
            ],
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                }
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
                &light_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = {
            // alt
            // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
            let shader_module_descriptor = wgpu::ShaderModuleDescriptor { 
                label: Some("shader"), 
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()), 
            };

            Self::create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(texture::Texture::DEPTH_FORMAT),
                &[model::ModelVertex::descriptor(), instance::InstanceRaw::descriptor()],
                shader_module_descriptor,
            )
        };

        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("light_pipeline_layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[],
            });

            let shader_module_descriptor = wgpu::ShaderModuleDescriptor {
                label: Some("light_shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };

            Self::create_render_pipeline(
                &device, 
                &layout, 
                config.format, 
                Some(texture::Texture::DEPTH_FORMAT), 
                &[model::ModelVertex::descriptor()], 
                shader_module_descriptor,
            )
        };

        let camera_controller = CameraController::new(0.2);

        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const SPACE_BETWEEN: f32 = 3.0;
        let model_instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = cgmath::Vector3 { x, y: 0.0, z };

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can affect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = model_instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let model_instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("instance_buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let obj_model =
            resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout)
                .await
                .unwrap();

        // todo! this is way too much data in one struct lol.. it needs to be split up majorly
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            instances: model_instances,
            instance_buffer: model_instance_buffer,
            depth_texture,
            obj_model,
            light_uniform,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        color_format: wgpu::TextureFormat,
        depth_format: Option<wgpu::TextureFormat>,
        vertex_layouts: &[wgpu::VertexBufferLayout],
        shader_module_descriptor: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(shader_module_descriptor);

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: vertex_layouts,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { 
                    format: color_format, 
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }), 
                    write_mask: wgpu::ColorWrites::ALL 
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
                format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        // Make sure that if you add new instances to the Vec, you recreate the instance_buffer as well as camera_bind_group. 
        // Otherwise, your new instances won't show up correctly.

        let old_position: cgmath::Vector3<_> = self.light_uniform.position.into();
        self.light_uniform.position = (
            cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0)) * old_position
        ).into();
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));

        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }), 
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                view: &self.depth_texture.view, 
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }), 
                stencil_ops: None, 
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_pipeline(&self.light_render_pipeline);
        render_pass.draw_light_model(
            &self.obj_model, 
            &self.camera_bind_group, 
            &self.light_bind_group
        );

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw_model_instanced(
            &self.obj_model,  
            0..self.instances.len() as u32, 
            &self.camera_bind_group, 
            &self.light_bind_group
        );

        drop(render_pass); //IMPORTANT: we have to release the borrow before we finish

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")] {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));
    
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("patina")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(&window).await;

    // must use result
    let _ = event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested 
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    },
                    WindowEvent::RedrawRequested if window_id == state.window().id() => {
                        state.update();
                        match state.render() {
                            Ok(_) => {},
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    },
                    _ => {}
                }
            },
            Event::AboutToWait => {
                state.window().request_redraw();
            },
            _ => {}
        }
    });
}

