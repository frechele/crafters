use std::collections::HashSet;

use crate::entities::{Arrow, Cow, Fence, Plant, Player, Skeleton, Zombie};
use crate::py_random::PyRandom;
use crate::{Direction, Material, Position, SemanticGrid};

macro_rules! object_storage {
    ($take:ident, $put:ident, $field:ident, $type:ty) => {
        pub(crate) fn $take(&mut self, idx: usize) -> Option<$type> {
            self.$field.get_mut(idx)?.take()
        }

        pub(crate) fn $put(&mut self, idx: usize, value: $type) {
            if idx >= self.$field.len() {
                self.$field.resize_with(idx + 1, || None);
            }
            self.$field[idx] = Some(value);
        }
    };
}

macro_rules! collect_handles {
    ($handles:expr, $self:expr, $(($field:ident, $variant:ident)),+) => {
        $(
            $handles.extend(
                $self.$field
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, obj)| obj.as_ref().map(|_| ObjectHandle::$variant(idx))),
            );
        )+
    };
}

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
    rng: PyRandom,
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
            rng: PyRandom::new(0),
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
        self.rng = PyRandom::new(seed as u32);
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

    pub(crate) fn object_handles(&self) -> Vec<ObjectHandle> {
        let mut handles = Vec::new();
        collect_handles!(
            handles, self,
            (cows, Cow),
            (zombies, Zombie),
            (skeletons, Skeleton),
            (arrows, Arrow),
            (plants, Plant),
            (fences, Fence)
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
            if let Some(target) = self.offset_pos(pos, direction.delta())
                && let Some(handle) = self.object_at(target, None)
                && let Some((_, kind)) = self.object_position_and_kind(handle)
            {
                result.push(kind);
            }
        }
        result
    }

    object_storage!(take_cow, put_cow, cows, Cow);
    object_storage!(take_zombie, put_zombie, zombies, Zombie);
    object_storage!(take_skeleton, put_skeleton, skeletons, Skeleton);
    object_storage!(take_arrow, put_arrow, arrows, Arrow);
    object_storage!(take_plant, put_plant, plants, Plant);
    object_storage!(take_fence, put_fence, fences, Fence);

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
        macro_rules! remove {
            ($field:ident, $idx:expr) => {
                if let Some(slot) = self.$field.get_mut($idx) {
                    *slot = None;
                }
            };
        }
        match handle {
            ObjectHandle::Cow(idx) => remove!(cows, idx),
            ObjectHandle::Zombie(idx) => remove!(zombies, idx),
            ObjectHandle::Skeleton(idx) => remove!(skeletons, idx),
            ObjectHandle::Arrow(idx) => remove!(arrows, idx),
            ObjectHandle::Plant(idx) => remove!(plants, idx),
            ObjectHandle::Fence(idx) => remove!(fences, idx),
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
