use vulkano::buffer::BufferContents;

#[derive(Clone)]
pub struct Camera {
    pub center: [f32; 2],
    pub zoom: f32
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            center: [0.0, 0.0],
            zoom: 1.0
        }
    }
}
