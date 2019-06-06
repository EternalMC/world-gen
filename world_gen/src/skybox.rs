use glm::{ Vector3, GenNum };

use graphics::{ Mesh, ShaderProgram, ShaderProgramBuilder, Texture, GraphicsError };
use crate::{ Model, Camera, WorldError };
use crate::traits::{ Translatable, Scalable };

pub struct Skybox {
    texture: Texture,
    shader: ShaderProgram,
    model: Model,
    mesh: Mesh
}

impl Skybox {
    pub fn new(img_file: &str) -> Result<Self, WorldError> {
        const CUBE_PATH: &'static str = "resources/obj/cube_inward.obj";
        info!("Creating skybox from obj '{}' with img '{}'", CUBE_PATH, img_file);

        let shader = ShaderProgramBuilder::new()
            .add_vertex_shader("resources/shader/skybox/VertexShader.glsl")
            .add_fragment_shader("resources/shader/skybox/FragmentShader.glsl")
            .add_resource("texture_img")
            .add_resource("mvp")
            .finish()?;
        if let Err(e) = shader.set_resource_integer("texture_img", 0) {
            return Err(GraphicsError::from(e).into());
        }

        let texture = Texture::new(img_file)?;

        let mut model = Model::default();
        model.set_translation(Vector3::new(0., 0., 250.));
        model.set_scale(Vector3::from_s(100.));

        let mesh = Mesh::from_obj(CUBE_PATH)?;

        let sb = Skybox {
            shader: shader,
            texture: texture,
            model: model,
            mesh: mesh
        };

        Ok(sb)
    }

    // caller must restore previously set shader/textures after call
    pub fn render(&self, camera: &Camera) -> Result<(), GraphicsError> {
        self.texture.activate();
        self.shader.use_program();

        let mvp = camera.create_mvp_matrix(&self.model);
        self.shader.set_resource_mat4("mvp", &mvp)?;
        self.mesh.render()?;

        self.texture.deactivate();
        Ok(())
    }
}