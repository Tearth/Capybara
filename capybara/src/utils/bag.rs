use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::collections::VecDeque;

pub struct Bag<T> {
    data: Vec<Option<T>>,
    removed_ids: VecDeque<usize>,
}

impl<T> Bag<T> {
    pub fn store(&mut self, item: T) -> usize {
        let id = self.get_new_id();
        self.data[id] = Some(item);

        id
    }

    pub fn contains(&self, id: usize) -> bool {
        self.data.get(id).is_some()
    }

    pub fn get(&self, id: usize) -> Result<&T> {
        match self.data.get(id) {
            Some(item) => Ok(item.as_ref().ok_or_else(|| anyhow!("Bag item {} not found", id))?),
            None => bail!("Bag item {} not found", id),
        }
    }

    pub fn get_mut(&mut self, id: usize) -> Result<&mut T> {
        match self.data.get_mut(id) {
            Some(item) => Ok(item.as_mut().ok_or_else(|| anyhow!("Bag item {} not found", id))?),
            None => bail!("Bag item {} not found", id),
        }
    }

    pub fn remove(&mut self, id: usize) -> Result<()> {
        if id >= self.data.len() || self.data[id].is_none() {
            bail!("Bag item {} not found", id);
        }

        self.data[id] = None;
        self.removed_ids.push_back(id);

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

impl<T> Default for Bag<T> {
    fn default() -> Self {
        Self { data: Default::default(), removed_ids: Default::default() }
    }
}
