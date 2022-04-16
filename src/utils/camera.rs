/*
 * Blue Engine by Elham Aryanpur
 *
 * The license is same as the one on the root.
*/

use crate::header::{uniform_type::Matrix, Camera, Renderer, UniformBuffer};
use anyhow::Result;

use super::default_resources::DEFAULT_MATRIX_4;

impl Camera {
    /// Creates a new camera. this should've been automatically done at the time of creating an engine
    pub fn new(renderer: &mut Renderer) -> Result<Self> {
        let camera_uniform =
            renderer.build_and_append_uniform_buffers(vec![UniformBuffer::Matrix(
                "Camera Uniform",
                DEFAULT_MATRIX_4,
            )])?;

        let mut camera = Self {
            position: nalgebra_glm::vec3(0.0, 0.0, 1.0),
            target: nalgebra_glm::vec3(0.0, 0.0, -1.0).into(),
            up: nalgebra_glm::vec3(0.0, 1.0, 0.0),
            aspect: renderer.config.width as f32 / renderer.config.height as f32,
            fov: 70f32 * (std::f32::consts::PI / 180f32),
            near: 0.1,
            far: 100.0,
            view_data: DEFAULT_MATRIX_4.to_im(),
            changed: true,
            entity_id: camera_uniform.0,
        };
        camera.build_view_projection_matrix()?;

        Ok(camera)
    }

    /// Updates the view uniform matrix that decides how camera works
    pub fn build_view_projection_matrix(&mut self) -> Result<()> {
        let view = nalgebra_glm::look_at_rh(&self.position, &self.target, &self.up);
        let proj = nalgebra_glm::perspective(self.fov, self.aspect, self.near, self.far);
        self.view_data = proj * view;
        self.changed = true;

        Ok(())
    }

    /// Returns a matrix uniform buffer from camera data that can be sent to GPU
    pub fn camera_uniform_buffer(&self) -> Result<Matrix> {
        Ok(Matrix::from_im(self.view_data))
    }

    /// Sets the position of camera
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) -> Result<()> {
        self.position = nalgebra_glm::vec3(x, y, z);
        self.build_view_projection_matrix()?;

        Ok(())
    }

    /// Sets the target of camera
    pub fn set_target(&mut self, x: f32, y: f32, z: f32) -> Result<()> {
        self.target = nalgebra_glm::vec3(x, y, z);
        self.build_view_projection_matrix()?;

        Ok(())
    }

    /// Sets the up of camera
    pub fn set_up(&mut self, x: f32, y: f32, z: f32) -> Result<()> {
        self.up = nalgebra_glm::vec3(x, y, z);
        self.build_view_projection_matrix()?;

        Ok(())
    }

    /// Sets the field of view of camera
    pub fn set_fov(&mut self, new_fov: f32) -> Result<()> {
        self.fov = new_fov;
        self.build_view_projection_matrix()?;

        Ok(())
    }

    /// Sets how far camera can look
    pub fn set_far(&mut self, new_far: f32) -> Result<()> {
        self.far = new_far;
        self.build_view_projection_matrix()?;

        Ok(())
    }

    /// Sets how near the camera can look
    pub fn set_near(&mut self, new_near: f32) -> Result<()> {
        self.near = new_near;
        self.build_view_projection_matrix()?;

        Ok(())
    }

    /// Sets the aspect ratio of the camera
    pub fn set_aspect(&mut self, new_aspect: f32) -> Result<()> {
        self.aspect = new_aspect;
        self.build_view_projection_matrix()?;

        Ok(())
    }

    /// This builds a uniform buffer data from camera view data that is sent to the GPU in next frame
    pub fn update_view_projection(&mut self, renderer: &mut Renderer) -> Result<()> {
        if self.changed {
            let mut updated_buffer = renderer
                .build_uniform_buffer(vec![UniformBuffer::Matrix(
                    "Camera Uniform",
                    self.camera_uniform_buffer()
                        .expect("Couldn't build camera projection"),
                )])
                .expect("Couldn't update the camera uniform buffer")
                .0;
            let _ = std::mem::replace(
                &mut renderer.get_uniform_buffer(self.entity_id)?,
                &mut updated_buffer,
            );
            self.changed = false;
        }

        Ok(())
    }
}
