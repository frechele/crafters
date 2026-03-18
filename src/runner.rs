use crate::Frame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RunnerKey {
    Left,
    Right,
    Up,
    Down,
    Do,
    Sleep,
    PlaceStone,
    PlaceTable,
    PlaceFurnace,
    PlacePlant,
    MakeWoodPickaxe,
    MakeStonePickaxe,
    MakeIronPickaxe,
    MakeWoodSword,
    MakeStoneSword,
    MakeIronSword,
}

pub fn runner_action_from_keys(keys: &[RunnerKey]) -> &'static str {
    if keys.contains(&RunnerKey::Do) {
        "do"
    } else if keys.contains(&RunnerKey::Sleep) {
        "sleep"
    } else if keys.contains(&RunnerKey::PlaceStone) {
        "place_stone"
    } else if keys.contains(&RunnerKey::PlaceTable) {
        "place_table"
    } else if keys.contains(&RunnerKey::PlaceFurnace) {
        "place_furnace"
    } else if keys.contains(&RunnerKey::PlacePlant) {
        "place_plant"
    } else if keys.contains(&RunnerKey::MakeWoodPickaxe) {
        "make_wood_pickaxe"
    } else if keys.contains(&RunnerKey::MakeStonePickaxe) {
        "make_stone_pickaxe"
    } else if keys.contains(&RunnerKey::MakeIronPickaxe) {
        "make_iron_pickaxe"
    } else if keys.contains(&RunnerKey::MakeWoodSword) {
        "make_wood_sword"
    } else if keys.contains(&RunnerKey::MakeStoneSword) {
        "make_stone_sword"
    } else if keys.contains(&RunnerKey::MakeIronSword) {
        "make_iron_sword"
    } else if keys.contains(&RunnerKey::Left) {
        "move_left"
    } else if keys.contains(&RunnerKey::Right) {
        "move_right"
    } else if keys.contains(&RunnerKey::Up) {
        "move_up"
    } else if keys.contains(&RunnerKey::Down) {
        "move_down"
    } else {
        "noop"
    }
}

pub fn runner_frame_to_buffer(frame: &Frame) -> Vec<u32> {
    frame
        .pixels()
        .chunks_exact(frame.channels())
        .map(|pixel| ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | pixel[2] as u32)
        .collect()
}
