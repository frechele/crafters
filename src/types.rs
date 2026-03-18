pub use crate::registry::{ItemId, MaterialId};

pub type Position = [usize; 2];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ];

    pub fn delta(self) -> [isize; 2] {
        match self {
            Direction::Left => [-1, 0],
            Direction::Right => [1, 0],
            Direction::Up => [0, -1],
            Direction::Down => [0, 1],
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Action {
    Noop = 0,
    MoveLeft = 1,
    MoveRight = 2,
    MoveUp = 3,
    MoveDown = 4,
    Do = 5,
    Sleep = 6,
    PlaceStone = 7,
    PlaceTable = 8,
    PlaceFurnace = 9,
    PlacePlant = 10,
    MakeWoodPickaxe = 11,
    MakeStonePickaxe = 12,
    MakeIronPickaxe = 13,
    MakeWoodSword = 14,
    MakeStoneSword = 15,
    MakeIronSword = 16,
}

pub const ACTIONS: [Action; 17] = [
    Action::Noop,
    Action::MoveLeft,
    Action::MoveRight,
    Action::MoveUp,
    Action::MoveDown,
    Action::Do,
    Action::Sleep,
    Action::PlaceStone,
    Action::PlaceTable,
    Action::PlaceFurnace,
    Action::PlacePlant,
    Action::MakeWoodPickaxe,
    Action::MakeStonePickaxe,
    Action::MakeIronPickaxe,
    Action::MakeWoodSword,
    Action::MakeStoneSword,
    Action::MakeIronSword,
];

pub const ACTION_NAMES: [&str; 17] = [
    "noop",
    "move_left",
    "move_right",
    "move_up",
    "move_down",
    "do",
    "sleep",
    "place_stone",
    "place_table",
    "place_furnace",
    "place_plant",
    "make_wood_pickaxe",
    "make_stone_pickaxe",
    "make_iron_pickaxe",
    "make_wood_sword",
    "make_stone_sword",
    "make_iron_sword",
];

impl Action {
    pub fn name(self) -> &'static str {
        ACTION_NAMES[self as usize]
    }

    pub fn from_index(index: usize) -> Option<Self> {
        ACTIONS.get(index).copied()
    }
}

/// Well-known material constants matching the default config order.
/// These provide named constants for built-in materials while allowing
/// the Registry to define additional materials dynamically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum Material {
    Water = 1,
    Grass = 2,
    Stone = 3,
    Path = 4,
    Sand = 5,
    Tree = 6,
    Lava = 7,
    Coal = 8,
    Iron = 9,
    Diamond = 10,
    Table = 11,
    Furnace = 12,
}

impl Material {
    pub fn from_name(name: &str) -> Option<Material> {
        match name {
            "water" => Some(Material::Water),
            "grass" => Some(Material::Grass),
            "stone" => Some(Material::Stone),
            "path" => Some(Material::Path),
            "sand" => Some(Material::Sand),
            "tree" => Some(Material::Tree),
            "lava" => Some(Material::Lava),
            "coal" => Some(Material::Coal),
            "iron" => Some(Material::Iron),
            "diamond" => Some(Material::Diamond),
            "table" => Some(Material::Table),
            "furnace" => Some(Material::Furnace),
            _ => None,
        }
    }

    pub const ALL: [Material; 12] = [
        Material::Water,
        Material::Grass,
        Material::Stone,
        Material::Path,
        Material::Sand,
        Material::Tree,
        Material::Lava,
        Material::Coal,
        Material::Iron,
        Material::Diamond,
        Material::Table,
        Material::Furnace,
    ];

    pub fn id(self) -> MaterialId {
        MaterialId(self as u16)
    }

    pub fn is_walkable(self) -> bool {
        matches!(self, Material::Grass | Material::Path | Material::Sand)
    }

    pub fn from_id(id: MaterialId) -> Option<Material> {
        match id.0 {
            1 => Some(Material::Water),
            2 => Some(Material::Grass),
            3 => Some(Material::Stone),
            4 => Some(Material::Path),
            5 => Some(Material::Sand),
            6 => Some(Material::Tree),
            7 => Some(Material::Lava),
            8 => Some(Material::Coal),
            9 => Some(Material::Iron),
            10 => Some(Material::Diamond),
            11 => Some(Material::Table),
            12 => Some(Material::Furnace),
            _ => None,
        }
    }
}

/// Well-known item constants matching the default config order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum ItemKind {
    Health = 0,
    Food = 1,
    Drink = 2,
    Energy = 3,
    Sapling = 4,
    Wood = 5,
    Stone = 6,
    Coal = 7,
    Iron = 8,
    Diamond = 9,
    WoodPickaxe = 10,
    StonePickaxe = 11,
    IronPickaxe = 12,
    WoodSword = 13,
    StoneSword = 14,
    IronSword = 15,
}

pub const ITEM_COUNT: usize = 16;

pub const ITEM_ORDER: [ItemKind; ITEM_COUNT] = [
    ItemKind::Health,
    ItemKind::Food,
    ItemKind::Drink,
    ItemKind::Energy,
    ItemKind::Sapling,
    ItemKind::Wood,
    ItemKind::Stone,
    ItemKind::Coal,
    ItemKind::Iron,
    ItemKind::Diamond,
    ItemKind::WoodPickaxe,
    ItemKind::StonePickaxe,
    ItemKind::IronPickaxe,
    ItemKind::WoodSword,
    ItemKind::StoneSword,
    ItemKind::IronSword,
];

impl ItemKind {
    pub fn from_name(name: &str) -> Option<ItemKind> {
        match name {
            "health" => Some(ItemKind::Health),
            "food" => Some(ItemKind::Food),
            "drink" => Some(ItemKind::Drink),
            "energy" => Some(ItemKind::Energy),
            "sapling" => Some(ItemKind::Sapling),
            "wood" => Some(ItemKind::Wood),
            "stone" => Some(ItemKind::Stone),
            "coal" => Some(ItemKind::Coal),
            "iron" => Some(ItemKind::Iron),
            "diamond" => Some(ItemKind::Diamond),
            "wood_pickaxe" => Some(ItemKind::WoodPickaxe),
            "stone_pickaxe" => Some(ItemKind::StonePickaxe),
            "iron_pickaxe" => Some(ItemKind::IronPickaxe),
            "wood_sword" => Some(ItemKind::WoodSword),
            "stone_sword" => Some(ItemKind::StoneSword),
            "iron_sword" => Some(ItemKind::IronSword),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ItemKind::Health => "health",
            ItemKind::Food => "food",
            ItemKind::Drink => "drink",
            ItemKind::Energy => "energy",
            ItemKind::Sapling => "sapling",
            ItemKind::Wood => "wood",
            ItemKind::Stone => "stone",
            ItemKind::Coal => "coal",
            ItemKind::Iron => "iron",
            ItemKind::Diamond => "diamond",
            ItemKind::WoodPickaxe => "wood_pickaxe",
            ItemKind::StonePickaxe => "stone_pickaxe",
            ItemKind::IronPickaxe => "iron_pickaxe",
            ItemKind::WoodSword => "wood_sword",
            ItemKind::StoneSword => "stone_sword",
            ItemKind::IronSword => "iron_sword",
        }
    }

    pub fn id(self) -> ItemId {
        ItemId(self as u16)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Achievement {
    CollectCoal = 0,
    CollectDiamond = 1,
    CollectDrink = 2,
    CollectIron = 3,
    CollectSapling = 4,
    CollectStone = 5,
    CollectWood = 6,
    DefeatSkeleton = 7,
    DefeatZombie = 8,
    EatCow = 9,
    EatPlant = 10,
    MakeIronPickaxe = 11,
    MakeIronSword = 12,
    MakeStonePickaxe = 13,
    MakeStoneSword = 14,
    MakeWoodPickaxe = 15,
    MakeWoodSword = 16,
    PlaceFurnace = 17,
    PlacePlant = 18,
    PlaceStone = 19,
    PlaceTable = 20,
    WakeUp = 21,
}

pub const ACHIEVEMENT_COUNT: usize = 22;

#[cfg(test)]
pub const ACHIEVEMENTS: [Achievement; ACHIEVEMENT_COUNT] = [
    Achievement::CollectCoal,
    Achievement::CollectDiamond,
    Achievement::CollectDrink,
    Achievement::CollectIron,
    Achievement::CollectSapling,
    Achievement::CollectStone,
    Achievement::CollectWood,
    Achievement::DefeatSkeleton,
    Achievement::DefeatZombie,
    Achievement::EatCow,
    Achievement::EatPlant,
    Achievement::MakeIronPickaxe,
    Achievement::MakeIronSword,
    Achievement::MakeStonePickaxe,
    Achievement::MakeStoneSword,
    Achievement::MakeWoodPickaxe,
    Achievement::MakeWoodSword,
    Achievement::PlaceFurnace,
    Achievement::PlacePlant,
    Achievement::PlaceStone,
    Achievement::PlaceTable,
    Achievement::WakeUp,
];

impl Achievement {
    pub fn from_name(name: &str) -> Option<Achievement> {
        match name {
            "collect_coal" => Some(Achievement::CollectCoal),
            "collect_diamond" => Some(Achievement::CollectDiamond),
            "collect_drink" => Some(Achievement::CollectDrink),
            "collect_iron" => Some(Achievement::CollectIron),
            "collect_sapling" => Some(Achievement::CollectSapling),
            "collect_stone" => Some(Achievement::CollectStone),
            "collect_wood" => Some(Achievement::CollectWood),
            "defeat_skeleton" => Some(Achievement::DefeatSkeleton),
            "defeat_zombie" => Some(Achievement::DefeatZombie),
            "eat_cow" => Some(Achievement::EatCow),
            "eat_plant" => Some(Achievement::EatPlant),
            "make_iron_pickaxe" => Some(Achievement::MakeIronPickaxe),
            "make_iron_sword" => Some(Achievement::MakeIronSword),
            "make_stone_pickaxe" => Some(Achievement::MakeStonePickaxe),
            "make_stone_sword" => Some(Achievement::MakeStoneSword),
            "make_wood_pickaxe" => Some(Achievement::MakeWoodPickaxe),
            "make_wood_sword" => Some(Achievement::MakeWoodSword),
            "place_furnace" => Some(Achievement::PlaceFurnace),
            "place_plant" => Some(Achievement::PlacePlant),
            "place_stone" => Some(Achievement::PlaceStone),
            "place_table" => Some(Achievement::PlaceTable),
            "wake_up" => Some(Achievement::WakeUp),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Achievement::CollectCoal => "collect_coal",
            Achievement::CollectDiamond => "collect_diamond",
            Achievement::CollectDrink => "collect_drink",
            Achievement::CollectIron => "collect_iron",
            Achievement::CollectSapling => "collect_sapling",
            Achievement::CollectStone => "collect_stone",
            Achievement::CollectWood => "collect_wood",
            Achievement::DefeatSkeleton => "defeat_skeleton",
            Achievement::DefeatZombie => "defeat_zombie",
            Achievement::EatCow => "eat_cow",
            Achievement::EatPlant => "eat_plant",
            Achievement::MakeIronPickaxe => "make_iron_pickaxe",
            Achievement::MakeIronSword => "make_iron_sword",
            Achievement::MakeStonePickaxe => "make_stone_pickaxe",
            Achievement::MakeStoneSword => "make_stone_sword",
            Achievement::MakeWoodPickaxe => "make_wood_pickaxe",
            Achievement::MakeWoodSword => "make_wood_sword",
            Achievement::PlaceFurnace => "place_furnace",
            Achievement::PlacePlant => "place_plant",
            Achievement::PlaceStone => "place_stone",
            Achievement::PlaceTable => "place_table",
            Achievement::WakeUp => "wake_up",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    items: Vec<i32>,
}

impl Inventory {
    pub fn new(size: usize) -> Self {
        Self {
            items: vec![0; size],
        }
    }

    pub fn from_initial(initial: &[i32]) -> Self {
        Self {
            items: initial.to_vec(),
        }
    }

    pub fn item(&self, id: ItemId) -> i32 {
        self.items[id.0 as usize]
    }

    pub fn set_item(&mut self, id: ItemId, value: i32) {
        self.items[id.0 as usize] = value;
    }

    pub fn add_item(&mut self, id: ItemId, delta: i32) {
        self.items[id.0 as usize] += delta;
    }

    pub fn clamp(&mut self) {
        for item in self.items.iter_mut() {
            *item = (*item).clamp(0, 9);
        }
    }

    pub fn clamp_with(&mut self, max: &[i32]) {
        for (item, max_val) in self.items.iter_mut().zip(max.iter()) {
            *item = (*item).clamp(0, *max_val);
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn raw(&self) -> &[i32] {
        &self.items
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new(ITEM_COUNT)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AchievementProgress {
    counts: Vec<u32>,
}

impl AchievementProgress {
    pub fn new(size: usize) -> Self {
        Self {
            counts: vec![0; size],
        }
    }

    pub fn increment(&mut self, index: usize) {
        self.counts[index] += 1;
    }

    pub fn count(&self, index: usize) -> u32 {
        self.counts[index]
    }

    pub fn len(&self) -> usize {
        self.counts.len()
    }

    /// Check if a named achievement is unlocked (backward compat via Achievement enum).
    pub fn unlocked(&self, name: &str) -> bool {
        Achievement::from_name(name)
            .map_or(false, |a| self.counts.get(a as usize).copied().unwrap_or(0) > 0)
    }
}

impl Default for AchievementProgress {
    fn default() -> Self {
        Self::new(ACHIEVEMENT_COUNT)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    width: usize,
    height: usize,
    channels: usize,
    pixels: Vec<u8>,
}

impl Frame {
    pub fn new(width: usize, height: usize, channels: usize, pixels: Vec<u8>) -> Self {
        Self {
            width,
            height,
            channels,
            pixels,
        }
    }

    pub fn blank(width: usize, height: usize, channels: usize) -> Self {
        Self {
            width,
            height,
            channels,
            pixels: vec![0; width * height * channels],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn channels(&self) -> usize {
        self.channels
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticGrid {
    width: usize,
    height: usize,
    cells: Vec<u16>,
}

impl SemanticGrid {
    pub fn new(width: usize, height: usize, cells: Vec<u16>) -> Self {
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> &[u16] {
        &self.cells
    }
}

#[derive(Clone, Debug)]
pub struct StepInfo {
    pub inventory: Inventory,
    pub achievements: AchievementProgress,
    pub discount: f32,
    pub semantic: SemanticGrid,
    pub player_pos: Position,
    pub reward: f32,
}

#[derive(Clone, Debug)]
pub struct StepResult {
    pub observation: Frame,
    pub reward: f32,
    pub done: bool,
    pub info: StepInfo,
}
