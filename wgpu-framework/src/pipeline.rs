use std::num::NonZeroU32;

macro_rules! build_field {
    ($field:ident: $field_type:ty) => {
        pub fn $field(&mut self, $field: $field_type) -> &mut Self {
            self.$field = $field.into();
            self
        }
    };
}


pub struct PipelineBuilder<'a> {
    /// The layout of bind groups for this pipeline.
    layout: Option<&'a wgpu::PipelineLayout>,
    /// The compiled vertex stage, its entry point, and the input buffers layout.
    vertex_shader: Option<wgpu::ShaderModuleDescriptor<'a>>,
    vertex_buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    /// The compiled fragment stage, its entry point, and the color targets.
    fragment_shader: Option<wgpu::ShaderModuleDescriptor<'a>>,
    color_states: Vec<Option<wgpu::ColorTargetState>>,

    /// The properties of the pipeline at the primitive assembly and rasterization level.
    primitive_topology: wgpu::PrimitiveTopology,
    front_face: wgpu::FrontFace,
    cull_mode: Option<wgpu::Face>,
    polygon_mode: wgpu::PolygonMode,
    
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    depth_stencil: Option<wgpu::DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    sample_count: u32,
    sample_mask: u64,
    alpha_to_coverage_enabled: bool,
    /// If the pipeline will be used with a multiview render pass, this indicates how many array
    /// layers the attachments will have.
    multiview: Option<NonZeroU32>,
}

impl<'a> PipelineBuilder<'a> {
    pub fn new() -> Self {
        Self {
            layout: None,
            vertex_shader: None,
            vertex_buffers: vec![],
            fragment_shader: None,
            color_states: vec![],

            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            
            depth_stencil: None,
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,

            multiview: None,
        }
    }

    build_field!(layout: &'a wgpu::PipelineLayout);
    build_field!(vertex_shader: wgpu::ShaderModuleDescriptor<'a>);
    build_field!(vertex_buffers: Vec<wgpu::VertexBufferLayout<'a>>);

    pub fn vertex_buffer(&mut self, vertex_buffer: wgpu::VertexBufferLayout<'a>) -> &mut Self {
        self.vertex_buffers.push(vertex_buffer);
        self
    }

    build_field!(fragment_shader: wgpu::ShaderModuleDescriptor<'a>);
    build_field!(color_states: Vec<Option<wgpu::ColorTargetState>>);
    
    pub fn color_state(&mut self, color_state: wgpu::ColorTargetState) -> &mut Self {
        self.color_states.push(Some(color_state));
        self
    }

    build_field!(primitive_topology: wgpu::PrimitiveTopology);
    build_field!(front_face: wgpu::FrontFace);
    build_field!(cull_mode: wgpu::Face);
    build_field!(polygon_mode: wgpu::PolygonMode);

    build_field!(depth_stencil: wgpu::DepthStencilState);
    build_field!(sample_count: u32);
    build_field!(sample_mask: u64);
    build_field!(alpha_to_coverage_enabled: bool);
    build_field!(multiview: NonZeroU32);

    pub fn build(&mut self, device: &wgpu::Device) -> Option<wgpu::RenderPipeline> {
        let layout = self.layout.unwrap();

        let vs = device.create_shader_module(self.vertex_shader.take().expect("No vertex shader supplied"));
        let fs = device.create_shader_module(self.fragment_shader.take().expect("No fragment shader supplied"));


        Some(device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: &vs,
                    entry_point: "main", // Vertex shader entry point function
                    buffers: &self.vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fs,
                    entry_point: "main", // Fragment shader entry point function
                    targets: &self.color_states,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: self.primitive_topology,
                    strip_index_format: None,
                    front_face: self.front_face,
                    cull_mode: self.cull_mode,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: self.polygon_mode,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
    
                depth_stencil: self.depth_stencil.clone(),
                multisample: wgpu::MultisampleState {
                    count: self.sample_count,
                    mask: self.sample_mask,
                    alpha_to_coverage_enabled: self.alpha_to_coverage_enabled,
                },
                multiview: self.multiview,
            }
        ))
    }


}
