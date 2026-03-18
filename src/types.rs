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
