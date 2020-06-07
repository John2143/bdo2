use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub type UserEventType = ();

mod camera;
mod texture;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    view_proj: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
    pub fn update_view_proj(&mut self, camera: &camera::Camera) {
        self.view_proj = camera.build_view_projection_matrix();
    }
}

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct InstanceRaw {
    model: cgmath::Matrix4<f32>,
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: cgmath::Matrix4::from_translation(self.position)
                * cgmath::Matrix4::from(self.rotation),
        }
    }
}

unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

impl Default for Uniforms {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity(),
        }
    }
}

#[allow(dead_code)]
struct State {
    surface: wgpu::Surface,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,

    render_pipeline: wgpu::RenderPipeline,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    diffuse_texture: texture::Texture,
    diffuse_bind_group: wgpu::BindGroup,

    size: winit::dpi::PhysicalSize<u32>,

    camera: camera::Camera,
    camera_controller: camera::CameraController,

    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,

    start_time: std::time::Instant,
    last_update: std::time::Instant,
    last_render: std::time::Instant,

    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

/*
 * 5 - - - - - 4
 * | \         | \
 * |   \       |   \
 * |     1 - - - - - 0
 * |     |     |     |
 * |     |     |     |
 * 6 - - | - - 7     |
 *   \   |       \   |
 *     \ |         \ |
 *       2 - - - - - 3
*/

const VERTICIES: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.5, 0.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, -1.0],
        tex_coords: [1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -1.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -1.0],
        tex_coords: [1.0, 1.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, // F - Top
    0, 2, 3, // F - Bot (ends in bottom left)
    2, 7, 3, // D - Top
    2, 6, 7, // D - Bot
    4, 5, 1, // U - Top
    1, 0, 4, // U - Bot
    1, 5, 6, // L - Top
    1, 6, 2, // L - Bot
    0, 3, 7, // R - Top
    0, 7, 4, // R - Bot
    5, 4, 7, // B - Top
    5, 7, 6, // B - Bot
];

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let surface = wgpu::Surface::create(window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY, // Vulkan + Metal + DX12 + Browser WebGPU
        )
        .await
        .unwrap(); // Get used to seeing this

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let diffuse_bytes = include_bytes!("test.png");
        let (diffuse_texture, cmd_buffer) =
            texture::Texture::from_bytes(&device, diffuse_bytes, "pip").unwrap();

        queue.submit(&[cmd_buffer]);

        let vertex_buffer = device
            .create_buffer_with_data(bytemuck::cast_slice(VERTICIES), wgpu::BufferUsage::VERTEX);

        let index_buffer =
            device.create_buffer_with_data(bytemuck::cast_slice(INDICES), wgpu::BufferUsage::INDEX);

        let _encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("texture_buffer_copy_encoder"),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: wgpu::TextureComponentType::Uint,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let camera_controller = camera::CameraController::new(0.02, 5.0);

        let camera = camera::Camera::new(sc_desc.width as f32 / sc_desc.height as f32);

        const NUM_INSTANCES_PER_ROW: u32 = 10;
        const NUM_INSTANCES: u32 = NUM_INSTANCES_PER_ROW * NUM_INSTANCES_PER_ROW;
        const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
            0.0,
            NUM_INSTANCES_PER_ROW as f32 * 0.5,
        );

        //make a 10 by 10 grid of objects
        let instances: Vec<Instance> = (0..NUM_INSTANCES)
            .flat_map(|z| {
                (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: (x)  as f32,
                        y: 0.0,
                        z: (z) as f32,
                    } - INSTANCE_DISPLACEMENT;

                    let rotation = cgmath::Rotation3::from_axis_angle(
                            position.clone(),
                            cgmath::Deg(0.0),
                        );

                    Instance { position, rotation }
                })
            })
            .collect();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer_size =
            instance_data.len() * std::mem::size_of::<cgmath::Matrix4<f32>>();
        let instance_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&instance_data),
            wgpu::BufferUsage::STORAGE_READ,
        );

        let mut uniforms = Uniforms::default();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&[uniforms]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::StorageBuffer {
                            dynamic: false,
                            readonly: true, //todo change
                        },
                    },
                ],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buffer,
                        range: 0..std::mem::size_of_val(&uniforms) as wgpu::BufferAddress,
                    },
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &instance_buffer,
                        range: 0..instance_buffer_size as wgpu::BufferAddress,
                    },
                },
            ],
            label: Some("uniform_bind_group"),
        });

        let vs_src = include_str!("shader.vert");
        let fs_src = include_str!("shader.frag");

        let mut compiler = shaderc::Compiler::new().unwrap();
        let vs_spirv = compiler
            .compile_into_spirv(
                vs_src,
                shaderc::ShaderKind::Vertex,
                "shader.vert",
                "main",
                None,
            )
            .unwrap();
        let fs_spirv = compiler
            .compile_into_spirv(
                fs_src,
                shaderc::ShaderKind::Fragment,
                "shader.frag",
                "main",
                None,
            )
            .unwrap();

        let vs_data = wgpu::read_spirv(std::io::Cursor::new(vs_spirv.as_binary_u8())).unwrap();
        let fs_data = wgpu::read_spirv(std::io::Cursor::new(fs_spirv.as_binary_u8())).unwrap();

        let vs_module = device.create_shader_module(&vs_data);
        let fs_module = device.create_shader_module(&fs_data);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main", // 1.
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                // 2.
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_clamp: 0.0,
                depth_bias_slope_scale: 0.0,
            }),
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],

            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let last_update = std::time::Instant::now();
        let last_render = std::time::Instant::now();
        let start_time = std::time::Instant::now();

        Self {
            surface,
            adapter,
            device,
            queue,
            sc_desc,
            swap_chain,
            render_pipeline,
            size,
            vertex_buffer,
            index_buffer,
            diffuse_texture,
            diffuse_bind_group,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            camera,
            camera_controller,
            last_update,
            last_render,
            start_time,
            instance_buffer,
            instances,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    fn input(&mut self, event: &Event<UserEventType>) -> bool {
        if self.camera_controller.process_events(&event) {
            return true;
        }

        false
    }

    fn update(&mut self) {
        let dt = self.last_update.elapsed();
        let _ups = std::time::Duration::from_secs(1).as_nanos() as f32 / (dt.as_nanos() as f32);

        self.camera_controller.update_camera(&mut self.camera, dt);
        self.uniforms.update_view_proj(&self.camera);

        // Copy operation's are performed on the gpu, so we'll need
        // a CommandEncoder for that
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("update encoder"),
            });

        let staging_buffer = self.device.create_buffer_with_data(
            bytemuck::cast_slice(&[self.uniforms]),
            wgpu::BufferUsage::COPY_SRC,
        );

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uniform_buffer,
            0,
            std::mem::size_of::<Uniforms>() as wgpu::BufferAddress,
        );

        // We need to remember to submit our CommandEncoder's output
        // otherwise we won't see any change.
        self.queue.submit(&[encoder.finish()]);

        self.last_update = std::time::Instant::now();
    }

    fn render(&mut self) {
        let dt = self.last_render.elapsed();
        let _fps = std::time::Duration::from_secs(1).as_nanos() as f32 / (dt.as_nanos() as f32);

        std::thread::spawn(move || println!("{} fps", _fps));

        let frame = self.swap_chain.get_next_texture().unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.0,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..100);

        drop(render_pass);

        self.queue.submit(&[encoder.finish()]);
        self.last_render = std::time::Instant::now();
    }
}

#[tokio::main]
async fn main() -> () {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title("bdo2");

    let mut state = State::new(&window).await;
    let mut has_focus = true;
    window.set_cursor_grab(has_focus);
    window.set_cursor_visible(!has_focus);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                state.update();
                state.render();
                return;
            }
            Event::MainEventsCleared => {
                window.request_redraw();
                return;
            }
            _ => {}
        }

        if has_focus && state.input(&event) {
            return;
        }

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    } = input
                    {
                        *control_flow = ControlFlow::Exit
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size);
                }
                WindowEvent::Focused(focus) => {
                    has_focus = *focus;
                    window.set_cursor_grab(has_focus);
                    window.set_cursor_visible(!has_focus);
                }
                _ => {}
            },
            _ => {}
        }
    });
}
