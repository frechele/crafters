use crate::{AchievementProgress, Action, Direction, Inventory, Position};

#[derive(Clone, Debug)]
pub struct Player {
    pos: Position,
    facing: Direction,
    inventory: Inventory,
    achievements: AchievementProgress,
    action: Action,
    sleeping: bool,
    last_health: i32,
    hunger: f32,
    thirst: f32,
    fatigue: i32,
    recover: f32,
}

impl Player {
    pub fn new(pos: Position) -> Self {
        let inventory = Inventory::new();
        let last_health = inventory.item(crate::ItemKind::Health);
        Self {
            pos,
            facing: Direction::Down,
            inventory,
            achievements: AchievementProgress::new(),
            action: Action::Noop,
            sleeping: false,
            last_health,
            hunger: 0.0,
            thirst: 0.0,
            fatigue: 0,
            recover: 0.0,
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

    pub fn action(&self) -> Action {
        self.action
    }

    pub fn set_action(&mut self, action: Action) {
        self.action = action;
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

    pub fn item(&self, kind: crate::ItemKind) -> i32 {
        self.inventory.item(kind)
    }

    pub fn set_item(&mut self, kind: crate::ItemKind, value: i32) {
        self.inventory.set_item(kind, value);
    }

    pub fn health(&self) -> i32 {
        self.item(crate::ItemKind::Health)
    }

    pub fn set_health(&mut self, value: i32) {
        self.set_item(crate::ItemKind::Health, value.max(0));
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

#[derive(Clone, Debug)]
pub struct Cow {
    pub pos: Position,
    pub health: i32,
}

impl Cow {
    pub fn new(pos: Position) -> Self {
        Self { pos, health: 3 }
    }
}

#[derive(Clone, Debug)]
pub struct Zombie {
    pub pos: Position,
    pub health: i32,
    pub cooldown: i32,
}

impl Zombie {
    pub fn new(pos: Position) -> Self {
        Self {
            pos,
            health: 5,
            cooldown: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub pos: Position,
    pub health: i32,
    pub reload: i32,
}

impl Skeleton {
    pub fn new(pos: Position) -> Self {
        Self {
            pos,
            health: 3,
            reload: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Arrow {
    pub pos: Position,
    pub facing: Direction,
}

impl Arrow {
    pub fn new(pos: Position, facing: Direction) -> Self {
        Self { pos, facing }
    }

    pub(crate) fn texture_name(&self) -> &'static str {
        match self.facing {
            Direction::Left => "arrow-left",
            Direction::Right => "arrow-right",
            Direction::Up => "arrow-up",
            Direction::Down => "arrow-down",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Plant {
    pub pos: Position,
    pub health: i32,
    pub grown: i32,
}

impl Plant {
    pub fn new(pos: Position) -> Self {
        Self {
            pos,
            health: 1,
            grown: 0,
        }
    }

    pub fn ripe(&self) -> bool {
        self.grown > 300
    }

    pub(crate) fn texture_name(&self) -> &'static str {
        if self.ripe() { "plant-ripe" } else { "plant" }
    }
}

#[derive(Clone, Debug)]
pub struct Fence {
    pub pos: Position,
}

impl Fence {
    pub fn new(pos: Position) -> Self {
        Self { pos }
    }
}
