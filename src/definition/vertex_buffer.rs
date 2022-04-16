/*
 * Blue Engine by Elham Aryanpur
 *
 * The license is same as the one on the root.
*/

use crate::header::{Vertex, VertexBuffers};
use wgpu::util::DeviceExt;

impl crate::header::Renderer {
    /// Creates and adds the vertex buffers to render queue
    pub fn build_and_append_vertex_buffers(
        &mut self,
        verticies: Vec<Vertex>,
        indicies: Vec<u16>,
    ) -> Result<legion::Entity, anyhow::Error> {
        let vertex_buffers = self
            .build_vertex_buffers(verticies, indicies)
            .expect("Couldn't create vertex buffer");
        let index = self.world.0.push((vertex_buffers,));
        Ok(index)
    }

    /// Creates a new vertex buffer and indecies
    pub fn build_vertex_buffers(
        &mut self,
        verticies: Vec<Vertex>,
        indicies: Vec<u16>,
    ) -> Result<VertexBuffers, anyhow::Error> {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(verticies.as_slice()),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indicies.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            });

        Ok(VertexBuffers {
            vertex_buffer,
            index_buffer,
            length: indicies.len() as u32,
        })
    }

    /// Appends a vertex buffer to render queue
    pub fn append_vertex_buffer(
        &mut self,
        vertex_buffer: VertexBuffers,
    ) -> Result<legion::Entity, anyhow::Error> {
        let index = self.world.0.push((vertex_buffer,));
        Ok(index)
    }

    /// Allows to modify a vertex buffer
    pub fn get_vertex_buffer(
        &mut self,
        index: legion::Entity,
    ) -> Result<&mut VertexBuffers, anyhow::Error> {
        match self.world.0.entry(index) {
            Some(pipeline_entry) => Ok(pipeline_entry.get_component_mut::<VertexBuffers>()?),
            None => Err(anyhow::Error::msg("Couldn't find the pipeline")),
        }
    }

    /// Removes vertex and index buffer group
    pub fn remove_vertex_buffer(&mut self, index: legion::Entity) -> Result<(), anyhow::Error> {
        self.world.0.remove(index);
        Ok(())
    }
}
