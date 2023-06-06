
use std::ops::Range;
use std::sync::Arc;
use std::time::Instant;

use image::{ImageBuffer, Rgba, DynamicImage, GrayImage};

use vulkano::pipeline::graphics::vertex_input::{VertexBufferDescription, VertexInputState, Vertex as VertexTrait};
use vulkano::pipeline::graphics::viewport::{ViewportState, Viewport};
use vulkano::render_pass::{Subpass, RenderPass, Framebuffer, FramebufferCreateInfo};
use vulkano::sampler::{Sampler, SamplerCreateInfo, Filter, SamplerMipmapMode};
use vulkano::shader::ShaderModule;
use vulkano::{VulkanLibrary, swapchain, format};
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::{Instance, InstanceCreateInfo};

use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CopyImageToBufferInfo, CommandBufferUsage, RenderPassBeginInfo, SubpassContents, PrimaryCommandBufferAbstract};

use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::descriptor_set::{WriteDescriptorSet, PersistentDescriptorSet};

use vulkano::device::{Device, Queue, DeviceCreateInfo, QueueCreateInfo, QueueFlags, DeviceExtensions};

use vulkano::pipeline::{PipelineBindPoint, ComputePipeline, Pipeline, GraphicsPipeline};

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer, BufferContents};

use vulkano::memory::allocator::{AllocationCreateInfo, MemoryUsage, StandardMemoryAllocator, MemoryAllocator, GenericMemoryAllocator};

// mandelbrot testing
use vulkano::image::{ImageDimensions, StorageImage, SwapchainImage, ImageUsage, ImageAccess};
use vulkano::image::view::ImageView;
use vulkano::format::Format;

use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::{self, GpuFuture};
use vulkano_util::context::VulkanoContext;
use vulkano_util::renderer::{DEFAULT_IMAGE_FORMAT, VulkanoWindowRenderer};
use vulkano_util::window::VulkanoWindows;
use vulkano_win::create_surface_from_winit;
use winit::window::Window;

use super::helper;

use crate::engine::camera::Camera;
use crate::engine::shaders::{self, mandelbrot};

const BASE_ITERS: u32 = 300;

#[derive(Clone)]
pub struct RenderCamera {
    pub translation: [f32; 2],
    pub zoom: f32,
    pub max_iters: u32
}

impl RenderCamera {
    pub fn new() -> RenderCamera {
        RenderCamera { 
            translation: [0.0, 0.0],
            zoom: 1.0,
            max_iters: BASE_ITERS
        }
    }
}

impl From<Camera> for RenderCamera {
    fn from(cam: Camera) -> RenderCamera {
        RenderCamera {
            translation: cam.center,
            zoom: cam.zoom,
            max_iters: (cam.zoom * BASE_ITERS as f32) as u32 / 2
        }
    }
}

#[repr(C)]
#[derive(BufferContents, VertexTrait)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    tex_coords: [f32; 2],
}

const VERTICES: [Vertex; 3] = [
    Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [-1.0, 3.0],
        tex_coords: [0.0, 2.0],
    },
    Vertex {
        position: [3.0, -1.0],
        tex_coords: [2.0, 0.0],
    },
];

pub struct Renderer {
    window: VulkanoWindows,
    render_target_id: usize,
    
    pub camera: RenderCamera,

    memory_allocator: GenericMemoryAllocator<Arc<vulkano::memory::allocator::FreeListAllocator>>,
    command_buffer_allocator: StandardCommandBufferAllocator,
    descriptor_set_allocator: StandardDescriptorSetAllocator,
    
    device: Arc<Device>,
    data_offset: u32,

    compute_queue: Arc<Queue>,
    graphics_queue: Arc<Queue>,

    compute_shader: Arc<ShaderModule>,
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,

    render_pass: Arc<RenderPass>,
    compute_pipeline: Arc<ComputePipeline>,
    graphics_pipeline: Arc<GraphicsPipeline>,

    delta_time: f32,
    previous_frame: Instant,

    vertex_buffer: Subbuffer<[Vertex]>
}

impl Renderer {
    pub fn new(mut window: VulkanoWindows, device: Arc<Device>, context: VulkanoContext, camera: Camera) -> Renderer {
    //     println!("{}", std::mem::size_of::<RenderCamera>());

        let memory_allocator: GenericMemoryAllocator<Arc<vulkano::memory::allocator::FreeListAllocator>> = StandardMemoryAllocator::new_default(device.clone());
        let command_buffer_allocator =
            StandardCommandBufferAllocator::new(device.clone(), Default::default());
        let descriptor_set_allocator = StandardDescriptorSetAllocator::new(device.clone());
    
        let compute_queue = context.compute_queue().clone();
        let graphics_queue = context.graphics_queue().clone();
    
        let compute_shader = mandelbrot::cs::load(device.clone()).unwrap();
        let vertex_shader = mandelbrot::vs::load(device.clone()).unwrap();
        let fragment_shader = mandelbrot::fs::load(device.clone()).unwrap();

        let render_target_id = 0;
        let renderer = window
            .get_primary_renderer_mut()
            .expect("Failed to create renderer");

        let mut usage = ImageUsage::empty();
        usage = usage.union(ImageUsage::SAMPLED);
        usage = usage.union(ImageUsage::STORAGE);
        usage = usage.union(ImageUsage::COLOR_ATTACHMENT);
        usage = usage.union(ImageUsage::TRANSFER_DST);

        renderer.add_additional_image_view(
            render_target_id,
            DEFAULT_IMAGE_FORMAT,
            usage
        );
    
        let compute_pipeline = ComputePipeline::new(
            device.clone(),
            compute_shader.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .unwrap();

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: renderer.swapchain_format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();
    
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        
        let graphics_pipeline = GraphicsPipeline::start()
            .render_pass(subpass.clone())
            .vertex_shader(vertex_shader.entry_point("main").unwrap(), ())
            .fragment_shader(fragment_shader.entry_point("main").unwrap(), ())
            .input_assembly_state(Default::default())
            .vertex_input_state(Vertex::per_vertex())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .build(device.clone())
            .unwrap();

        let vertex_buffer = Buffer::from_iter(
            &memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                usage: MemoryUsage::Upload,
                ..Default::default()
            },
            VERTICES,
        )
        .unwrap();

        let offset = device.physical_device().properties().min_uniform_buffer_offset_alignment;

        Renderer {
            window,
            render_target_id,

            camera: camera.into(),

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,

            device,
            data_offset: offset.log2(),

            compute_queue,
            graphics_queue,

            compute_shader,
            vertex_shader,
            fragment_shader,

            render_pass,
            compute_pipeline,
            graphics_pipeline,

            delta_time: 0.0,
            previous_frame: Instant::now(),

            vertex_buffer
        }
    }

    pub fn get_iter(&self) -> u32 {
        return self.camera.max_iters;
    }

    pub fn update_view(&mut self, cam: Camera) {
        let iter = self.camera.max_iters;
        self.camera = cam.into();
        self.camera.max_iters = iter;
    }

    pub fn get_delta(&self) -> f32 {
        return self.delta_time;
    }

    pub fn resize(&mut self) {
        let renderer = self.window.get_primary_renderer_mut().unwrap();

        renderer.resize();
    }

    fn pad(min: u32, original: u32) -> u32 {
        // Calculate required alignment based on minimum device offset alignment
        // let minUboAlignment = _gpuProperties.limits.minUniformBufferOffsetAlignment;

        let mut aligned_size = original;
        if min > 0 {
            aligned_size = (aligned_size + min - 1) & !(min - 1);
        }
        return aligned_size; 
    }

    pub fn render(&mut self) {
        let renderer = self.window.get_primary_renderer_mut().unwrap();

        let now = Instant::now();
        let dt = now - self.previous_frame;
        self.delta_time = dt.as_secs_f32();

        //todo:do something with this
        let _fps = 1.0 / self.delta_time;


        self.previous_frame = now;

        let dimensions = renderer.window_size();
        let width = dimensions[0];
        let height = dimensions[1];

        if width == 0.0 || height == 0.0 {
            return;
        }

        let acquire_future = match renderer.acquire() {
            Ok(future) => future,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        let image_view = renderer.get_additional_image_view(self.render_target_id);

        let mut compute_command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let set_layout = self.compute_pipeline.layout().set_layouts().get(0).unwrap();

        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            set_layout.clone(),
            [WriteDescriptorSet::image_view(0, image_view.clone())],
        )
        .unwrap();

        let img_dims = image_view.image().dimensions().width_height();

        let render_data = self.camera.clone();

        let push_constants = mandelbrot::cs::PushConstants {
            scale: render_data.zoom.into(),
            translation: render_data.translation,
            max_iters: render_data.max_iters,
        };

        compute_command_buffer_builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.compute_pipeline.layout().clone(),
                0,
                set,
            )
            .push_constants(self.compute_pipeline.layout().clone(), 0, push_constants)
            .dispatch([img_dims[0] / 8, img_dims[1] / 8, 1])
            .expect("err er re r re  ");

        let compute_command_buffer = compute_command_buffer_builder.build().unwrap();

        let compute_future = compute_command_buffer
            .execute(self.graphics_queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .join(acquire_future);

        let sampler = Sampler::new(
            self.device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                mipmap_mode: SamplerMipmapMode::Linear,
                ..Default::default()
            },
        )
        .unwrap();

        let set_layout = self.graphics_pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_set_allocator,
            set_layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                image_view.clone(),
                sampler.clone(),
            )],
        )
        .unwrap();

        let mut graphics_command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.command_buffer_allocator,
            self.graphics_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let target = renderer.swapchain_image_view();

        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![target],
                ..Default::default()
            },
        )
        .unwrap();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: renderer.window_size(),
            depth_range: 0.0..1.0,
        };

        graphics_command_buffer_builder
            .bind_pipeline_graphics(self.graphics_pipeline.clone())
            .begin_render_pass(
                RenderPassBeginInfo {
                    render_pass: self.render_pass.clone(),
                    clear_values: vec![Some([0.3, 0.3, 0.3, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassContents::Inline,
            )
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.graphics_pipeline.layout().clone(),
                0,
                set,
            )
            .set_viewport(0, [viewport])
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(3 as u32, 1, 0, 0)
            .unwrap()
            .end_render_pass()
            .unwrap();

        let graphics_command_buffer = graphics_command_buffer_builder.build().unwrap();

        let after_future = compute_future
            .then_execute(self.graphics_queue.clone(), graphics_command_buffer)
            .unwrap()
            .boxed();

        renderer.present(after_future, true);
    }

}