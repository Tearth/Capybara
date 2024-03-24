use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct Storage<T> {
    data: Vec<Option<T>>,
    name_to_id_hashmap: FxHashMap<String, usize>,
    id_to_name_hashmap: FxHashMap<usize, String>,
    removed_ids: VecDeque<usize>,
}

impl<T> Storage<T> {
    pub fn store(&mut self, item: T) -> usize {
        let id = self.get_new_id();
        self.data[id] = Some(item);

        id
    }

    pub fn store_with_name(&mut self, name: &str, item: T) -> Result<usize> {
        if self.name_to_id_hashmap.contains_key(name) {
            bail!("Name already exists".to_string());
        }

        let id = self.get_new_id();
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

    pub fn get_unchecked(&self, id: usize) -> &T {
        unsafe { self.data[id].as_ref().unwrap_unchecked() }
    }

    pub fn get_id(&self, name: &str) -> Result<usize> {
        match self.name_to_id_hashmap.get(name) {
            Some(id) => Ok(*id),
            None => bail!("Storage item {} not found", name),
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

    pub fn get_unchecked_mut(&mut self, id: usize) -> &mut T {
        unsafe { self.data[id].as_mut().unwrap_unchecked() }
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Result<&mut T> {
        match self.name_to_id_hashmap.get_mut(name) {
            Some(id) => Ok(self.data[*id].as_mut().ok_or_else(|| anyhow!("Storage item {} not found", id))?),
            None => bail!("Storage item {} not found", name),
        }
    }

    pub fn remove(&mut self, id: usize) -> Option<T> {
        if id >= self.data.len() || self.data[id].is_none() {
            return None;
        }

        let item = self.data[id].take();
        self.removed_ids.push_back(id);

        if let Some(name) = self.id_to_name_hashmap.get(&id) {
            self.name_to_id_hashmap.remove(name);
            self.id_to_name_hashmap.remove(&id);
        }

        item
    }

    pub fn remove_by_name(&mut self, name: &str) -> Option<T> {
        if !self.name_to_id_hashmap.contains_key(name) {
            return None;
        }

        let id = self.name_to_id_hashmap[name];

        let item = self.data[id].take();
        self.removed_ids.push_back(id);

        self.name_to_id_hashmap.remove(name);
        self.id_to_name_hashmap.remove(&id);

        item
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

    pub fn get_next_id(&self, id: Option<usize>) -> Option<usize> {
        let from = match id {
            Some(id) => id + 1,
            None => 0,
        };

        (from..self.data.len()).find(|&i| self.data[i].is_some())
    }

    pub fn len(&self) -> usize {
        self.data.len() - self.removed_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.len() == self.removed_ids.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.name_to_id_hashmap.clear();
        self.id_to_name_hashmap.clear();
        self.removed_ids.clear();
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
            data: Vec::default(),
            name_to_id_hashmap: HashMap::default(),
            id_to_name_hashmap: HashMap::default(),
            removed_ids: VecDeque::default(),
        }
    }
}
