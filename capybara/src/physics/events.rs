use parking_lot::RwLock;
use rapier2d::prelude::*;

#[derive(Default)]
pub struct EventCollector {
    pub collisions: RwLock<Vec<CollisionData>>,
    pub contacts: RwLock<Vec<ContactData>>,
}

pub struct CollisionData {
    pub event: CollisionEvent,
    pub pair: Option<ContactPair>,
}

pub struct ContactData {
    pub force: Real,
    pub pair: ContactPair,
}

impl EventCollector {
    pub fn clear(&mut self) {
        self.collisions.write().clear();
        self.contacts.write().clear();
    }
}

impl EventHandler for EventCollector {
    fn handle_collision_event(&self, _bodies: &RigidBodySet, _colliders: &ColliderSet, event: CollisionEvent, contact_pair: Option<&ContactPair>) {
        self.collisions.write().push(CollisionData::new(event, contact_pair.cloned()));
    }

    fn handle_contact_force_event(
        &self,
        _dt: Real,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        contact_pair: &ContactPair,
        total_force_magnitude: Real,
    ) {
        self.contacts.write().push(ContactData::new(total_force_magnitude, contact_pair.clone()));
    }
}

impl CollisionData {
    pub fn new(event: CollisionEvent, pair: Option<ContactPair>) -> Self {
        Self { event, pair }
    }
}

impl ContactData {
    pub fn new(force: Real, pair: ContactPair) -> Self {
        Self { force, pair }
    }
}
