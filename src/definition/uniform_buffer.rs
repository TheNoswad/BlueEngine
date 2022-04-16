/*
 * Blue Engine by Elham Aryanpur
 *
 * The license is same as the one on the root.
*/

use crate::header::{UniformBuffer, UniformBuffers};
use wgpu::{util::DeviceExt, BindGroupLayout};

impl crate::header::Renderer {
    /// Creates and adds the uniform buffers to render queue
    pub fn build_and_append_uniform_buffers(
        &mut self,
        uniforms: Vec<UniformBuffer>,
    ) -> Result<(legion::Entity, BindGroupLayout), anyhow::Error> {
        let uniform_buffers = self
            .build_uniform_buffer(uniforms)
            .expect("Couldn't create uniform buffer");
        let index = self.world.0.push((uniform_buffers,));
        Ok((index, uniform_buffers.1))
    }

    /// Creates a new uniform buffer group, according to a list of types
    pub fn build_uniform_buffer(
        &mut self,
        uniforms: Vec<UniformBuffer>,
    ) -> Result<(UniformBuffers, BindGroupLayout), anyhow::Error> {
        let mut buffer_entry = Vec::<wgpu::BindGroupEntry>::new();
        let mut buffer_layout = Vec::<wgpu::BindGroupLayoutEntry>::new();
        let mut buffer_vec = Vec::<wgpu::Buffer>::new();
        for i in uniforms.iter() {
            match i {
                UniformBuffer::Matrix(name, value) => {
                    buffer_vec.push(self.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some(*name),
                            contents: bytemuck::cast_slice(&[*value]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        },
                    ));
                }
                UniformBuffer::Array(name, value) => {
                    buffer_vec.push(self.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some(*name),
                            contents: bytemuck::cast_slice(&[*value]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        },
                    ));
                }
                UniformBuffer::Float(name, value) => {
                    buffer_vec.push(self.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some(*name),
                            contents: bytemuck::cast_slice(&[*value]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        },
                    ));
                }
            }
        }
        for i in 0..buffer_vec.len() {
            let descriptor = wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer_vec.get(i).unwrap(),
                    offset: 0,
                    size: None,
                }),
            };
            buffer_entry.push(descriptor);
            buffer_layout.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        let uniform_bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniform dynamic bind group layout"),
                    entries: &buffer_layout.as_slice(),
                });

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Groups"),
            layout: &uniform_bind_group_layout,
            entries: &buffer_entry.as_slice(),
        });

        Ok((uniform_bind_group, uniform_bind_group_layout))
    }

    /// Appends a uniform buffer to render queue
    pub fn append_uniform_buffer(
        &mut self,
        buffer: UniformBuffers,
    ) -> Result<legion::Entity, anyhow::Error> {
        let index = self.world.0.push((buffer,));
        Ok(index)
    }

    /// Allows to modify a uniform buffer
    pub fn get_uniform_buffer(
        &mut self,
        index: legion::Entity,
    ) -> Result<&mut UniformBuffers, anyhow::Error> {
        match self.world.0.entry(index) {
            Some(pipeline_entry) => Ok(pipeline_entry.get_component_mut::<UniformBuffers>()?),
            None => Err(anyhow::Error::msg("Couldn't find the pipeline")),
        }
    }

    /// Removes uniform buffer group
    pub fn remove_uniform_buffer(&mut self, index: legion::Entity) -> Result<(), anyhow::Error> {
        self.world.0.remove(index);
        Ok(())
    }
}
