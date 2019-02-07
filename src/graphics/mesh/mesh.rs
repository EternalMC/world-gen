use std::collections::BTreeMap;
use std::collections::btree_map::Entry;
use std::convert::TryFrom;
use std::{ ptr, io, ffi::c_void, mem::size_of, ops::Sub };
use gl;
use gl::types::{ GLint, GLuint, GLenum, GLsizeiptr };
use glm::{ Matrix4, Vector3, builtin::{ dot, normalize } };

use crate::utility::read_obj;
use crate::graphics::{ check_opengl_error, OpenglError, mesh::{ Vertex, Triangle } };
use super::{ VAO, MeshError, Buffer };

pub struct Mesh {
    vao: Option<VAO>
}

impl Mesh {
    pub fn from_obj(obj_path: &str) -> Result<Mesh, MeshError> {
        Self::try_from((read_obj(obj_path)?).as_slice())
    }

    pub fn get_vertex_count(&self) -> u32 {
        match self.vao {
            Some(ref vao) => vao.get_index_count(),
            _ => 0
        }
    }

    pub fn render(&self) -> Result<(), MeshError> {
        match self.vao {
            Some(ref vao) => vao.render(),
            None => { Ok(()) }
        }
    }
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            vao: None
        }
    }
}

impl TryFrom<Buffer> for Mesh {
    type Error = MeshError;
    fn try_from(buffer: Buffer) -> Result<Self, Self::Error> {
        let mesh = Self {
            vao: Some(VAO::try_from(buffer)?)
        };
        Ok(mesh)
    }
}

impl TryFrom<&[Triangle]> for Mesh {
    type Error = MeshError;
    fn try_from(triangles: &[Triangle]) -> Result<Self, Self::Error> {
        let buffer = Buffer::from(triangles);
        let mesh = Self {
            vao: Some(VAO::try_from(buffer)?)
        };
        Ok(mesh)
    }
}
