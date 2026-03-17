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

    pub fn id(self) -> u16 {
        self as u16
    }

    pub fn is_walkable(self) -> bool {
        matches!(self, Material::Grass | Material::Path | Material::Sand)
    }

    pub(crate) fn texture_name(self) -> &'static str {
        match self {
            Material::Water => "water",
            Material::Grass => "grass",
            Material::Stone => "stone",
            Material::Path => "path",
            Material::Sand => "sand",
            Material::Tree => "tree",
            Material::Lava => "lava",
            Material::Coal => "coal",
            Material::Iron => "iron",
            Material::Diamond => "diamond",
            Material::Table => "table",
            Material::Furnace => "furnace",
        }
    }
}

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

    pub(crate) fn texture_name(self) -> &'static str {
        self.name()
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
    items: [i32; ITEM_COUNT],
}

impl Inventory {
    pub fn new() -> Self {
        let mut items = [0; ITEM_COUNT];
        items[ItemKind::Health as usize] = 9;
        items[ItemKind::Food as usize] = 9;
        items[ItemKind::Drink as usize] = 9;
        items[ItemKind::Energy as usize] = 9;
        Self { items }
    }

    pub fn item(&self, kind: ItemKind) -> i32 {
        self.items[kind as usize]
    }

    pub fn set_item(&mut self, kind: ItemKind, value: i32) {
        self.items[kind as usize] = value;
    }

    pub fn add_item(&mut self, kind: ItemKind, delta: i32) {
        self.items[kind as usize] += delta;
    }

    pub fn clamp(&mut self) {
        for kind in ITEM_ORDER {
            self.items[kind as usize] = self.items[kind as usize].clamp(0, 9);
        }
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AchievementProgress {
    counts: [u32; ACHIEVEMENT_COUNT],
}

impl AchievementProgress {
    pub fn new() -> Self {
        Self {
            counts: [0; ACHIEVEMENT_COUNT],
        }
    }

    pub fn increment(&mut self, achievement: Achievement) {
        self.counts[achievement as usize] += 1;
    }

    pub fn count(&self, achievement: Achievement) -> u32 {
        self.counts[achievement as usize]
    }

    pub fn unlocked(&self, name: &str) -> bool {
        ACHIEVEMENTS
            .iter()
            .any(|achievement| achievement.name() == name && self.count(*achievement) > 0)
    }
}

impl Default for AchievementProgress {
    fn default() -> Self {
        Self::new()
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
