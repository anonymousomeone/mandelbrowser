use std::ops::ControlFlow;
use std::sync::Arc;

use vulkano::device::{DeviceExtensions, Features};
use vulkano::swapchain::PresentMode;
use vulkano_util::context::{VulkanoContext, VulkanoConfig};
use vulkano_util::window::{self, VulkanoWindows, WindowDescriptor};
use winit::event_loop::EventLoop;

use super::camera::Camera;

use super::renderer::render::Renderer;

// default startup dimensions
const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;

// default camera stuff
const PAN_SPEED: f32 = 0.25;
const ZOOM_SPEED: f32 = 0.3;
// gengine impact REAL

pub struct Engine {
    camera: Camera,

    pub renderer: Renderer
}

impl Engine {
    pub fn new() -> (Engine, EventLoop<()>) {
        let context = VulkanoContext::new(VulkanoConfig::default());

        let device = context.device();

        // window init stuff --
        let event_loop = EventLoop::new();
        let mut windows = VulkanoWindows::default();
        
        let _window_id = windows.create_window(
            &event_loop,
            &context,
            &WindowDescriptor {
                width: WIDTH as f32,
                height: HEIGHT as f32,
                title: "Mandelbrowser".to_string(),
                present_mode: PresentMode::Fifo,
                ..Default::default()
            },
            |_| {},
        );

        // -------------

        // renderer initialization
        let camera = Camera::new();

        let renderer = Renderer::new(windows, device.clone(), context, camera.clone());

        (Engine {
            camera,

            renderer
        },
        event_loop)
    }

    pub fn render(&mut self) {
        self.renderer.update_view(self.camera.clone());

        self.renderer.render();
    }

    pub fn camera_up(&mut self) {
        // let d = self.renderer.get_delta();

        self.camera.center[1] -= PAN_SPEED / self.camera.zoom as f32;
    }

    pub fn camera_down(&mut self) {
        // let d = self.renderer.get_delta();

        self.camera.center[1] += PAN_SPEED / self.camera.zoom as f32;
    }

    pub fn camera_right(&mut self) {
        // let d = self.renderer.get_delta();

        self.camera.center[0] += PAN_SPEED / self.camera.zoom as f32;
    }

    pub fn camera_left(&mut self) {
        // let d = self.renderer.get_delta();

        self.camera.center[0] -= PAN_SPEED / self.camera.zoom as f32;
    }

    pub fn zoom(&mut self, delta: f32) {

        self.camera.zoom += (delta * ZOOM_SPEED as f32) * 1.0f32.max(0.25 * self.camera.zoom);
        self.camera.zoom = self.camera.zoom.max(1.0);
    }

    pub fn resolution_up(&mut self) {
        self.renderer.camera.max_iters += 10;

    }

    pub fn resolution_down(&mut self) {
        self.renderer.camera.max_iters -= 10;

        self.renderer.camera.max_iters = self.renderer.camera.max_iters.max(10);
    }

    pub fn get_zoom(&self) -> f32 {
        return self.camera.zoom;
    }

    pub fn resize(&mut self) {
        self.renderer.resize();
    }

    pub fn reset_camera(&mut self) {
        self.camera = Camera::new();

        self.renderer.update_view(self.camera.clone());
    }
}