/*
 * Blue Engine by Elham Aryanpur
 *
 *
 *
 * The license is same as the one on the root.
*/

use legion::EntityStore;

use crate::header::{Entity, Pipeline, Shaders, Textures, UniformBuffers, VertexBuffers};

impl crate::header::Renderer {
    /// Creates and adds the pipeline to render queue
    pub fn build_and_append_pipeline(
        &mut self,
        shader: Entity,
        vertex_buffer: Entity,
        texture: Entity,
        uniform: Option<Entity>,
    ) -> Result<legion::Entity, anyhow::Error> {
        let pipe = self
            .build_pipeline(shader, vertex_buffer, texture, uniform)
            .expect("Couldn't Create Render Pipeline");
        let pipeline_entity = self.world.0.push((pipe,));
        Ok(pipeline_entity)
    }

    /// Creates a new render pipeline. Could be thought of as like materials in game engines.
    pub fn build_pipeline(
        &mut self,
        shader: Entity,
        vertex_buffer: Entity,
        texture: Entity,
        uniform: Option<Entity>,
    ) -> Result<Pipeline, anyhow::Error> {
        Ok(Pipeline {
            shader,
            vertex_buffer,
            texture,
            uniform,
        })
    }

    /// Appends a pipeline to render queue
    pub fn append_pipeline(&mut self, pipeline: Pipeline) -> Result<legion::Entity, anyhow::Error> {
        let pipeline_entity = self.world.0.push((pipeline,));
        Ok(pipeline_entity)
    }

    /// Allows to modify a pipeline
    pub fn get_pipeline(&mut self, index: legion::Entity) -> Result<&mut Pipeline, anyhow::Error> {
        match self.world.0.entry_mut(index) {
            Ok(mut pipeline_entry) => Ok(pipeline_entry
                .get_component_mut::<Pipeline>()
                .expect("Couldn't get pipeline")),
            Err(e) => Err(anyhow::Error::msg(format!(
                "Couldn't find the pipeline: {:?}",
                e
            ))),
        }
    }

    /// Deletes a render pipeline
    pub fn remove_pipeline(&mut self, index: legion::Entity) -> Result<(), anyhow::Error> {
        let result = self.world.0.remove(index);
        if result {
            Ok(())
        } else {
            Err(anyhow::Error::msg("Couldn't delete the pipeline"))
        }
    }
}
