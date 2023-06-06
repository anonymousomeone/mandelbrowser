use std::{ops::Range, sync::Arc};

use vulkano::device::{physical::PhysicalDevice, Device};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::image::{SwapchainImage, ImageUsage};

use winit::window::Window;

pub fn new_swapchain(p_device: Arc<PhysicalDevice>, 
        window: Arc<Window>,
        surface: Arc<Surface>,
        device: Arc<Device>) -> (Arc<Swapchain>, Vec<Arc<SwapchainImage>>){
    let caps = p_device
    .surface_capabilities(&surface, Default::default())
    .expect("failed to get surface capabilities");

    let dimensions = window.inner_size();
    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    let image_format = Some(
    p_device
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0,
    );

    let (swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
            composite_alpha,
            ..Default::default()
        },
    )
    .unwrap();

    (swapchain, images)
}