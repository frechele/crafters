use std::collections::HashSet;

use crate::entities::{Entity, Player};
use crate::game_rules::{EntityBehavior, GameRules};
use crate::py_random::PyRandom;
use crate::registry::{EntityTypeId, MaterialId};
use crate::{Direction, Position, SemanticGrid};

#[derive(Clone, Debug)]
pub struct World {
    area: [usize; 2],
    chunk_size: [usize; 2],
    materials: Vec<u16>, // 0 = empty, 1+ = MaterialId
    daylight: f32,
    rng: PyRandom,
    entities: Vec<Option<Entity>>,
}

impl World {
    pub fn new(area: [usize; 2], chunk_size: [usize; 2]) -> Self {
        Self {
            area,
            chunk_size,
            materials: vec![0; area[0] * area[1]],
            daylight: 0.0,
            rng: PyRandom::new(0),
            entities: Vec::new(),
        }
    }

    pub fn reset(&mut self, seed: u64) {
        self.materials.fill(0);
        self.daylight = 0.0;
        self.rng = PyRandom::new(seed as u32);
        self.clear_objects();
    }

    pub fn area(&self) -> [usize; 2] {
        self.area
    }

    pub fn chunk_size(&self) -> [usize; 2] {
        self.chunk_size
    }

    pub fn material(&self, pos: Position) -> Option<MaterialId> {
        let val = self.materials[self.index(pos)];
        if val == 0 {
            None
        } else {
            Some(MaterialId(val))
        }
    }

    pub fn set_material(&mut self, pos: Position, material: MaterialId) {
        let index = self.index(pos);
        self.materials[index] = material.0;
    }

    pub fn fill(&mut self, material: MaterialId) {
        self.materials.fill(material.0);
    }

    pub fn clear_objects(&mut self) {
        self.entities.clear();
    }

    /// Spawn an entity at the given position if the position is not occupied.
    pub fn spawn_entity(&mut self, pos: Position, type_id: EntityTypeId, health: i32) {
        if self.entity_at(pos, None).is_none() {
            self.entities.push(Some(Entity::new(type_id, pos, health)));
        }
    }

    /// Spawn an entity with a specific facing (used for arrows/projectiles).
    pub fn spawn_entity_facing(&mut self, pos: Position, type_id: EntityTypeId, facing: Direction) {
        if self.entity_at(pos, None).is_none() {
            self.entities.push(Some(Entity::with_facing(type_id, pos, facing)));
        }
    }

    // Legacy convenience methods for tests and external callers
    pub fn spawn_cow(&mut self, pos: Position, health: i32) {
        // type_id 0 = cow (first entity in config)
        self.spawn_entity(pos, EntityTypeId(0), health);
    }

    pub fn spawn_zombie(&mut self, pos: Position, health: i32) {
        // type_id 1 = zombie
        self.spawn_entity(pos, EntityTypeId(1), health);
    }

    pub fn spawn_skeleton(&mut self, pos: Position, health: i32) {
        // type_id 2 = skeleton
        self.spawn_entity(pos, EntityTypeId(2), health);
    }

    pub fn spawn_arrow(&mut self, pos: Position, facing: Direction) {
        // type_id 3 = arrow
        self.spawn_entity_facing(pos, EntityTypeId(3), facing);
    }

    pub fn spawn_plant(&mut self, pos: Position, health: i32, _ripen_time: i32) {
        // type_id 4 = plant
        self.spawn_entity(pos, EntityTypeId(4), health);
    }

    pub fn spawn_fence(&mut self, pos: Position) {
        // type_id 5 = fence
        self.spawn_entity(pos, EntityTypeId(5), 0);
    }

    pub fn arrow_count(&self) -> usize {
        self.entities.iter().flatten()
            .filter(|e| e.type_id == EntityTypeId(3))
            .count()
    }

    pub fn semantic_view(&self, player: &Player, rules: &GameRules) -> SemanticGrid {
        let mut cells: Vec<u16> = self.materials.clone();
        for (_, entity) in self.entity_iter() {
            let def = rules.entity_def(entity.type_id);
            cells[self.index(entity.pos)] = def.semantic_id;
        }
        cells[self.index(player.pos())] = 13; // player semantic ID
        SemanticGrid::new(self.area[0], self.area[1], cells)
    }

    pub fn random_f32(&mut self) -> f32 {
        self.rng.uniform() as f32
    }

    pub fn random_f64(&mut self) -> f64 {
        self.rng.uniform()
    }

    pub fn random_usize(&mut self, upper_exclusive: usize) -> usize {
        self.rng.randint(upper_exclusive as u32) as usize
    }

    pub fn random_i64(&mut self, upper_exclusive: i64) -> i64 {
        self.rng.randint(upper_exclusive as u32) as i64
    }

    pub fn random_bool(&mut self, probability: f32) -> bool {
        self.random_f64() < probability as f64
    }

    pub fn random_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    pub fn daylight(&self) -> f32 {
        self.daylight
    }

    pub fn set_daylight(&mut self, daylight: f32) {
        self.daylight = daylight;
    }

    pub(crate) fn inside_signed(&self, pos: [isize; 2]) -> bool {
        pos[0] >= 0
            && pos[1] >= 0
            && (pos[0] as usize) < self.area[0]
            && (pos[1] as usize) < self.area[1]
    }

    pub(crate) fn offset_pos(&self, pos: Position, delta: [isize; 2]) -> Option<Position> {
        let next = [pos[0] as isize + delta[0], pos[1] as isize + delta[1]];
        self.inside_signed(next)
            .then_some([next[0] as usize, next[1] as usize])
    }

    /// Returns all live entity handles (indices).
    pub(crate) fn entity_handles(&self) -> Vec<usize> {
        self.entities
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| slot.as_ref().map(|_| idx))
            .collect()
    }

    /// Iterator over (index, &Entity) for live entities.
    pub(crate) fn entity_iter(&self) -> impl Iterator<Item = (usize, &Entity)> {
        self.entities
            .iter()
            .enumerate()
            .filter_map(|(idx, slot)| slot.as_ref().map(|e| (idx, e)))
    }

    /// Get entity reference by handle.
    pub(crate) fn entity(&self, idx: usize) -> Option<&Entity> {
        self.entities.get(idx)?.as_ref()
    }

    /// Take an entity out for mutation.
    pub(crate) fn take_entity(&mut self, idx: usize) -> Option<Entity> {
        self.entities.get_mut(idx)?.take()
    }

    /// Put an entity back after mutation.
    pub(crate) fn put_entity(&mut self, idx: usize, entity: Entity) {
        if idx >= self.entities.len() {
            self.entities.resize_with(idx + 1, || None);
        }
        self.entities[idx] = Some(entity);
    }

    /// Remove an entity.
    pub(crate) fn remove_entity(&mut self, idx: usize) {
        if let Some(slot) = self.entities.get_mut(idx) {
            *slot = None;
        }
    }

    /// Find entity at a position, optionally ignoring one handle.
    pub(crate) fn entity_at(
        &self,
        pos: Position,
        ignore: Option<usize>,
    ) -> Option<usize> {
        self.entities.iter().enumerate().find_map(|(idx, slot)| {
            if Some(idx) == ignore {
                return None;
            }
            slot.as_ref().and_then(|e| if e.pos == pos { Some(idx) } else { None })
        })
    }

    /// Damage an entity by handle.
    pub(crate) fn damage_entity(&mut self, idx: usize, amount: i32, rules: &GameRules) {
        if let Some(Some(entity)) = self.entities.get_mut(idx) {
            let def = rules.entity_def(entity.type_id);
            match def.behavior {
                // Entities with health get damaged
                EntityBehavior::Passive { .. }
                | EntityBehavior::Melee { .. }
                | EntityBehavior::Ranged { .. }
                | EntityBehavior::Growing { .. } => {
                    entity.health -= amount;
                }
                // Projectiles and statics are just removed
                EntityBehavior::Projectile { .. } | EntityBehavior::Static => {
                    self.entities[idx] = None;
                }
            }
        }
    }

    pub(crate) fn nearby_materials(&self, pos: Position, distance: usize) -> HashSet<MaterialId> {
        let mut result = HashSet::new();
        for x in pos[0].saturating_sub(distance)..=(pos[0] + distance).min(self.area[0] - 1) {
            for y in pos[1].saturating_sub(distance)..=(pos[1] + distance).min(self.area[1] - 1) {
                if let Some(material) = self.material([x, y]) {
                    result.insert(material);
                }
            }
        }
        result
    }

    pub(crate) fn adjacent_entity_types(&self, pos: Position) -> Vec<EntityTypeId> {
        let mut result = Vec::new();
        for direction in Direction::ALL {
            if let Some(target) = self.offset_pos(pos, direction.delta())
                && let Some(idx) = self.entity_at(target, None)
                && let Some(entity) = self.entity(idx)
            {
                result.push(entity.type_id);
            }
        }
        result
    }

    pub(crate) fn is_free_for(
        &self,
        pos: Position,
        walkable: &[MaterialId],
        player_pos: Position,
        ignore: Option<usize>,
    ) -> bool {
        self.material(pos)
            .map(|material| walkable.contains(&material))
            .unwrap_or(false)
            && self.entity_at(pos, ignore).is_none()
            && pos != player_pos
    }

    /// Get texture name for an entity (used by render).
    pub(crate) fn entity_texture_name(&self, idx: usize, rules: &GameRules) -> Option<String> {
        let entity = self.entity(idx)?;
        let def = rules.entity_def(entity.type_id);
        match &def.behavior {
            EntityBehavior::Projectile { .. } => {
                // Arrow-like: append facing direction
                let facing = match entity.facing {
                    Direction::Left => "left",
                    Direction::Right => "right",
                    Direction::Up => "up",
                    Direction::Down => "down",
                };
                Some(format!("{}-{}", def.name, facing))
            }
            EntityBehavior::Growing { ripen_time } => {
                if entity.timer > *ripen_time {
                    Some(format!("{}-ripe", def.name))
                } else {
                    Some(def.name.clone())
                }
            }
            _ => Some(def.name.clone()),
        }
    }

    fn index(&self, pos: Position) -> usize {
        pos[0] * self.area[1] + pos[1]
    }
}
