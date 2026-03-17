use std::collections::HashSet;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::entities::{Arrow, Cow, Fence, Plant, Player, Skeleton, Zombie};
use crate::{Direction, Material, Position, SemanticGrid};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObjectHandle {
    Cow(usize),
    Zombie(usize),
    Skeleton(usize),
    Arrow(usize),
    Plant(usize),
    Fence(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ObjectKind {
    Cow,
    Zombie,
    Skeleton,
    Arrow,
    Plant,
    Fence,
}

#[derive(Clone, Debug)]
pub struct World {
    area: [usize; 2],
    chunk_size: [usize; 2],
    materials: Vec<Option<Material>>,
    daylight: f32,
    rng: ChaCha8Rng,
    cows: Vec<Option<Cow>>,
    zombies: Vec<Option<Zombie>>,
    skeletons: Vec<Option<Skeleton>>,
    arrows: Vec<Option<Arrow>>,
    plants: Vec<Option<Plant>>,
    fences: Vec<Option<Fence>>,
}

impl World {
    pub fn new(area: [usize; 2], chunk_size: [usize; 2]) -> Self {
        Self {
            area,
            chunk_size,
            materials: vec![None; area[0] * area[1]],
            daylight: 0.0,
            rng: ChaCha8Rng::seed_from_u64(0),
            cows: Vec::new(),
            zombies: Vec::new(),
            skeletons: Vec::new(),
            arrows: Vec::new(),
            plants: Vec::new(),
            fences: Vec::new(),
        }
    }

    pub fn reset(&mut self, seed: u64) {
        self.materials.fill(None);
        self.daylight = 0.0;
        self.rng = ChaCha8Rng::seed_from_u64(seed);
        self.clear_objects();
    }

    pub fn area(&self) -> [usize; 2] {
        self.area
    }

    pub fn chunk_size(&self) -> [usize; 2] {
        self.chunk_size
    }

    pub fn material(&self, pos: Position) -> Option<Material> {
        self.materials[self.index(pos)]
    }

    pub fn set_material(&mut self, pos: Position, material: Material) {
        let index = self.index(pos);
        self.materials[index] = Some(material);
    }

    pub fn fill(&mut self, material: Material) {
        self.materials.fill(Some(material));
    }

    pub fn iter_materials(&self) -> impl Iterator<Item = Option<Material>> + '_ {
        self.materials.iter().copied()
    }

    pub fn clear_objects(&mut self) {
        self.cows.clear();
        self.zombies.clear();
        self.skeletons.clear();
        self.arrows.clear();
        self.plants.clear();
        self.fences.clear();
    }

    pub fn spawn_cow(&mut self, pos: Position) {
        if self.object_at(pos, None).is_none() {
            self.cows.push(Some(Cow::new(pos)));
        }
    }

    pub fn spawn_zombie(&mut self, pos: Position) {
        if self.object_at(pos, None).is_none() {
            self.zombies.push(Some(Zombie::new(pos)));
        }
    }

    pub fn spawn_skeleton(&mut self, pos: Position) {
        if self.object_at(pos, None).is_none() {
            self.skeletons.push(Some(Skeleton::new(pos)));
        }
    }

    pub fn spawn_plant(&mut self, pos: Position) {
        if self.object_at(pos, None).is_none() {
            self.plants.push(Some(Plant::new(pos)));
        }
    }

    pub fn spawn_arrow(&mut self, pos: Position, facing: Direction) {
        if self.object_at(pos, None).is_none() {
            self.arrows.push(Some(Arrow::new(pos, facing)));
        }
    }

    pub fn arrow_count(&self) -> usize {
        self.arrows.iter().flatten().count()
    }

    pub fn semantic_view(&self, player: &Player) -> SemanticGrid {
        let mut cells: Vec<u16> = self
            .materials
            .iter()
            .map(|material| material.map(Material::id).unwrap_or_default())
            .collect();
        for handle in self.object_handles() {
            if let Some((pos, kind)) = self.object_position_and_kind(handle) {
                cells[self.index(pos)] = semantic_id(kind);
            }
        }
        cells[self.index(player.pos())] = semantic_id_player();
        SemanticGrid::new(self.area[0], self.area[1], cells)
    }

    pub fn random_f32(&mut self) -> f32 {
        self.rng.random::<f32>()
    }

    pub fn random_usize(&mut self, upper_exclusive: usize) -> usize {
        self.rng.random_range(0..upper_exclusive)
    }

    pub fn random_bool(&mut self, probability: f32) -> bool {
        self.random_f32() < probability
    }

    pub fn random_u32(&mut self) -> u32 {
        self.rng.random::<u32>()
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

    pub(crate) fn object_handles(&self) -> Vec<ObjectHandle> {
        let mut handles = Vec::new();
        handles.extend(
            self.cows
                .iter()
                .enumerate()
                .filter_map(|(idx, obj)| obj.as_ref().map(|_| ObjectHandle::Cow(idx))),
        );
        handles.extend(
            self.zombies
                .iter()
                .enumerate()
                .filter_map(|(idx, obj)| obj.as_ref().map(|_| ObjectHandle::Zombie(idx))),
        );
        handles.extend(
            self.skeletons
                .iter()
                .enumerate()
                .filter_map(|(idx, obj)| obj.as_ref().map(|_| ObjectHandle::Skeleton(idx))),
        );
        handles.extend(
            self.arrows
                .iter()
                .enumerate()
                .filter_map(|(idx, obj)| obj.as_ref().map(|_| ObjectHandle::Arrow(idx))),
        );
        handles.extend(
            self.plants
                .iter()
                .enumerate()
                .filter_map(|(idx, obj)| obj.as_ref().map(|_| ObjectHandle::Plant(idx))),
        );
        handles.extend(
            self.fences
                .iter()
                .enumerate()
                .filter_map(|(idx, obj)| obj.as_ref().map(|_| ObjectHandle::Fence(idx))),
        );
        handles
    }

    pub(crate) fn object_position_and_kind(
        &self,
        handle: ObjectHandle,
    ) -> Option<(Position, ObjectKind)> {
        match handle {
            ObjectHandle::Cow(idx) => self
                .cows
                .get(idx)?
                .as_ref()
                .map(|obj| (obj.pos, ObjectKind::Cow)),
            ObjectHandle::Zombie(idx) => self
                .zombies
                .get(idx)?
                .as_ref()
                .map(|obj| (obj.pos, ObjectKind::Zombie)),
            ObjectHandle::Skeleton(idx) => self
                .skeletons
                .get(idx)?
                .as_ref()
                .map(|obj| (obj.pos, ObjectKind::Skeleton)),
            ObjectHandle::Arrow(idx) => self
                .arrows
                .get(idx)?
                .as_ref()
                .map(|obj| (obj.pos, ObjectKind::Arrow)),
            ObjectHandle::Plant(idx) => self
                .plants
                .get(idx)?
                .as_ref()
                .map(|obj| (obj.pos, ObjectKind::Plant)),
            ObjectHandle::Fence(idx) => self
                .fences
                .get(idx)?
                .as_ref()
                .map(|obj| (obj.pos, ObjectKind::Fence)),
        }
    }

    pub(crate) fn object_texture_name(&self, handle: ObjectHandle) -> Option<&'static str> {
        match handle {
            ObjectHandle::Cow(idx) => self.cows.get(idx)?.as_ref().map(|_| "cow"),
            ObjectHandle::Zombie(idx) => self.zombies.get(idx)?.as_ref().map(|_| "zombie"),
            ObjectHandle::Skeleton(idx) => self.skeletons.get(idx)?.as_ref().map(|_| "skeleton"),
            ObjectHandle::Arrow(idx) => self
                .arrows
                .get(idx)?
                .as_ref()
                .map(crate::entities::Arrow::texture_name),
            ObjectHandle::Plant(idx) => self
                .plants
                .get(idx)?
                .as_ref()
                .map(crate::entities::Plant::texture_name),
            ObjectHandle::Fence(idx) => self.fences.get(idx)?.as_ref().map(|_| "fence"),
        }
    }

    pub(crate) fn object_at(
        &self,
        pos: Position,
        ignore: Option<ObjectHandle>,
    ) -> Option<ObjectHandle> {
        self.object_handles().into_iter().find(|handle| {
            if Some(*handle) == ignore {
                return false;
            }
            self.object_position_and_kind(*handle)
                .map(|(object_pos, _)| object_pos == pos)
                .unwrap_or(false)
        })
    }

    pub(crate) fn nearby_materials(&self, pos: Position, distance: usize) -> HashSet<Material> {
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

    pub(crate) fn adjacent_object_kinds(&self, pos: Position) -> Vec<ObjectKind> {
        let mut result = Vec::new();
        for direction in Direction::ALL {
            if let Some(target) = self.offset_pos(pos, direction.delta()) {
                if let Some(handle) = self.object_at(target, None) {
                    if let Some((_, kind)) = self.object_position_and_kind(handle) {
                        result.push(kind);
                    }
                }
            }
        }
        result
    }

    pub(crate) fn take_cow(&mut self, idx: usize) -> Option<Cow> {
        self.cows.get_mut(idx)?.take()
    }

    pub(crate) fn put_cow(&mut self, idx: usize, value: Cow) {
        if idx >= self.cows.len() {
            self.cows.resize_with(idx + 1, || None);
        }
        self.cows[idx] = Some(value);
    }

    pub(crate) fn take_zombie(&mut self, idx: usize) -> Option<Zombie> {
        self.zombies.get_mut(idx)?.take()
    }

    pub(crate) fn put_zombie(&mut self, idx: usize, value: Zombie) {
        if idx >= self.zombies.len() {
            self.zombies.resize_with(idx + 1, || None);
        }
        self.zombies[idx] = Some(value);
    }

    pub(crate) fn take_skeleton(&mut self, idx: usize) -> Option<Skeleton> {
        self.skeletons.get_mut(idx)?.take()
    }

    pub(crate) fn put_skeleton(&mut self, idx: usize, value: Skeleton) {
        if idx >= self.skeletons.len() {
            self.skeletons.resize_with(idx + 1, || None);
        }
        self.skeletons[idx] = Some(value);
    }

    pub(crate) fn take_arrow(&mut self, idx: usize) -> Option<Arrow> {
        self.arrows.get_mut(idx)?.take()
    }

    pub(crate) fn put_arrow(&mut self, idx: usize, value: Arrow) {
        if idx >= self.arrows.len() {
            self.arrows.resize_with(idx + 1, || None);
        }
        self.arrows[idx] = Some(value);
    }

    pub(crate) fn take_plant(&mut self, idx: usize) -> Option<Plant> {
        self.plants.get_mut(idx)?.take()
    }

    pub(crate) fn put_plant(&mut self, idx: usize, value: Plant) {
        if idx >= self.plants.len() {
            self.plants.resize_with(idx + 1, || None);
        }
        self.plants[idx] = Some(value);
    }

    pub(crate) fn take_fence(&mut self, idx: usize) -> Option<Fence> {
        self.fences.get_mut(idx)?.take()
    }

    pub(crate) fn put_fence(&mut self, idx: usize, value: Fence) {
        if idx >= self.fences.len() {
            self.fences.resize_with(idx + 1, || None);
        }
        self.fences[idx] = Some(value);
    }

    pub(crate) fn damage_object(&mut self, handle: ObjectHandle, amount: i32) {
        match handle {
            ObjectHandle::Cow(idx) => {
                if let Some(Some(obj)) = self.cows.get_mut(idx) {
                    obj.health -= amount;
                }
            }
            ObjectHandle::Zombie(idx) => {
                if let Some(Some(obj)) = self.zombies.get_mut(idx) {
                    obj.health -= amount;
                }
            }
            ObjectHandle::Skeleton(idx) => {
                if let Some(Some(obj)) = self.skeletons.get_mut(idx) {
                    obj.health -= amount;
                }
            }
            ObjectHandle::Plant(idx) => {
                if let Some(Some(obj)) = self.plants.get_mut(idx) {
                    obj.health -= amount;
                }
            }
            ObjectHandle::Arrow(idx) => {
                self.remove_object(ObjectHandle::Arrow(idx));
            }
            ObjectHandle::Fence(idx) => {
                self.remove_object(ObjectHandle::Fence(idx));
            }
        }
    }

    pub(crate) fn remove_object(&mut self, handle: ObjectHandle) {
        match handle {
            ObjectHandle::Cow(idx) => {
                if let Some(slot) = self.cows.get_mut(idx) {
                    *slot = None;
                }
            }
            ObjectHandle::Zombie(idx) => {
                if let Some(slot) = self.zombies.get_mut(idx) {
                    *slot = None;
                }
            }
            ObjectHandle::Skeleton(idx) => {
                if let Some(slot) = self.skeletons.get_mut(idx) {
                    *slot = None;
                }
            }
            ObjectHandle::Arrow(idx) => {
                if let Some(slot) = self.arrows.get_mut(idx) {
                    *slot = None;
                }
            }
            ObjectHandle::Plant(idx) => {
                if let Some(slot) = self.plants.get_mut(idx) {
                    *slot = None;
                }
            }
            ObjectHandle::Fence(idx) => {
                if let Some(slot) = self.fences.get_mut(idx) {
                    *slot = None;
                }
            }
        }
    }

    pub(crate) fn is_free_for(
        &self,
        pos: Position,
        walkable: &[Material],
        player_pos: Position,
        ignore: Option<ObjectHandle>,
    ) -> bool {
        self.material(pos)
            .map(|material| walkable.contains(&material))
            .unwrap_or(false)
            && self.object_at(pos, ignore).is_none()
            && pos != player_pos
    }

    fn index(&self, pos: Position) -> usize {
        pos[0] * self.area[1] + pos[1]
    }
}

fn semantic_id(kind: ObjectKind) -> u16 {
    match kind {
        ObjectKind::Cow => 14,
        ObjectKind::Zombie => 15,
        ObjectKind::Skeleton => 16,
        ObjectKind::Arrow => 17,
        ObjectKind::Plant => 18,
        ObjectKind::Fence => 19,
    }
}

fn semantic_id_player() -> u16 {
    13
}
