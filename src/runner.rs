use crate::{Action, Frame};

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

pub fn runner_action_from_keys(keys: &[RunnerKey]) -> Action {
    if keys.contains(&RunnerKey::Do) {
        Action::Do
    } else if keys.contains(&RunnerKey::Sleep) {
        Action::Sleep
    } else if keys.contains(&RunnerKey::PlaceStone) {
        Action::PlaceStone
    } else if keys.contains(&RunnerKey::PlaceTable) {
        Action::PlaceTable
    } else if keys.contains(&RunnerKey::PlaceFurnace) {
        Action::PlaceFurnace
    } else if keys.contains(&RunnerKey::PlacePlant) {
        Action::PlacePlant
    } else if keys.contains(&RunnerKey::MakeWoodPickaxe) {
        Action::MakeWoodPickaxe
    } else if keys.contains(&RunnerKey::MakeStonePickaxe) {
        Action::MakeStonePickaxe
    } else if keys.contains(&RunnerKey::MakeIronPickaxe) {
        Action::MakeIronPickaxe
    } else if keys.contains(&RunnerKey::MakeWoodSword) {
        Action::MakeWoodSword
    } else if keys.contains(&RunnerKey::MakeStoneSword) {
        Action::MakeStoneSword
    } else if keys.contains(&RunnerKey::MakeIronSword) {
        Action::MakeIronSword
    } else if keys.contains(&RunnerKey::Left) {
        Action::MoveLeft
    } else if keys.contains(&RunnerKey::Right) {
        Action::MoveRight
    } else if keys.contains(&RunnerKey::Up) {
        Action::MoveUp
    } else if keys.contains(&RunnerKey::Down) {
        Action::MoveDown
    } else {
        Action::Noop
    }
}

pub fn runner_frame_to_buffer(frame: &Frame) -> Vec<u32> {
    frame
        .pixels()
        .chunks_exact(frame.channels())
        .map(|pixel| ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | pixel[2] as u32)
        .collect()
}
