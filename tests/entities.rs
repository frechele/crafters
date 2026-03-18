use crafter_rs::{Action, Direction, Env, EnvConfig, Material};

#[test]
fn achievements_affect_reward_once_and_length_can_end_episode() {
    let mut env = Env::new(EnvConfig {
        length: Some(2),
        ..EnvConfig::default()
    });
    env.reset();
    env.world_mut().fill(Material::Grass.id());

    env.player_mut().set_facing(Direction::Right);
    let front = [env.player_position()[0] + 1, env.player_position()[1]];
    env.world_mut().set_material(front, Material::Tree.id());
    let first = env.step(Action::Do);
    assert!(first.reward >= 1.0);
    assert!(first.info.achievements.unlocked("collect_wood"));

    env.player_mut().set_facing(Direction::Right);
    env.world_mut().set_material(front, Material::Tree.id());
    let second = env.step(Action::Do);
    assert!(second.reward < 1.0);
    assert!(second.done);
}

#[test]
fn zombie_and_skeleton_updates_affect_player_and_world() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();

    let player = env.player_position();
    let zhealth = env.rules().entity_def(env.rules().entity_type_id("zombie").unwrap()).health;
    env.world_mut().spawn_zombie([player[0] + 1, player[1]], zhealth);
    let health_before = env.player().health();
    let mut zombie_rewards = Vec::new();
    for _ in 0..10 {
        let zombie_step = env.step(Action::Noop);
        zombie_rewards.push(zombie_step.reward);
        if env.player().health() < health_before {
            break;
        }
    }
    assert!(env.player().health() < health_before);
    assert!(zombie_rewards.into_iter().any(|reward| reward < 0.0));

    env.world_mut().clear_objects();
    let shealth = env.rules().entity_def(env.rules().entity_type_id("skeleton").unwrap()).health;
    env.world_mut().spawn_skeleton([player[0] + 4, player[1]], shealth);
    for _ in 0..20 {
        env.step(Action::Noop);
        if env.world().arrow_count() > 0 {
            break;
        }
    }
    assert!(env.world().arrow_count() > 0);
}

#[test]
fn chunk_balancing_can_spawn_skeletons_in_populated_path_chunks() {
    let mut env = Env::new(EnvConfig {
        area: [64, 64],
        ..EnvConfig::default()
    });
    env.world_mut().fill(Material::Path.id());
    env.world_mut().clear_objects();

    let player = env.player_position();
    let cow_health = env.rules().entity_def(env.rules().entity_type_id("cow").unwrap()).health;
    for chunk_x in (0..64).step_by(12) {
        for chunk_y in (0..64).step_by(12) {
            let pos = [(chunk_x + 1).min(63), (chunk_y + 1).min(63)];
            if pos != player {
                env.world_mut().spawn_cow(pos, cow_health);
            }
        }
    }

    assert_eq!(count_semantic_id(&env, 16), 0);
    for _ in 0..10 {
        env.step(Action::Noop);
    }
    assert!(
        count_semantic_id(&env, 16) > 0,
        "expected periodic chunk balancing to spawn at least one skeleton"
    );
}

#[test]
fn chunk_balancing_can_spawn_zombies_in_populated_grass_chunks() {
    let mut env = Env::new(EnvConfig {
        area: [64, 64],
        ..EnvConfig::default()
    });
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();

    let player = env.player_position();
    let plant_def = env.rules().entity_def(env.rules().entity_type_id("plant").unwrap());
    let plant_health = plant_def.health;
    for chunk_x in (0..64).step_by(12) {
        for chunk_y in (0..64).step_by(12) {
            let pos = [(chunk_x + 1).min(63), (chunk_y + 1).min(63)];
            if pos != player {
                env.world_mut().spawn_plant(pos, plant_health, 300);
            }
        }
    }

    assert_eq!(count_semantic_id(&env, 15), 0);
    for _ in 0..300 {
        env.step(Action::Noop);
    }
    assert!(
        count_semantic_id(&env, 15) > 0,
        "expected nighttime chunk balancing to spawn at least one zombie"
    );
}

#[test]
fn lethal_damage_marks_done_and_zero_discount() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();

    let player = env.player_position();
    env.player_mut().set_health(1);
    let zhealth = env.rules().entity_def(env.rules().entity_type_id("zombie").unwrap()).health;
    env.world_mut().spawn_zombie([player[0] + 1, player[1]], zhealth);
    let mut result = env.step(Action::Noop);
    for _ in 0..10 {
        if result.done {
            break;
        }
        result = env.step(Action::Noop);
    }
    assert!(result.done);
    assert_eq!(result.info.discount, 0.0);
}

fn count_semantic_id(env: &Env, id: u16) -> usize {
    env.semantic_view()
        .cells()
        .iter()
        .filter(|cell| **cell == id)
        .count()
}
