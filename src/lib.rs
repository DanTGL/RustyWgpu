use wgpu::{Backends, include_wgsl, util::DeviceExt};
use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::{WindowBuilder, Window},
};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
	const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
		0 => Float32x3,	// Position
		1 => Float32x3, // Color
	];

	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		use std::mem;

		wgpu::VertexBufferLayout  {
			array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::ATTRIBS,
		}
	}
}

const VERTICES: &[Vertex] = &[
	Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
	Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
	Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

struct State {
	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	size: winit::dpi::PhysicalSize<u32>,
	clear_color: wgpu::Color,
	render_pipeline: wgpu::RenderPipeline,
	vertex_buffer: wgpu::Buffer,
	num_vertices: u32,
}

impl State {
	// Creating some of the wgpu types requires async code
	async fn new(window: &Window) -> Self {
		let size = window.inner_size();

		// The instance is a handle to our GPU
		// Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { instance.create_surface(window) };
		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: Some(&surface),
				force_fallback_adapter: false,
			},
		).await.unwrap();

		// The code below might be better than the one above
		/*
		let adapter = instance
    		.enumerate_adapters(wgpu::Backends::all())
    		.filter(|adapter| {
        		// Check if this adapter supports our surface
        		!surface.get_supported_formats(&adapter).is_empty()
    		})
    		.next()
    		.unwrap();*/
		
		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),

				// WebGL doesn't support all of wgpu's features, so if
				// we're building for the web we'll have to disable some.
				limits: if cfg!(target_arch = "wasm32") {
					wgpu::Limits::downlevel_webgl2_defaults()
				} else {
					wgpu::Limits::default()
				},
				label: None
			},
			None,	// Trace path
		).await.unwrap();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_supported_formats(&adapter)[0],
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
		};
		surface.configure(&device, &config);

		let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

		let render_pipeline_layout =
			device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main", // Vertex shader entry point function
				buffers: &[ // Vertex buffers
					Vertex::desc(),
				],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main", // Fragment shader entry point function
				targets: &[Some(wgpu::ColorTargetState { // Output information
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList, // Every three vertices correspond to one triangle
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

			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0, // Use all samples
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(VERTICES),
				usage: wgpu::BufferUsages::VERTEX,
			}
		);

		let num_vertices = VERTICES.len() as u32;

		Self {
			surface,
			device,
			queue,
			config,
			size,
			clear_color: wgpu::Color::WHITE,
			render_pipeline,
			vertex_buffer,
			num_vertices,
		}

	}

	fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.size = new_size;
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			self.surface.configure(&self.device, &self.config);
		}
	}

	fn input(&mut self, event: &WindowEvent) -> bool {
		match event {
			WindowEvent::CursorMoved { position, .. } => {
				self.clear_color = wgpu::Color {
					r: position.to_logical(self.config.width as f64).x,
					g: position.to_logical(self.config.height as f64).y,
					b: 0.0,
					a: 1.0,
				};

				true
			}

			_ => false
		}
	}

	fn update(&mut self) {

	}

	fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;

		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(self.clear_color),
						store: true,
					},
				})],
				depth_stencil_attachment: None,
			});

			render_pass.set_pipeline(&self.render_pipeline);

			render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
			render_pass.draw(0..self.num_vertices, 0..1);
		}

		// Submit will accept anything that implements IntoIter
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
	

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new().build(&event_loop).unwrap();

	#[cfg(target_arch = "wasm32")] {
    	// Winit prevents sizing with CSS, so we have to set
    	// the size manually when on web.
    	use winit::dpi::PhysicalSize;
    	window.set_inner_size(PhysicalSize::new(450, 400));

    	use winit::platform::web::WindowExtWebSys;
    	web_sys::window()
    	    .and_then(|win| win.document())
    	    .and_then(|doc| {
    	        let dst = doc.get_element_by_id("wasm-example")?;
    	        let canvas = web_sys::Element::from(window.canvas());
    	        dst.append_child(&canvas).ok()?;
    	        Some(())
    	    })
    	    .expect("Couldn't append canvas to document body.");
	}

	let mut state = State::new(&window).await;

	event_loop.run(move |event, _, control_flow| match event {
		Event::WindowEvent { 
			ref event,
			window_id,
		} if window_id == window.id() => if !state.input(event) {
			match event {
				WindowEvent::CloseRequested
				| WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(VirtualKeyCode::Escape),
							..
						},
					..
				} => *control_flow = ControlFlow::Exit,

				WindowEvent::Resized(physical_size) => {
					state.resize(*physical_size);
				}

				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					// new_inner_size is &&mut so we have to dereference it twice
					state.resize(**new_inner_size);
				}

				_ => {}
			}
		},

		Event::RedrawRequested(window_id) if window_id == window.id() => {
			state.update();
			match state.render() {
				Ok(_) => {}
				// Reconfigure the surface if lost
				Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
				// The system is out of memory, we should probably quit
				Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
				// All other errors (Outdated, Timeout) should be resolved by the next frame
				Err(e) => eprintln!("{:?}", e),
			}
		}

		Event::MainEventsCleared => {
			// RedrawRequested will only trigger once, unless we manually
			// request it.
			window.request_redraw();
		}

		_ => {}
	});
}
