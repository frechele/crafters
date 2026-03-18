use crate::registry::{EntityTypeId, ItemId};
use crate::{AchievementProgress, Direction, Inventory, Position};

#[derive(Clone, Debug)]
pub struct Player {
    pos: Position,
    facing: Direction,
    inventory: Inventory,
    achievements: AchievementProgress,
    sleeping: bool,
    last_health: i32,
    hunger: f32,
    thirst: f32,
    fatigue: i32,
    recover: f32,
    health_id: ItemId,
}

impl Player {
    pub fn with_inventory(pos: Position, inventory: Inventory, achievement_count: usize, health_id: ItemId) -> Self {
        let last_health = inventory.item(health_id);
        Self {
            pos,
            facing: Direction::Down,
            inventory,
            achievements: AchievementProgress::new(achievement_count),
            sleeping: false,
            last_health,
            hunger: 0.0,
            thirst: 0.0,
            fatigue: 0,
            recover: 0.0,
            health_id,
        }
    }

    pub fn pos(&self) -> Position {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Position) {
        self.pos = pos;
    }

    pub fn facing(&self) -> Direction {
        self.facing
    }

    pub fn set_facing(&mut self, facing: Direction) {
        self.facing = facing;
    }

    pub fn sleeping(&self) -> bool {
        self.sleeping
    }

    pub fn set_sleeping(&mut self, sleeping: bool) {
        self.sleeping = sleeping;
    }

    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }

    pub fn achievements(&self) -> &AchievementProgress {
        &self.achievements
    }

    pub fn achievements_mut(&mut self) -> &mut AchievementProgress {
        &mut self.achievements
    }

    pub fn item(&self, id: ItemId) -> i32 {
        self.inventory.item(id)
    }

    pub fn set_item(&mut self, id: ItemId, value: i32) {
        self.inventory.set_item(id, value);
    }

    pub fn health(&self) -> i32 {
        self.item(self.health_id)
    }

    pub fn set_health(&mut self, value: i32) {
        self.set_item(self.health_id, value.max(0));
    }

    pub fn last_health(&self) -> i32 {
        self.last_health
    }

    pub fn set_last_health(&mut self, last_health: i32) {
        self.last_health = last_health;
    }

    pub fn hunger(&self) -> f32 {
        self.hunger
    }

    pub fn hunger_mut(&mut self) -> &mut f32 {
        &mut self.hunger
    }

    pub fn thirst(&self) -> f32 {
        self.thirst
    }

    pub fn thirst_mut(&mut self) -> &mut f32 {
        &mut self.thirst
    }

    pub fn fatigue(&self) -> i32 {
        self.fatigue
    }

    pub fn fatigue_mut(&mut self) -> &mut i32 {
        &mut self.fatigue
    }

    pub fn recover(&self) -> f32 {
        self.recover
    }

    pub fn recover_mut(&mut self) -> &mut f32 {
        &mut self.recover
    }

    pub fn clamp_inventory(&mut self) {
        self.inventory.clamp();
    }

    pub(crate) fn texture_name(&self) -> &'static str {
        if self.sleeping {
            "player-sleep"
        } else {
            match self.facing {
                Direction::Left => "player-left",
                Direction::Right => "player-right",
                Direction::Up => "player-up",
                Direction::Down => "player-down",
            }
        }
    }
}

/// Unified entity struct replacing Cow, Zombie, Skeleton, Arrow, Plant, Fence.
#[derive(Clone, Debug)]
pub struct Entity {
    pub type_id: EntityTypeId,
    pub pos: Position,
    pub health: i32,
    pub facing: Direction,
    pub timer: i32,  // cooldown for melee, reload for ranged, grown for growing
}

impl Entity {
    pub fn new(type_id: EntityTypeId, pos: Position, health: i32) -> Self {
        Self {
            type_id,
            pos,
            health,
            facing: Direction::Down,
            timer: 0,
        }
    }

    pub fn with_facing(type_id: EntityTypeId, pos: Position, facing: Direction) -> Self {
        Self {
            type_id,
            pos,
            health: 0,
            facing,
            timer: 0,
        }
    }
}
