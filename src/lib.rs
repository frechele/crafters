mod config;
mod entities;
mod game_rules;
mod opensimplex;
mod pillow_resize_16;
mod py_random;
#[cfg(feature = "python-module")]
mod python_api;
pub mod registry;
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
pub use entities::Player;
pub use game_rules::{GameRules, WellKnownItems};
pub use runner::{RunnerKey, runner_action_from_keys, runner_frame_to_buffer};
pub use registry::{EntityTypeId, ItemId, MaterialId};
pub use types::{
    AchievementProgress, Direction, Frame, Inventory, ItemKind, Material, Position, SemanticGrid,
    StepInfo, StepResult,
};
pub use world::World;

use crate::game_rules::{EntityBehavior, PlaceType, SpawnConfig, spawn_targets};
use crate::registry::EntityTypeId as ETypeId;
type ChunkKey = (usize, usize, usize, usize);

#[derive(Clone, Debug)]
pub struct Env {
    config: EnvConfig,
    rules: GameRules,
    episode: u64,
    step_count: u32,
    render_count: Cell<u64>,
    world: World,
    player: Player,
    last_health: i32,
    unlocked: HashSet<usize>,
}

impl Env {
    pub fn new(config: EnvConfig) -> Self {
        Self::with_rules(config, GameRules::default())
    }

    pub fn with_rules(config: EnvConfig, rules: GameRules) -> Self {
        let center = [config.area[0] / 2, config.area[1] / 2];
        let inventory = Inventory::from_initial(&rules.item_initial);
        let ach_count = rules.achievement_count();
        let health_id = rules.well_known.health;
        let player = Player::with_inventory(center, inventory, ach_count, health_id);
        Self {
            world: World::new(config.area, [12, 12]),
            config,
            rules,
            episode: 0,
            step_count: 0,
            render_count: Cell::new(0),
            last_health: player.health(),
            player,
            unlocked: HashSet::new(),
        }
    }

    pub fn rules(&self) -> &GameRules {
        &self.rules
    }

    pub fn reset(&mut self) -> Frame {
        let center = [self.config.area[0] / 2, self.config.area[1] / 2];
        self.episode += 1;
        self.step_count = 0;
        self.render_count.set(0);
        self.world
            .reset(episode_seed(self.config.seed, self.episode));
        self.update_time();
        let inventory = Inventory::from_initial(&self.rules.item_initial);
        let ach_count = self.rules.achievement_count();
        let health_id = self.rules.well_known.health;
        self.player = Player::with_inventory(center, inventory, ach_count, health_id);
        self.last_health = self.player.health();
        self.unlocked.clear();
        worldgen::generate_world(&mut self.world, center, &self.rules);
        self.observation()
    }

    pub fn step_by_name(&mut self, name: &str) -> Option<StepResult> {
        let index = self.rules.action_index(name)?;
        Some(self.step_by_index(index))
    }

    pub fn step_by_index(&mut self, action_index: usize) -> StepResult {
        self.step_count += 1;
        self.update_time();
        self.update_player(action_index);

        let update_distance = 2 * self.config.view[0].max(self.config.view[1]);
        let handles = self.world.entity_handles();
        for idx in handles {
            if let Some(entity) = self.world.entity(idx) {
                if manhattan_distance(self.player.pos(), entity.pos) < update_distance as i32 {
                    self.update_entity(idx);
                }
            }
        }
        if self.step_count.is_multiple_of(10) {
            self.balance_chunks();
        }

        let obs = self.observation();
        let mut reward = (self.player.health() - self.last_health) as f32 / 10.0;
        self.last_health = self.player.health();

        let achievement_count = self.rules.achievement_count();
        for i in 0..achievement_count {
            if self.player.achievements().count(i) > 0 && !self.unlocked.contains(&i) {
                self.unlocked.insert(i);
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
        if action < self.rules.action_count() {
            Some(self.step_by_index(action))
        } else {
            None
        }
    }

    pub fn action_names(&self) -> Vec<&str> {
        self.rules.actions.iter().map(|a| a.name.as_str()).collect()
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
        self.world.semantic_view(&self.player, &self.rules)
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
        let progress =
            (self.step_count as f32 / self.rules.balance.daylight_cycle) % 1.0 + 0.3;
        let daylight = 1.0 - (std::f32::consts::PI * progress).cos().abs().powi(3);
        self.world.set_daylight(daylight);
    }

    fn update_player(&mut self, mut action_index: usize) {
        if self.player.sleeping() {
            if self.player.item(self.rules.well_known.energy) < 9 {
                action_index = self.rules.sleep_action_index;
            } else {
                self.player.set_sleeping(false);
                let wake_up = self.rules.achievement_index("wake_up");
                self.player.achievements_mut().increment(wake_up);
            }
        }

        let action_kind = self.rules.actions[action_index].kind.clone();
        match action_kind {
            game_rules::ActionKind::Noop => {}
            game_rules::ActionKind::Move(dir) => self.move_player(dir),
            game_rules::ActionKind::Do => self.player_do(),
            game_rules::ActionKind::Sleep => {
                if self.player.item(self.rules.well_known.energy) < 9 {
                    self.player.set_sleeping(true);
                }
            }
            game_rules::ActionKind::Place(name) => self.place_by_name(&name),
            game_rules::ActionKind::Make(item_id) => self.make_from_rules(item_id),
        }

        self.update_life_stats();
        self.degen_or_regen_health();
        self.player.inventory_mut().clamp_with(&self.rules.item_max);
        self.wake_up_when_hurt();
    }

    fn move_player(&mut self, direction: Direction) {
        self.player.set_facing(direction);
        if let Some(target) = self.world.offset_pos(self.player.pos(), direction.delta())
            && self
                .world
                .is_free_for(target, &self.rules.player_walkable, self.player.pos(), None)
        {
            self.player.set_pos(target);
            if self.world.material(target) == Some(Material::Lava.id()) {
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
        if let Some(idx) = self.world.entity_at(target, None) {
            self.do_entity(idx);
        } else if let Some(material) = self.world.material(target) {
            self.do_material(target, material);
        }
    }

    fn do_entity(&mut self, idx: usize) {
        let Some(entity) = self.world.entity(idx) else {
            return;
        };
        let def = self.rules.entity_def(entity.type_id).clone();
        let damage = sword_damage(&self.player, &self.rules.balance.combat);

        match &def.behavior {
            EntityBehavior::Growing { ripen_time } => {
                let Some(mut entity) = self.world.take_entity(idx) else {
                    return;
                };
                if entity.timer > *ripen_time {
                    entity.timer = 0;
                    for &(item, amount) in &def.drops {
                        self.player.inventory_mut().add_item(item, amount);
                    }
                    if let Some(achievement) = def.on_kill {
                        self.player.achievements_mut().increment(achievement);
                    }
                }
                self.world.put_entity(idx, entity);
            }
            EntityBehavior::Passive { .. } => {
                let Some(mut entity) = self.world.take_entity(idx) else {
                    return;
                };
                entity.health -= damage;
                if entity.health <= 0 {
                    for &(item, amount) in &def.drops {
                        self.player.inventory_mut().add_item(item, amount);
                    }
                    if let Some(achievement) = def.on_kill {
                        self.player.achievements_mut().increment(achievement);
                    }
                    *self.player.hunger_mut() = 0.0;
                } else {
                    self.world.put_entity(idx, entity);
                }
            }
            EntityBehavior::Melee { .. } | EntityBehavior::Ranged { .. } => {
                let Some(mut entity) = self.world.take_entity(idx) else {
                    return;
                };
                entity.health -= damage;
                if entity.health <= 0 {
                    if let Some(achievement) = def.on_kill {
                        self.player.achievements_mut().increment(achievement);
                    }
                } else {
                    self.world.put_entity(idx, entity);
                }
            }
            EntityBehavior::Projectile { .. } | EntityBehavior::Static => {
                self.world.remove_entity(idx);
            }
        }
    }

    fn do_material(&mut self, target: Position, material: MaterialId) {
        // Special behavior: interacting with water resets thirst
        if material == Material::Water.id() {
            *self.player.thirst_mut() = 0.0;
        }
        let Some(rule) = self.rules.collect.get(&material).cloned() else {
            return;
        };
        // Check requirements
        for &(kind, amount) in &rule.require {
            if self.player.item(kind) < amount {
                return;
            }
        }
        // Check probability
        if rule.probability < 1.0 && self.world.random_f32() > rule.probability {
            return;
        }
        // Transform material
        self.world.set_material(target, rule.leaves);
        // Give items and record achievement
        for &(kind, amount) in &rule.receive {
            self.player.inventory_mut().add_item(kind, amount);
        }
        self.player.achievements_mut().increment(rule.achievement);
    }

    fn place_by_name(&mut self, name: &str) {
        let Some(rule) = self.rules.place.get(name).cloned() else {
            return;
        };
        let Some(target) = self
            .world
            .offset_pos(self.player.pos(), self.player.facing().delta())
        else {
            return;
        };
        if self.world.entity_at(target, None).is_some() {
            return;
        }
        let Some(target_material) = self.world.material(target) else {
            return;
        };
        if !rule.where_materials.contains(&target_material) {
            return;
        }
        // Check resource cost
        for &(kind, amount) in &rule.uses {
            if self.player.item(kind) < amount {
                return;
            }
        }
        // Deduct resources
        for &(kind, amount) in &rule.uses {
            self.player.inventory_mut().add_item(kind, -amount);
        }
        // Place
        match rule.place_type {
            PlaceType::Material => {
                if let Some(material) = rule.placed_material {
                    self.world.set_material(target, material);
                }
            }
            PlaceType::Object => {
                // Find the plant entity type
                let plant_id = self.rules.entity_type_id("plant")
                    .expect("plant entity type must exist");
                let plant_def = self.rules.entity_def(plant_id);
                self.world.spawn_entity(target, plant_id, plant_def.health);
            }
        }
        self.player.achievements_mut().increment(rule.achievement);
    }

    fn make_from_rules(&mut self, item: ItemId) {
        let Some(rule) = self.rules.make.get(&item).cloned() else {
            return;
        };
        let nearby_materials = self.world.nearby_materials(self.player.pos(), 1);
        if !rule
            .nearby
            .iter()
            .all(|material| nearby_materials.contains(material))
        {
            return;
        }
        for &(kind, amount) in &rule.uses {
            if self.player.item(kind) < amount {
                return;
            }
        }
        for &(kind, amount) in &rule.uses {
            self.player.inventory_mut().add_item(kind, -amount);
        }
        self.player.inventory_mut().add_item(item, rule.gives);
        self.player.achievements_mut().increment(rule.achievement);
    }

    fn update_life_stats(&mut self) {
        let pb = &self.rules.balance.player;
        let wk = &self.rules.well_known;
        let sleep_factor = if self.player.sleeping() {
            pb.sleep_consumption_factor
        } else {
            1.0
        };
        *self.player.hunger_mut() += sleep_factor;
        if self.player.hunger() > pb.hunger_threshold {
            *self.player.hunger_mut() = 0.0;
            self.player.inventory_mut().add_item(wk.food, -1);
        }
        *self.player.thirst_mut() += sleep_factor;
        if self.player.thirst() > pb.thirst_threshold {
            *self.player.thirst_mut() = 0.0;
            self.player.inventory_mut().add_item(wk.drink, -1);
        }
        if self.player.sleeping() {
            *self.player.fatigue_mut() = (self.player.fatigue() - 1).min(0);
        } else {
            *self.player.fatigue_mut() += 1;
        }
        if self.player.fatigue() < pb.fatigue_min {
            *self.player.fatigue_mut() = 0;
            self.player.inventory_mut().add_item(wk.energy, 1);
        }
        if self.player.fatigue() > pb.fatigue_max {
            *self.player.fatigue_mut() = 0;
            self.player.inventory_mut().add_item(wk.energy, -1);
        }
    }

    fn degen_or_regen_health(&mut self) {
        let pb = &self.rules.balance.player;
        let wk = &self.rules.well_known;
        let necessities = [
            self.player.item(wk.food) > 0,
            self.player.item(wk.drink) > 0,
            self.player.item(wk.energy) > 0 || self.player.sleeping(),
        ];
        if necessities.into_iter().all(|ok| ok) {
            *self.player.recover_mut() += if self.player.sleeping() {
                pb.sleep_recovery_factor
            } else {
                1.0
            };
        } else {
            *self.player.recover_mut() -= if self.player.sleeping() {
                pb.sleep_degeneration_factor
            } else {
                1.0
            };
        }
        if self.player.recover() > pb.recover_gain {
            *self.player.recover_mut() = 0.0;
            self.player.set_health(self.player.health() + 1);
        }
        if self.player.recover() < pb.recover_loss {
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

    fn update_entity(&mut self, idx: usize) {
        let Some(entity) = self.world.entity(idx) else {
            return;
        };
        let type_id = entity.type_id;
        let def = self.rules.entity_def(type_id).clone();
        match &def.behavior {
            EntityBehavior::Passive { move_prob } => {
                self.update_passive(idx, *move_prob);
            }
            EntityBehavior::Melee { chase_dist, chase_prob, direct_chase, damage, sleeping_damage, cooldown } => {
                self.update_melee(idx, *chase_dist, *chase_prob, *direct_chase, *damage, *sleeping_damage, *cooldown);
            }
            EntityBehavior::Ranged { flee_dist, shoot_dist, chase_dist, reload, flee_prob, shoot_prob, chase_prob, wander_prob, projectile } => {
                self.update_ranged(idx, *flee_dist, *shoot_dist, *chase_dist, *reload, *flee_prob, *shoot_prob, *chase_prob, *wander_prob, *projectile);
            }
            EntityBehavior::Projectile { damage } => {
                self.update_projectile(idx, *damage);
            }
            EntityBehavior::Growing { ripen_time: _ } => {
                self.update_growing(idx);
            }
            EntityBehavior::Static => {
                // Static entities do nothing, but we need to keep them alive
                // (take + put to maintain consistency)
            }
        }
    }

    fn update_passive(&mut self, idx: usize, move_prob: f32) {
        let Some(mut entity) = self.world.take_entity(idx) else {
            return;
        };
        if entity.health <= 0 {
            return;
        }
        if self.world.random_f32() < move_prob {
            let dir = random_direction(&mut self.world);
            self.try_move_entity(&mut entity.pos, dir, idx, &self.rules.walkable);
        }
        self.world.put_entity(idx, entity);
    }

    fn update_melee(&mut self, idx: usize, chase_dist: i32, chase_prob: f32, direct_chase: f32, damage: i32, sleeping_damage: i32, cooldown: i32) {
        let Some(mut entity) = self.world.take_entity(idx) else {
            return;
        };
        if entity.health <= 0 {
            return;
        }
        let dist = manhattan_distance(entity.pos, self.player.pos());
        if dist <= chase_dist && self.world.random_f32() < chase_prob {
            let dir = toward(
                entity.pos,
                self.player.pos(),
                self.world.random_f32() < direct_chase,
            );
            self.try_move_entity(
                &mut entity.pos,
                dir,
                idx,
                &self.rules.walkable,
            );
        } else {
            let dir = random_direction(&mut self.world);
            self.try_move_entity(
                &mut entity.pos,
                dir,
                idx,
                &self.rules.walkable,
            );
        }
        let dist = manhattan_distance(entity.pos, self.player.pos());
        if dist <= 1 {
            if entity.timer > 0 {
                entity.timer -= 1;
            } else {
                let dmg = if self.player.sleeping() {
                    sleeping_damage
                } else {
                    damage
                };
                self.player.set_health(self.player.health() - dmg);
                entity.timer = cooldown;
            }
        }
        self.world.put_entity(idx, entity);
    }

    fn update_ranged(&mut self, idx: usize, flee_dist: i32, shoot_dist: i32, chase_dist: i32, reload_time: i32, flee_prob: f32, shoot_prob: f32, chase_prob: f32, wander_prob: f32, projectile: ETypeId) {
        let Some(mut entity) = self.world.take_entity(idx) else {
            return;
        };
        if entity.health <= 0 {
            return;
        }
        entity.timer = (entity.timer - 1).max(0);
        let dist = manhattan_distance(entity.pos, self.player.pos());
        if dist <= flee_dist {
            let dir = toward(
                entity.pos,
                self.player.pos(),
                self.world.random_f32() < flee_prob,
            )
            .opposite();
            if self.try_move_entity(
                &mut entity.pos,
                dir,
                idx,
                &self.rules.walkable,
            ) {
                self.world.put_entity(idx, entity);
                return;
            }
        }
        if dist <= shoot_dist && self.world.random_f32() < shoot_prob {
            let dir = toward(entity.pos, self.player.pos(), true);
            self.shoot_projectile(entity.pos, dir, &mut entity.timer, reload_time, projectile);
        } else if dist <= chase_dist && self.world.random_f32() < chase_prob {
            let dir = toward(
                entity.pos,
                self.player.pos(),
                self.world.random_f32() < flee_prob,
            );
            self.try_move_entity(
                &mut entity.pos,
                dir,
                idx,
                &self.rules.walkable,
            );
        } else if self.world.random_f32() < wander_prob {
            let dir = random_direction(&mut self.world);
            self.try_move_entity(
                &mut entity.pos,
                dir,
                idx,
                &self.rules.walkable,
            );
        }
        self.world.put_entity(idx, entity);
    }

    fn shoot_projectile(&mut self, pos: Position, direction: Direction, reload: &mut i32, reload_time: i32, projectile_type: ETypeId) {
        if *reload > 0 {
            return;
        }
        let Some(target) = self.world.offset_pos(pos, direction.delta()) else {
            return;
        };
        if self
            .world
            .is_free_for(target, &self.rules.arrow_walkable, self.player.pos(), None)
        {
            self.world.spawn_entity_facing(target, projectile_type, direction);
            *reload = reload_time;
        }
    }

    fn update_projectile(&mut self, idx: usize, damage: i32) {
        let Some(mut entity) = self.world.take_entity(idx) else {
            return;
        };
        let Some(target) = self.world.offset_pos(entity.pos, entity.facing.delta()) else {
            return;
        };
        if target == self.player.pos() {
            self.player
                .set_health(self.player.health() - damage);
            return;
        }
        if let Some(hit_idx) = self.world.entity_at(target, None) {
            self.world.damage_entity(hit_idx, damage, &self.rules);
            return;
        }
        let Some(material) = self.world.material(target) else {
            return;
        };
        if !self.rules.arrow_walkable.contains(&material) {
            if material == Material::Table.id() || material == Material::Furnace.id() {
                self.world.set_material(target, Material::Path.id());
            }
            return;
        }
        entity.pos = target;
        self.world.put_entity(idx, entity);
    }

    fn update_growing(&mut self, idx: usize) {
        let Some(mut entity) = self.world.take_entity(idx) else {
            return;
        };
        entity.timer += 1;
        // Check for adjacent hostile entities (cow, zombie, skeleton = passive, melee, ranged behaviors)
        let adjacent_types = self.world.adjacent_entity_types(entity.pos);
        if adjacent_types.iter().any(|type_id| {
            let adj_def = self.rules.entity_def(*type_id);
            matches!(adj_def.behavior, EntityBehavior::Passive { .. } | EntityBehavior::Melee { .. } | EntityBehavior::Ranged { .. })
        }) {
            entity.health -= 1;
        }
        if entity.health > 0 {
            self.world.put_entity(idx, entity);
        }
    }

    fn try_move_entity(
        &self,
        pos: &mut Position,
        direction: Direction,
        handle: usize,
        walkable: &[MaterialId],
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

    fn balance_chunks(&mut self) {
        let mut seen = HashSet::new();
        let mut chunks = Vec::new();

        let player_chunk = chunk_key(&self.world, self.player.pos());
        seen.insert(player_chunk);
        chunks.push(player_chunk);

        for idx in self.world.entity_handles() {
            if let Some(entity) = self.world.entity(idx) {
                let chunk = chunk_key(&self.world, entity.pos);
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

        // Iterate over entity types that have spawning config
        for def_idx in 0..self.rules.entity_defs.len() {
            let def = &self.rules.entity_defs[def_idx];
            if let Some(ref spawn_cfg) = def.spawning {
                let type_id = def.type_id;
                let health = def.health;
                let spawn_cfg = spawn_cfg.clone();
                self.balance_spawned(chunk, type_id, health, &spawn_cfg, light);
            }
        }
    }

    fn balance_spawned(
        &mut self,
        chunk: ChunkKey,
        type_id: ETypeId,
        health: i32,
        cfg: &SpawnConfig,
        light: f32,
    ) {
        let positions = chunk_positions_with_material(&self.world, chunk, cfg.material);
        let creatures = self.chunk_creatures(chunk, type_id);
        let (target_min, target_max) = spawn_targets(cfg, light, positions.len());

        if creatures.len() < target_min as usize && self.world.random_f32() < cfg.spawn_prob {
            if positions.is_empty() {
                return;
            }
            let pos = positions[self.world.random_usize(positions.len())];
            let empty = self.world.entity_at(pos, None).is_none() && pos != self.player.pos();
            let away = manhattan_distance(self.player.pos(), pos) >= cfg.spawn_dist;
            if empty && away {
                self.world.spawn_entity(pos, type_id, health);
            }
        } else if creatures.len() > target_max as usize
            && self.world.random_f32() < cfg.despawn_prob
        {
            let handle = creatures[self.world.random_usize(creatures.len())];
            if let Some(entity) = self.world.entity(handle) {
                if manhattan_distance(self.player.pos(), entity.pos) >= cfg.despawn_dist {
                    self.world.remove_entity(handle);
                }
            }
        }
    }

    fn chunk_creatures(&self, chunk: ChunkKey, type_id: ETypeId) -> Vec<usize> {
        self.world
            .entity_handles()
            .into_iter()
            .filter(|&idx| {
                self.world
                    .entity(idx)
                    .map(|e| e.type_id == type_id && chunk_contains(chunk, e.pos))
                    .unwrap_or(false)
            })
            .collect()
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

fn chunk_positions_with_material(world: &World, chunk: ChunkKey, material: MaterialId) -> Vec<Position> {
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

fn sword_damage(player: &Player, combat: &game_rules::CombatBalance) -> i32 {
    let mut max_damage = combat.base_damage;
    for &(item, damage) in &combat.swords {
        if player.item(item) > 0 && damage > max_damage {
            max_damage = damage;
        }
    }
    max_damage
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
