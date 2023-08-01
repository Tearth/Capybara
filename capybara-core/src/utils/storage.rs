use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::collections::HashMap;
use std::collections::VecDeque;

pub struct Storage<T> {
    data: Vec<Option<T>>,
    name_to_id_hashmap: HashMap<String, usize>,
    id_to_name_hashmap: HashMap<usize, String>,
    removed_ids: VecDeque<usize>,
}

pub trait StorageItem {
    fn get_id(&self) -> usize;
    fn set_id(&mut self, id: usize);

    fn get_name(&self) -> Option<String>;
    fn set_name(&mut self, name: Option<String>);
}

impl<T> Storage<T>
where
    T: StorageItem,
{
    pub fn store(&mut self, mut item: T) -> usize {
        let id = self.get_new_id();
        item.set_id(id);
        self.data[id] = Some(item);

        id
    }

    pub fn store_with_name(&mut self, name: &str, mut item: T) -> Result<usize> {
        if self.name_to_id_hashmap.contains_key(name) {
            bail!("Name already exists".to_string());
        }

        let id = self.get_new_id();
        item.set_id(id);
        item.set_name(Some(name.to_string()));
        self.data[id] = Some(item);

        self.name_to_id_hashmap.insert(name.to_string(), id);
        self.id_to_name_hashmap.insert(id, name.to_string());

        Ok(id)
    }

    pub fn contains(&self, id: usize) -> bool {
        self.data.get(id).is_some()
    }

    pub fn contains_by_name(&self, name: &str) -> bool {
        self.name_to_id_hashmap.get(name).is_some()
    }

    pub fn get(&self, id: usize) -> Result<&T> {
        match self.data.get(id) {
            Some(item) => Ok(item.as_ref().ok_or_else(|| anyhow!("Storage item {} not found", id))?),
            None => bail!("Storage item {} not found", id),
        }
    }

    pub fn get_by_name(&self, name: &str) -> Result<&T> {
        match self.name_to_id_hashmap.get(name) {
            Some(id) => Ok(self.data[*id].as_ref().ok_or_else(|| anyhow!("Storage item {} not found", id))?),
            None => bail!("Storage item {} not found", name),
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Result<&mut T> {
        match self.data.get_mut(id) {
            Some(item) => Ok(item.as_mut().ok_or_else(|| anyhow!("Storage item {} not found", id))?),
            None => bail!("Storage item {} not found", id),
        }
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Result<&mut T> {
        match self.name_to_id_hashmap.get_mut(name) {
            Some(id) => Ok(self.data[*id].as_mut().ok_or_else(|| anyhow!("Storage item {} not found", id))?),
            None => bail!("Storage item {} not found", name),
        }
    }

    pub fn remove(&mut self, id: usize) -> Result<()> {
        if id >= self.data.len() || self.data[id].is_none() {
            bail!("Storage item {} not found", id);
        }

        self.data[id] = None;
        self.removed_ids.push_back(id);

        if let Some(name) = self.id_to_name_hashmap.get(&id) {
            self.name_to_id_hashmap.remove(name);
            self.id_to_name_hashmap.remove(&id);
        }

        Ok(())
    }

    pub fn remove_by_name(&mut self, name: &str) -> Result<()> {
        if !self.name_to_id_hashmap.contains_key(name) {
            bail!("Name doesn't exist".to_string());
        }

        let id = self.name_to_id_hashmap[name];

        self.data[id] = None;
        self.removed_ids.push_back(id);

        self.name_to_id_hashmap.remove(name);
        self.id_to_name_hashmap.remove(&id);

        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().filter_map(|p| p.as_ref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().filter_map(|p| p.as_mut())
    }

    pub fn iter_enumerate(&self) -> impl Iterator<Item = (usize, &T)> {
        self.data.iter().enumerate().filter(|(_, p)| p.is_some()).map(|(i, p)| (i, p.as_ref().unwrap()))
    }

    pub fn iter_enumerate_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.data.iter_mut().enumerate().filter(|(_, p)| p.is_some()).map(|(i, p)| (i, p.as_mut().unwrap()))
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.removed_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == self.removed_ids.len()
    }

    fn get_new_id(&mut self) -> usize {
        if let Some(id) = self.removed_ids.pop_front() {
            id
        } else {
            self.data.push(None);
            self.data.len() - 1
        }
    }
}

impl<T> Default for Storage<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            name_to_id_hashmap: Default::default(),
            id_to_name_hashmap: Default::default(),
            removed_ids: Default::default(),
        }
    }
}
