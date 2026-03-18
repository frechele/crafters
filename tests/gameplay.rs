use crafter_rs::{Action, Direction, Env, ItemKind, Material};

#[test]
fn player_can_move_collect_place_and_craft() {
    let mut env = Env::default();
    env.reset();

    env.world_mut().fill(Material::Grass.id());
    let origin = env.player_position();

    let step = env.step(Action::MoveRight);
    assert_eq!(env.player_position(), [origin[0] + 1, origin[1]]);
    assert_eq!(env.player().facing(), Direction::Right);
    assert!(!step.done);

    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Tree.id());
    let wood_before = env.player().item(ItemKind::Wood.id());
    let collect_tree = env.step(Action::Do);
    assert_eq!(env.player().item(ItemKind::Wood.id()), wood_before + 1);
    assert_eq!(env.world().material(front), Some(Material::Grass.id()));
    assert!(collect_tree.info.achievements.unlocked("collect_wood"));

    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Stone.id());
    env.step(Action::Do);
    assert_eq!(env.player().item(ItemKind::Stone.id()), 0);

    env.player_mut().set_item(ItemKind::WoodPickaxe.id(), 1);
    env.player_mut().set_facing(Direction::Right);
    env.step(Action::Do);
    assert_eq!(env.player().item(ItemKind::Stone.id()), 1);
    assert_eq!(env.world().material(front), Some(Material::Path.id()));

    env.player_mut().set_item(ItemKind::Wood.id(), 3);
    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Grass.id());
    env.step(Action::PlaceTable);
    assert_eq!(env.world().material(front), Some(Material::Table.id()));
    assert_eq!(env.player().item(ItemKind::Wood.id()), 1);

    env.player_mut().set_item(ItemKind::Wood.id(), 2);
    env.step(Action::MakeWoodPickaxe);
    assert_eq!(env.player().item(ItemKind::WoodPickaxe.id()), 2);
    assert_eq!(env.player().item(ItemKind::Wood.id()), 1);
}

#[test]
fn survival_needs_can_reduce_and_restore_health() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());

    env.player_mut().set_health(8);
    env.player_mut().set_item(ItemKind::Food.id(), 0);
    env.player_mut().set_item(ItemKind::Drink.id(), 0);
    env.player_mut().set_item(ItemKind::Energy.id(), 0);
    for _ in 0..40 {
        env.step(Action::Noop);
    }
    assert!(env.player().health() < 8);

    env.player_mut().set_health(5);
    env.player_mut().set_item(ItemKind::Food.id(), 9);
    env.player_mut().set_item(ItemKind::Drink.id(), 9);
    env.player_mut().set_item(ItemKind::Energy.id(), 1);
    for _ in 0..40 {
        env.step(Action::Sleep);
    }
    assert!(env.player().health() > 5);
}
