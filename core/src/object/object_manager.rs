use std::collections::BTreeMap;
use std::sync::Arc;

use crate::file::read_file;
use super::{ Object, ObjectPrototype, ObjectError, FilePrototype };

pub struct ObjectManager {
    prototype_map: BTreeMap<String, Arc<ObjectPrototype>>
}

impl ObjectManager {

    pub fn from_yaml(file_path: &str) -> Result<ObjectManager, ObjectError> {
        info!("Creating object manager by yaml, path = '{}'", file_path);
        let file = read_file(file_path)?;
        let parsed_file: FilePrototype = serde_yaml::from_str(file.as_str())?;

        let mut obj_manager = ObjectManager::default();

        for (name, lod0_path, lod1_path) in parsed_file.into_iter() {
            info!("Loading prototype '{}', lod0 = '{}', lod1 = '{}'", name, lod0_path, lod1_path);
            obj_manager.add_prototype(&name, &lod0_path, &lod1_path)?;
        }

        Ok(obj_manager)
    }

    pub fn add_prototype(&mut self, name: &str, lod0_path: &str, lod1_path: &str) -> Result<(), ObjectError> {
        debug_assert!(!self.prototype_map.contains_key(name));
        let prototype = ObjectPrototype::from_obj(lod0_path, lod1_path)?;
        self.prototype_map.insert(name.to_string(), Arc::new(prototype));
        Ok(())
    }

    pub fn create_object(&self, prototype_name: &str) -> Result<Object, ObjectError> {
        match self.prototype_map.get(prototype_name) {
            Some(proto) => {
                let obj = Object::new(proto.clone());
                Ok(obj)
            },
            None => {
                Err(ObjectError::PrototypeNotExisting(prototype_name.to_string()).into())
            }
        }
    }

}

impl Default for ObjectManager {
    fn default() -> ObjectManager {
        ObjectManager {
            prototype_map: BTreeMap::new()
        }
    }
}

