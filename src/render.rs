/*
 * Blue Engine by Elham Aryanpur
 *
 * The license is same as the one on the root.
*/

use crate::header::{Pipeline, Renderer};
use anyhow::Result;
use legion::IntoQuery;
use wgpu::Features;
use winit::window::Window;

#[cfg(not(target_feature = "NON_FILL_POLYGON_MODE"))]
fn get_render_features() -> Features {
    Features::empty()
}
#[cfg(target_feature = "NON_FILL_POLYGON_MODE")]
fn get_render_features() -> Features {
    Features::NON_FILL_POLYGON_MODE
}

impl Renderer {
    pub(crate) async fn new(window: &Window) -> anyhow::Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                //force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    features: get_render_features(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let default_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform dynamic bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let depth_buffer = Renderer::build_depth_buffer("Depth Buffer", &device, &config);

        let world = (
            legion::World::default(),
            legion::Schedule::builder().build(),
        );

        let mut renderer = Self {
            surface,
            device,
            queue,
            config,
            size,

            texture_bind_group_layout,
            default_uniform_bind_group_layout,
            depth_buffer,

            world,
            default_data: None,
        };

        let default_uniform = renderer.build_and_append_uniform_buffers(vec![
            crate::header::UniformBuffer::Matrix(
                "Transformation Matrix",
                crate::utils::default_resources::DEFAULT_MATRIX_4,
            ),
            crate::header::UniformBuffer::Array(
                "Color",
                crate::header::uniform_type::Array {
                    data: crate::utils::default_resources::DEFAULT_COLOR,
                },
            ),
        ])?;

        let default_shader = renderer.build_and_append_shaders(
            "Default Shader",
            crate::utils::default_resources::DEFAULT_SHADER.to_string(),
            Some(&default_uniform.1),
            crate::header::ShaderSettings::default(),
        )?;

        let default_texture = renderer.build_and_append_texture(
            "Default Texture",
            crate::header::TextureData::Bytes(
                crate::utils::default_resources::DEFAULT_TEXTURE.to_vec(),
            ),
            crate::header::TextureMode::Clamp,
            //crate::header::TextureFormat::PNG
        )?;

        renderer.default_data = Some((default_uniform.0, default_shader, default_texture));

        Ok(renderer)
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_buffer = Self::build_depth_buffer("Depth Buffer", &self.device, &self.config);
    }

    pub(crate) fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_frame()?.output;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_buffer.1,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        let mut already_loaded_shader: usize = 0;
        let mut already_loaded_buffer: usize = 5;
        let mut already_loaded_texture: usize = 0;
        let mut already_loaded_uniform_buffer: usize = 0;

        render_pass.set_bind_group(
            1,
            self.get_uniform_buffer(self.default_data.unwrap().0)
                .expect("Couldn't find the camera uniform data"),
            &[],
        );
        render_pass.set_pipeline(
            self.get_shader(self.default_data.unwrap().1)
                .expect("Couldn't find the default shader"),
        );

        self.get_texture_mut(self.default_data.unwrap().2, |tex| {
            render_pass.set_bind_group(0, tex, &[]);
        })
        .expect("Couldn't lock the default texture");

        let query = <&Pipeline>::query();

        for i in query.iter(&self.world.0) {
            // Shaders
            let shader = self
                .get_shader(i.shader)
                .expect("Couldn't get shader from the world");
            render_pass.set_pipeline(shader);

            // Textures
            self.get_texture(i.texture, |tex| {
                render_pass.set_bind_group(0, tex, &[]);
            })
            .expect("Couldn't get texture from the world");

            // Uniform Buffer
            let uniform_buffer_enum_option = i.uniform;
            if uniform_buffer_enum_option.is_some() {
                let uniform_buffer = uniform_buffer_enum_option
                    .expect(format!("Uniform buffer group at doesn't exist",).as_str());

                let uniform_buffer = self
                    .get_uniform_buffer(uniform_buffer)
                    .expect("Couldn't get uniform buffer from the world");
                render_pass.set_bind_group(2, &uniform_buffer, &[]);
            }

            // Vertex Buffer
            let buffers = *self
                .get_vertex_buffer(i.vertex_buffer)
                .expect("Couldn't get vertex buffer from the world");
            render_pass.set_vertex_buffer(0, buffers.vertex_buffer.slice(..));
            render_pass.set_index_buffer(buffers.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..buffers.length, 0, 0..1);
        }

        drop(render_pass);

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
