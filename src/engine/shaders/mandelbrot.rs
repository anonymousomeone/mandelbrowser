pub mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: r"
            #version 460

            layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
            layout(binding = 0, set = 0, rgba8) uniform writeonly image2D image;

            layout(push_constant) uniform PushConstants {
                float scale;
                vec2 translation;
                uint max_iters;
    
            } push_constants;

            // https://github.com/hughsk/glsl-hsv2rgb
            vec3 hsv2rgb(vec3 c) {
                vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
                vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
                return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
            }

            void main() {
                vec2 dims = vec2(imageSize(image));

                if (gl_GlobalInvocationID.x > dims.x || gl_GlobalInvocationID.x < dims.x) {
                    return;
                }

                if (gl_GlobalInvocationID.y > dims.y || gl_GlobalInvocationID.y < dims.y) {
                    return;
                }
                
                float ar = dims.x / dims.y;
                float x_norm = (gl_GlobalInvocationID.x / dims.x);
                float y_norm = (gl_GlobalInvocationID.y / dims.y);
    
                float x0 = ar * (x_norm * 4.0 / push_constants.scale) - (2.0 / push_constants.scale) + push_constants.translation.x;
                float y0 = (y_norm * 4.0 / push_constants.scale) - (2.0 / push_constants.scale) + push_constants.translation.y;

                uint iterations;

                vec2 c = vec2(x0, y0);
                vec2 z = c;

                vec2 old = vec2(0.0, 0.0);
                uint period = 0;

                for (iterations = 0; iterations < push_constants.max_iters; iterations += 1) {
                    z = vec2(
                        z.x * z.x - z.y * z.y + c.x,
                        z.y * z.x + z.x * z.y + c.y
                    );
            
                    if (length(z) > 4.0) {
                        break;
                    }
                    
                    // periodicity checking
                    if (z == old) {
                        iterations = push_constants.max_iters;
                        break;
                    }

                    period += 1;
                    if (period > 20) {
                        period = 0;
                        old = z;
                    }
                }

                float i = float(iterations) / push_constants.max_iters;
                vec4 pixel = vec4(vec3(i), 0.1);


                imageStore(image, ivec2(gl_GlobalInvocationID.xy), pixel);
            }
        ",
    }
}

pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
        #version 460

        layout(location = 0) in vec2 position;
        layout(location = 1) in vec2 tex_coords;

        layout(location = 0) out vec2 f_tex_coords;

        void main() {
            gl_Position = vec4(position.xy, 0.0, 1.0);
            f_tex_coords = tex_coords;;
        }
        "
    }
}

pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
        #version 460

        layout(location = 0) in vec2 tex_coords;

        layout(location = 0) out vec4 f_color;

        layout(set = 0, binding = 0) uniform sampler2D tex;

        void main() {
            f_color = texture(tex, tex_coords);
        }

        "
    }
}