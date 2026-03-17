mod config;
mod entities;
mod opensimplex;
mod pillow_resize_16;
mod py_random;
#[cfg(feature = "python-module")]
mod python_api;
mod render;
mod runner;
mod types;
mod world;
mod worldgen;

use std::cell::Cell;
use std::collections::HashSet;

#[cfg(feature = "python-module")]
use pyo3::prelude::*;

pub use config::EnvConfig;
pub use entities::{Arrow, Cow, Fence, Plant, Player, Skeleton, Zombie};
pub use runner::{RunnerKey, runner_action_from_keys, runner_frame_to_buffer};
pub use types::{
    ACTION_NAMES, ACTIONS, Achievement, AchievementProgress, Action, Direction, Frame, ITEM_COUNT,
    ITEM_ORDER, Inventory, ItemKind, Material, Position, SemanticGrid, StepInfo, StepResult,
};
pub use world::World;

use crate::world::{ObjectHandle, ObjectKind};

const WALKABLE: [Material; 3] = [Material::Grass, Material::Path, Material::Sand];
const PLAYER_WALKABLE: [Material; 4] = [
    Material::Grass,
    Material::Path,
    Material::Sand,
    Material::Lava,
];
const ARROW_WALKABLE: [Material; 5] = [
    Material::Grass,
    Material::Path,
    Material::Sand,
    Material::Water,
    Material::Lava,
];
type ChunkKey = (usize, usize, usize, usize);

#[derive(Clone, Debug)]
pub struct Env {
    config: EnvConfig,
    episode: u64,
    step_count: u32,
    render_count: Cell<u64>,
    world: World,
    player: Player,
    last_health: i32,
    unlocked: HashSet<Achievement>,
}

impl Env {
    pub fn new(config: EnvConfig) -> Self {
        let center = [config.area[0] / 2, config.area[1] / 2];
        let player = Player::new(center);
        Self {
            world: World::new(config.area, [12, 12]),
            config,
            episode: 0,
            step_count: 0,
            render_count: Cell::new(0),
            last_health: player.health(),
            player,
            unlocked: HashSet::new(),
        }
    }

    pub fn reset(&mut self) -> Frame {
        let center = [self.config.area[0] / 2, self.config.area[1] / 2];
        self.episode += 1;
        self.step_count = 0;
        self.render_count.set(0);
        self.world
            .reset(episode_seed(self.config.seed, self.episode));
        self.update_time();
        self.player = Player::new(center);
        self.last_health = self.player.health();
        self.unlocked.clear();
        worldgen::generate_world(&mut self.world, center);
        self.observation()
    }

    pub fn step(&mut self, action: Action) -> StepResult {
        self.step_count += 1;
        self.update_time();
        self.player.set_action(action);
        self.update_player();

        let update_distance = 2 * self.config.view[0].max(self.config.view[1]);
        let handles = self.world.object_handles();
        for handle in handles {
            if let Some((pos, _)) = self.world.object_position_and_kind(handle)
                && manhattan_distance(self.player.pos(), pos) < update_distance as i32
            {
                self.update_object(handle);
            }
        }
        if self.step_count.is_multiple_of(10) {
            self.balance_chunks();
        }

        let obs = self.observation();
        let mut reward = (self.player.health() - self.last_health) as f32 / 10.0;
        self.last_health = self.player.health();

        for achievement in crate::types::ACHIEVEMENTS {
            if self.player.achievements().count(achievement) > 0
                && !self.unlocked.contains(&achievement)
            {
                self.unlocked.insert(achievement);
                reward += 1.0;
            }
        }

        let dead = self.player.health() <= 0;
        let over = self
            .config
            .length
            .map(|limit| self.step_count >= limit)
            .unwrap_or(false);
        let done = dead || over;
        let semantic = self.semantic_view();
        let info = StepInfo {
            inventory: self.player.inventory().clone(),
            achievements: self.player.achievements().clone(),
            discount: if dead { 0.0 } else { 1.0 },
            semantic,
            player_pos: self.player.pos(),
            reward,
        };
        StepResult {
            observation: obs,
            reward: if self.config.reward { reward } else { 0.0 },
            done,
            info,
        }
    }

    pub fn step_index(&mut self, action: usize) -> Option<StepResult> {
        Action::from_index(action).map(|action| self.step(action))
    }

    pub fn action_names(&self) -> &[&str; 17] {
        &ACTION_NAMES
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn player_mut(&mut self) -> &mut Player {
        &mut self.player
    }

    pub fn player_position(&self) -> Position {
        self.player.pos()
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn config(&self) -> &EnvConfig {
        &self.config
    }

    pub fn episode(&self) -> u64 {
        self.episode
    }

    pub fn step_count(&self) -> u32 {
        self.step_count
    }

    pub fn semantic_view(&self) -> SemanticGrid {
        self.world.semantic_view(&self.player)
    }

    pub fn render(&self, size: Option<[usize; 2]>) -> Frame {
        let noise_index = self.render_count.get();
        self.render_count.set(noise_index.wrapping_add(1));
        render::render(self, size, noise_index)
    }

    fn observation(&self) -> Frame {
        self.render(None)
    }

    fn update_time(&mut self) {
        let progress = (self.step_count as f32 / 300.0) % 1.0 + 0.3;
        let daylight = 1.0 - (std::f32::consts::PI * progress).cos().abs().powi(3);
        self.world.set_daylight(daylight);
    }

    fn update_player(&mut self) {
        let mut action = self.player.action();
        if self.player.sleeping() {
            if self.player.item(ItemKind::Energy) < 9 {
                action = Action::Sleep;
            } else {
                self.player.set_sleeping(false);
                self.player
                    .achievements_mut()
                    .increment(Achievement::WakeUp);
            }
        }

        match action {
            Action::Noop => {}
            Action::MoveLeft => self.move_player(Direction::Left),
            Action::MoveRight => self.move_player(Direction::Right),
            Action::MoveUp => self.move_player(Direction::Up),
            Action::MoveDown => self.move_player(Direction::Down),
            Action::Do => self.player_do(),
            Action::Sleep => {
                if self.player.item(ItemKind::Energy) < 9 {
                    self.player.set_sleeping(true);
                }
            }
            Action::PlaceStone => self.place_item(Material::Stone),
            Action::PlaceTable => self.place_item(Material::Table),
            Action::PlaceFurnace => self.place_item(Material::Furnace),
            Action::PlacePlant => self.place_plant(),
            Action::MakeWoodPickaxe => self.make_item(
                ItemKind::WoodPickaxe,
                &[(ItemKind::Wood, 1)],
                &[Material::Table],
                1,
            ),
            Action::MakeStonePickaxe => self.make_item(
                ItemKind::StonePickaxe,
                &[(ItemKind::Wood, 1), (ItemKind::Stone, 1)],
                &[Material::Table],
                1,
            ),
            Action::MakeIronPickaxe => self.make_item(
                ItemKind::IronPickaxe,
                &[
                    (ItemKind::Wood, 1),
                    (ItemKind::Coal, 1),
                    (ItemKind::Iron, 1),
                ],
                &[Material::Table, Material::Furnace],
                1,
            ),
            Action::MakeWoodSword => self.make_item(
                ItemKind::WoodSword,
                &[(ItemKind::Wood, 1)],
                &[Material::Table],
                1,
            ),
            Action::MakeStoneSword => self.make_item(
                ItemKind::StoneSword,
                &[(ItemKind::Wood, 1), (ItemKind::Stone, 1)],
                &[Material::Table],
                1,
            ),
            Action::MakeIronSword => self.make_item(
                ItemKind::IronSword,
                &[
                    (ItemKind::Wood, 1),
                    (ItemKind::Coal, 1),
                    (ItemKind::Iron, 1),
                ],
                &[Material::Table, Material::Furnace],
                1,
            ),
        }

        self.update_life_stats();
        self.degen_or_regen_health();
        self.player.clamp_inventory();
        self.wake_up_when_hurt();
    }

    fn move_player(&mut self, direction: Direction) {
        self.player.set_facing(direction);
        if let Some(target) = self.world.offset_pos(self.player.pos(), direction.delta())
            && self
                .world
                .is_free_for(target, &PLAYER_WALKABLE, self.player.pos(), None)
        {
            self.player.set_pos(target);
            if self.world.material(target) == Some(Material::Lava) {
                self.player.set_health(0);
            }
        }
    }

    fn player_do(&mut self) {
        let Some(target) = self
            .world
            .offset_pos(self.player.pos(), self.player.facing().delta())
        else {
            return;
        };
        if let Some(handle) = self.world.object_at(target, None) {
            self.do_object(handle);
        } else if let Some(material) = self.world.material(target) {
            self.do_material(target, material);
        }
    }

    fn do_object(&mut self, handle: ObjectHandle) {
        let damage = sword_damage(&self.player);
        match handle {
            ObjectHandle::Plant(idx) => {
                let Some(mut plant) = self.world.take_plant(idx) else {
                    return;
                };
                if plant.ripe() {
                    plant.grown = 0;
                    self.player.inventory_mut().add_item(ItemKind::Food, 4);
                    self.player
                        .achievements_mut()
                        .increment(Achievement::EatPlant);
                }
                self.world.put_plant(idx, plant);
            }
            ObjectHandle::Zombie(idx) => {
                self.attack_zombie(idx, damage);
            }
            ObjectHandle::Skeleton(idx) => {
                self.attack_skeleton(idx, damage);
            }
            ObjectHandle::Cow(idx) => {
                let Some(mut cow) = self.world.take_cow(idx) else {
                    return;
                };
                cow.health -= damage;
                if cow.health <= 0 {
                    self.player.inventory_mut().add_item(ItemKind::Food, 6);
                    self.player
                        .achievements_mut()
                        .increment(Achievement::EatCow);
                    *self.player.hunger_mut() = 0.0;
                } else {
                    self.world.put_cow(idx, cow);
                }
            }
            ObjectHandle::Fence(idx) => {
                self.world.remove_object(ObjectHandle::Fence(idx));
            }
            ObjectHandle::Arrow(idx) => {
                self.world.remove_object(ObjectHandle::Arrow(idx));
            }
        }
    }

    fn attack_zombie(&mut self, idx: usize, damage: i32) {
        let Some(mut zombie) = self.world.take_zombie(idx) else {
            return;
        };
        zombie.health -= damage;
        if zombie.health <= 0 {
            self.player
                .achievements_mut()
                .increment(Achievement::DefeatZombie);
        } else {
            self.world.put_zombie(idx, zombie);
        }
    }

    fn attack_skeleton(&mut self, idx: usize, damage: i32) {
        let Some(mut skeleton) = self.world.take_skeleton(idx) else {
            return;
        };
        skeleton.health -= damage;
        if skeleton.health <= 0 {
            self.player
                .achievements_mut()
                .increment(Achievement::DefeatSkeleton);
        } else {
            self.world.put_skeleton(idx, skeleton);
        }
    }

    fn do_material(&mut self, target: Position, material: Material) {
        if material == Material::Water {
            *self.player.thirst_mut() = 0.0;
        }
        match material {
            Material::Tree => {
                self.world.set_material(target, Material::Grass);
                self.collect(ItemKind::Wood, Achievement::CollectWood);
            }
            Material::Stone if self.player.item(ItemKind::WoodPickaxe) >= 1 => {
                self.mine(target, ItemKind::Stone, Achievement::CollectStone);
            }
            Material::Coal if self.player.item(ItemKind::WoodPickaxe) >= 1 => {
                self.mine(target, ItemKind::Coal, Achievement::CollectCoal);
            }
            Material::Iron if self.player.item(ItemKind::StonePickaxe) >= 1 => {
                self.mine(target, ItemKind::Iron, Achievement::CollectIron);
            }
            Material::Diamond if self.player.item(ItemKind::IronPickaxe) >= 1 => {
                self.mine(target, ItemKind::Diamond, Achievement::CollectDiamond);
            }
            Material::Water => {
                self.collect(ItemKind::Drink, Achievement::CollectDrink);
            }
            Material::Grass if self.world.random_f32() <= 0.1 => {
                self.collect(ItemKind::Sapling, Achievement::CollectSapling);
            }
            _ => {}
        }
    }

    fn mine(&mut self, target: Position, item: ItemKind, achievement: Achievement) {
        self.world.set_material(target, Material::Path);
        self.collect(item, achievement);
    }

    fn collect(&mut self, item: ItemKind, achievement: Achievement) {
        self.player.inventory_mut().add_item(item, 1);
        self.player.achievements_mut().increment(achievement);
    }

    fn place_item(&mut self, material: Material) {
        let Some(target) = self
            .world
            .offset_pos(self.player.pos(), self.player.facing().delta())
        else {
            return;
        };
        if self.world.object_at(target, None).is_some() {
            return;
        }
        let Some(target_material) = self.world.material(target) else {
            return;
        };
        match material {
            Material::Stone => {
                if !matches!(
                    target_material,
                    Material::Grass
                        | Material::Sand
                        | Material::Path
                        | Material::Water
                        | Material::Lava
                ) || self.player.item(ItemKind::Stone) < 1
                {
                    return;
                }
                self.player.inventory_mut().add_item(ItemKind::Stone, -1);
                self.world.set_material(target, Material::Stone);
                self.player
                    .achievements_mut()
                    .increment(Achievement::PlaceStone);
            }
            Material::Table => {
                if !matches!(
                    target_material,
                    Material::Grass | Material::Sand | Material::Path
                ) || self.player.item(ItemKind::Wood) < 2
                {
                    return;
                }
                self.player.inventory_mut().add_item(ItemKind::Wood, -2);
                self.world.set_material(target, Material::Table);
                self.player
                    .achievements_mut()
                    .increment(Achievement::PlaceTable);
            }
            Material::Furnace => {
                if !matches!(
                    target_material,
                    Material::Grass | Material::Sand | Material::Path
                ) || self.player.item(ItemKind::Stone) < 4
                {
                    return;
                }
                self.player.inventory_mut().add_item(ItemKind::Stone, -4);
                self.world.set_material(target, Material::Furnace);
                self.player
                    .achievements_mut()
                    .increment(Achievement::PlaceFurnace);
            }
            _ => {}
        }
    }

    fn place_plant(&mut self) {
        let Some(target) = self
            .world
            .offset_pos(self.player.pos(), self.player.facing().delta())
        else {
            return;
        };
        if self.world.object_at(target, None).is_some()
            || self.world.material(target) != Some(Material::Grass)
            || self.player.item(ItemKind::Sapling) < 1
        {
            return;
        }
        self.player.inventory_mut().add_item(ItemKind::Sapling, -1);
        self.world.spawn_plant(target);
        self.player
            .achievements_mut()
            .increment(Achievement::PlacePlant);
    }

    fn make_item(
        &mut self,
        item: ItemKind,
        uses: &[(ItemKind, i32)],
        nearby: &[Material],
        gives: i32,
    ) {
        let nearby_materials = self.world.nearby_materials(self.player.pos(), 1);
        if !nearby
            .iter()
            .all(|material| nearby_materials.contains(material))
        {
            return;
        }
        if uses
            .iter()
            .any(|(kind, amount)| self.player.item(*kind) < *amount)
        {
            return;
        }
        for (kind, amount) in uses {
            self.player.inventory_mut().add_item(*kind, -*amount);
        }
        self.player.inventory_mut().add_item(item, gives);
        self.player.achievements_mut().increment(match item {
            ItemKind::WoodPickaxe => Achievement::MakeWoodPickaxe,
            ItemKind::StonePickaxe => Achievement::MakeStonePickaxe,
            ItemKind::IronPickaxe => Achievement::MakeIronPickaxe,
            ItemKind::WoodSword => Achievement::MakeWoodSword,
            ItemKind::StoneSword => Achievement::MakeStoneSword,
            ItemKind::IronSword => Achievement::MakeIronSword,
            _ => return,
        });
    }

    fn update_life_stats(&mut self) {
        let sleep_factor = if self.player.sleeping() { 0.5 } else { 1.0 };
        *self.player.hunger_mut() += sleep_factor;
        if self.player.hunger() > 25.0 {
            *self.player.hunger_mut() = 0.0;
            self.player.inventory_mut().add_item(ItemKind::Food, -1);
        }
        *self.player.thirst_mut() += sleep_factor;
        if self.player.thirst() > 20.0 {
            *self.player.thirst_mut() = 0.0;
            self.player.inventory_mut().add_item(ItemKind::Drink, -1);
        }
        if self.player.sleeping() {
            *self.player.fatigue_mut() = (self.player.fatigue() - 1).min(0);
        } else {
            *self.player.fatigue_mut() += 1;
        }
        if self.player.fatigue() < -10 {
            *self.player.fatigue_mut() = 0;
            self.player.inventory_mut().add_item(ItemKind::Energy, 1);
        }
        if self.player.fatigue() > 30 {
            *self.player.fatigue_mut() = 0;
            self.player.inventory_mut().add_item(ItemKind::Energy, -1);
        }
    }

    fn degen_or_regen_health(&mut self) {
        let necessities = [
            self.player.item(ItemKind::Food) > 0,
            self.player.item(ItemKind::Drink) > 0,
            self.player.item(ItemKind::Energy) > 0 || self.player.sleeping(),
        ];
        if necessities.into_iter().all(|ok| ok) {
            *self.player.recover_mut() += if self.player.sleeping() { 2.0 } else { 1.0 };
        } else {
            *self.player.recover_mut() -= if self.player.sleeping() { 0.5 } else { 1.0 };
        }
        if self.player.recover() > 25.0 {
            *self.player.recover_mut() = 0.0;
            self.player.set_health(self.player.health() + 1);
        }
        if self.player.recover() < -15.0 {
            *self.player.recover_mut() = 0.0;
            self.player.set_health(self.player.health() - 1);
        }
    }

    fn wake_up_when_hurt(&mut self) {
        if self.player.health() < self.player.last_health() {
            self.player.set_sleeping(false);
        }
        self.player.set_last_health(self.player.health());
    }

    fn update_object(&mut self, handle: ObjectHandle) {
        match handle {
            ObjectHandle::Cow(idx) => self.update_cow(idx),
            ObjectHandle::Zombie(idx) => self.update_zombie(idx),
            ObjectHandle::Skeleton(idx) => self.update_skeleton(idx),
            ObjectHandle::Arrow(idx) => self.update_arrow(idx),
            ObjectHandle::Plant(idx) => self.update_plant(idx),
            ObjectHandle::Fence(idx) => self.update_fence(idx),
        }
    }

    fn balance_chunks(&mut self) {
        let mut seen = HashSet::new();
        let mut chunks = Vec::new();

        let player_chunk = chunk_key(&self.world, self.player.pos());
        seen.insert(player_chunk);
        chunks.push(player_chunk);

        for handle in self.world.object_handles() {
            if let Some((pos, _)) = self.world.object_position_and_kind(handle) {
                let chunk = chunk_key(&self.world, pos);
                if seen.insert(chunk) {
                    chunks.push(chunk);
                }
            }
        }

        for chunk in chunks {
            self.balance_chunk(chunk);
        }
    }

    fn balance_chunk(&mut self, chunk: ChunkKey) {
        let light = self.world.daylight();
        self.balance_object(
            chunk,
            ObjectKind::Zombie,
            Material::Grass,
            6,
            0,
            0.3,
            0.4,
            |space| {
                let target = if space < 50 { 0.0 } else { 3.5 - 3.0 * light };
                (target, 3.5 - 3.0 * light)
            },
        );
        self.balance_object(
            chunk,
            ObjectKind::Skeleton,
            Material::Path,
            7,
            7,
            0.1,
            0.1,
            |space| (if space < 6 { 0.0 } else { 1.0 }, 2.0),
        );
        self.balance_object(
            chunk,
            ObjectKind::Cow,
            Material::Grass,
            5,
            5,
            0.01,
            0.1,
            |space| (if space < 30 { 0.0 } else { 1.0 }, 1.5 + light),
        );
    }

    fn balance_object<F>(
        &mut self,
        chunk: ChunkKey,
        kind: ObjectKind,
        material: Material,
        spawn_dist: i32,
        despawn_dist: i32,
        spawn_prob: f32,
        despawn_prob: f32,
        target_fn: F,
    ) where
        F: Fn(usize) -> (f32, f32),
    {
        let positions = chunk_positions_with_material(&self.world, chunk, material);
        let creatures = self.chunk_creatures(chunk, kind);
        let (target_min, target_max) = target_fn(positions.len());

        if creatures.len() < target_min as usize && self.world.random_f32() < spawn_prob {
            if positions.is_empty() {
                return;
            }
            let pos = positions[self.world.random_usize(positions.len())];
            let empty = self.world.object_at(pos, None).is_none() && pos != self.player.pos();
            let away = manhattan_distance(self.player.pos(), pos) >= spawn_dist;
            if empty && away {
                self.spawn_object(kind, pos);
            }
        } else if creatures.len() > target_max as usize && self.world.random_f32() < despawn_prob {
            let handle = creatures[self.world.random_usize(creatures.len())];
            if let Some((pos, _)) = self.world.object_position_and_kind(handle)
                && manhattan_distance(self.player.pos(), pos) >= despawn_dist
            {
                self.world.remove_object(handle);
            }
        }
    }

    fn chunk_creatures(&self, chunk: ChunkKey, kind: ObjectKind) -> Vec<ObjectHandle> {
        self.world
            .object_handles()
            .into_iter()
            .filter(|handle| {
                self.world
                    .object_position_and_kind(*handle)
                    .map(|(pos, object_kind)| object_kind == kind && chunk_contains(chunk, pos))
                    .unwrap_or(false)
            })
            .collect()
    }

    fn spawn_object(&mut self, kind: ObjectKind, pos: Position) {
        match kind {
            ObjectKind::Cow => self.world.spawn_cow(pos),
            ObjectKind::Zombie => self.world.spawn_zombie(pos),
            ObjectKind::Skeleton => self.world.spawn_skeleton(pos),
            ObjectKind::Arrow | ObjectKind::Plant | ObjectKind::Fence => {}
        }
    }

    fn update_cow(&mut self, idx: usize) {
        let Some(mut cow) = self.world.take_cow(idx) else {
            return;
        };
        if cow.health <= 0 {
            return;
        }
        if self.world.random_f32() < 0.5 {
            let dir = random_direction(&mut self.world);
            self.try_move_object(&mut cow.pos, dir, ObjectHandle::Cow(idx), &WALKABLE);
        }
        self.world.put_cow(idx, cow);
    }

    fn update_zombie(&mut self, idx: usize) {
        let Some(mut zombie) = self.world.take_zombie(idx) else {
            return;
        };
        if zombie.health <= 0 {
            return;
        }
        let dist = manhattan_distance(zombie.pos, self.player.pos());
        if dist <= 8 && self.world.random_f32() < 0.9 {
            let dir = toward(zombie.pos, self.player.pos(), self.world.random_f32() < 0.8);
            self.try_move_object(&mut zombie.pos, dir, ObjectHandle::Zombie(idx), &WALKABLE);
        } else {
            let dir = random_direction(&mut self.world);
            self.try_move_object(&mut zombie.pos, dir, ObjectHandle::Zombie(idx), &WALKABLE);
        }
        let dist = manhattan_distance(zombie.pos, self.player.pos());
        if dist <= 1 {
            if zombie.cooldown > 0 {
                zombie.cooldown -= 1;
            } else {
                let damage = if self.player.sleeping() { 7 } else { 2 };
                self.player.set_health(self.player.health() - damage);
                zombie.cooldown = 5;
            }
        }
        self.world.put_zombie(idx, zombie);
    }

    fn update_skeleton(&mut self, idx: usize) {
        let Some(mut skeleton) = self.world.take_skeleton(idx) else {
            return;
        };
        if skeleton.health <= 0 {
            return;
        }
        skeleton.reload = (skeleton.reload - 1).max(0);
        let dist = manhattan_distance(skeleton.pos, self.player.pos());
        if dist <= 3 {
            let dir = toward(
                skeleton.pos,
                self.player.pos(),
                self.world.random_f32() < 0.6,
            )
            .opposite();
            if self.try_move_object(
                &mut skeleton.pos,
                dir,
                ObjectHandle::Skeleton(idx),
                &WALKABLE,
            ) {
                self.world.put_skeleton(idx, skeleton);
                return;
            }
        }
        if dist <= 5 && self.world.random_f32() < 0.5 {
            let dir = toward(skeleton.pos, self.player.pos(), true);
            self.shoot_arrow(skeleton.pos, dir, &mut skeleton.reload);
        } else if dist <= 8 && self.world.random_f32() < 0.3 {
            let dir = toward(
                skeleton.pos,
                self.player.pos(),
                self.world.random_f32() < 0.6,
            );
            self.try_move_object(
                &mut skeleton.pos,
                dir,
                ObjectHandle::Skeleton(idx),
                &WALKABLE,
            );
        } else if self.world.random_f32() < 0.2 {
            let dir = random_direction(&mut self.world);
            self.try_move_object(
                &mut skeleton.pos,
                dir,
                ObjectHandle::Skeleton(idx),
                &WALKABLE,
            );
        }
        self.world.put_skeleton(idx, skeleton);
    }

    fn shoot_arrow(&mut self, pos: Position, direction: Direction, reload: &mut i32) {
        if *reload > 0 {
            return;
        }
        let Some(target) = self.world.offset_pos(pos, direction.delta()) else {
            return;
        };
        if self
            .world
            .is_free_for(target, &ARROW_WALKABLE, self.player.pos(), None)
        {
            self.world.spawn_arrow(target, direction);
            *reload = 4;
        }
    }

    fn update_arrow(&mut self, idx: usize) {
        let Some(mut arrow) = self.world.take_arrow(idx) else {
            return;
        };
        let Some(target) = self.world.offset_pos(arrow.pos, arrow.facing.delta()) else {
            return;
        };
        if target == self.player.pos() {
            self.player.set_health(self.player.health() - 2);
            return;
        }
        if let Some(handle) = self.world.object_at(target, None) {
            self.world.damage_object(handle, 2);
            return;
        }
        let Some(material) = self.world.material(target) else {
            return;
        };
        if !ARROW_WALKABLE.contains(&material) {
            if matches!(material, Material::Table | Material::Furnace) {
                self.world.set_material(target, Material::Path);
            }
            return;
        }
        arrow.pos = target;
        self.world.put_arrow(idx, arrow);
    }

    fn update_plant(&mut self, idx: usize) {
        let Some(mut plant) = self.world.take_plant(idx) else {
            return;
        };
        plant.grown += 1;
        if self
            .world
            .adjacent_object_kinds(plant.pos)
            .into_iter()
            .any(|kind| {
                matches!(
                    kind,
                    ObjectKind::Zombie | ObjectKind::Skeleton | ObjectKind::Cow
                )
            })
        {
            plant.health -= 1;
        }
        if plant.health > 0 {
            self.world.put_plant(idx, plant);
        }
    }

    fn update_fence(&mut self, idx: usize) {
        if let Some(fence) = self.world.take_fence(idx) {
            self.world.put_fence(idx, fence);
        }
    }

    fn try_move_object(
        &self,
        pos: &mut Position,
        direction: Direction,
        handle: ObjectHandle,
        walkable: &[Material],
    ) -> bool {
        let Some(target) = self.world.offset_pos(*pos, direction.delta()) else {
            return false;
        };
        if self
            .world
            .is_free_for(target, walkable, self.player.pos(), Some(handle))
        {
            *pos = target;
            true
        } else {
            false
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new(EnvConfig::default())
    }
}

fn manhattan_distance(lhs: Position, rhs: Position) -> i32 {
    (lhs[0] as i32 - rhs[0] as i32).abs() + (lhs[1] as i32 - rhs[1] as i32).abs()
}

fn toward(from: Position, target: Position, long_axis: bool) -> Direction {
    let dx = target[0] as i32 - from[0] as i32;
    let dy = target[1] as i32 - from[1] as i32;
    if dx == 0 && dy == 0 {
        return Direction::Down;
    }
    if (dx.abs() > dy.abs()) == long_axis {
        if dx < 0 {
            Direction::Left
        } else {
            Direction::Right
        }
    } else if dy < 0 {
        Direction::Up
    } else {
        Direction::Down
    }
}

fn random_direction(world: &mut World) -> Direction {
    Direction::ALL[world.random_usize(Direction::ALL.len())]
}

fn chunk_key(world: &World, pos: Position) -> ChunkKey {
    let [chunk_width, chunk_height] = world.chunk_size();
    let [area_width, area_height] = world.area();
    let xmin = (pos[0] / chunk_width) * chunk_width;
    let ymin = (pos[1] / chunk_height) * chunk_height;
    let xmax = (xmin + chunk_width).min(area_width);
    let ymax = (ymin + chunk_height).min(area_height);
    (xmin, xmax, ymin, ymax)
}

fn chunk_contains(chunk: ChunkKey, pos: Position) -> bool {
    let (xmin, xmax, ymin, ymax) = chunk;
    xmin <= pos[0] && pos[0] < xmax && ymin <= pos[1] && pos[1] < ymax
}

fn chunk_positions_with_material(world: &World, chunk: ChunkKey, material: Material) -> Vec<Position> {
    let (xmin, xmax, ymin, ymax) = chunk;
    let mut positions = Vec::new();
    for x in xmin..xmax {
        for y in ymin..ymax {
            if world.material([x, y]) == Some(material) {
                positions.push([x, y]);
            }
        }
    }
    positions
}

fn sword_damage(player: &Player) -> i32 {
    [
        1,
        if player.item(ItemKind::WoodSword) > 0 {
            2
        } else {
            0
        },
        if player.item(ItemKind::StoneSword) > 0 {
            3
        } else {
            0
        },
        if player.item(ItemKind::IronSword) > 0 {
            5
        } else {
            0
        },
    ]
    .into_iter()
    .max()
    .unwrap_or(1)
}

#[cfg(feature = "python-module")]
#[pymodule(name = "_core")]
fn crafters_python_module(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    python_api::register(module)
}

fn episode_seed(seed: u64, episode: u64) -> u64 {
    python_tuple_hash_pair(seed, episode).rem_euclid(((1_u64 << 31) - 1) as i64) as u64
}

fn python_tuple_hash_pair(lhs: u64, rhs: u64) -> i64 {
    const XXPRIME_1: u64 = 11_400_714_785_074_694_791;
    const XXPRIME_2: u64 = 14_029_467_366_897_019_727;
    const XXPRIME_5: u64 = 2_870_177_450_012_600_261;
    const TUPLE_HASH_XXPRIME_5: u64 = 3_527_539;

    let mut acc = XXPRIME_5;
    for lane in [python_int_hash_u64(lhs) as u64, python_int_hash_u64(rhs) as u64] {
        acc = acc.wrapping_add(lane.wrapping_mul(XXPRIME_2));
        acc = acc.rotate_left(31);
        acc = acc.wrapping_mul(XXPRIME_1);
    }
    acc = acc.wrapping_add(2 ^ (XXPRIME_5 ^ TUPLE_HASH_XXPRIME_5));
    if acc == u64::MAX {
        1_546_275_796
    } else {
        acc as i64
    }
}

fn python_int_hash_u64(value: u64) -> i64 {
    const PY_HASH_MODULUS: u64 = (1_u64 << 61) - 1;
    (value % PY_HASH_MODULUS) as i64
}

#[cfg(test)]
mod tests {
    use super::episode_seed;

    #[test]
    fn episode_seed_matches_python_tuple_hash_reference() {
        let cases = [
            ((0, 1), 1_256_191_933),
            ((1, 1), 535_231_920),
            ((7, 3), 693_148_721),
            ((123, 456), 1_039_445_943),
            ((2_147_483_646, 9), 390_076_502),
        ];
        for ((seed, episode), expected) in cases {
            assert_eq!(episode_seed(seed, episode), expected);
        }
    }
}
