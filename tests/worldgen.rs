use std::collections::HashSet;

use crafter_rs::{Env, EnvConfig, Material};

#[test]
fn worldgen_is_seeded_and_diverse() {
    let config = EnvConfig {
        seed: 7,
        ..EnvConfig::default()
    };
    let mut env_a = Env::new(config.clone());
    let mut env_b = Env::new(config);

    env_a.reset();
    env_b.reset();

    let area = env_a.world().area();
    let mut materials_a = Vec::with_capacity(area[0] * area[1]);
    let mut materials_b = Vec::with_capacity(area[0] * area[1]);
    for x in 0..area[0] {
        for y in 0..area[1] {
            materials_a.push(env_a.world().material([x, y]));
            materials_b.push(env_b.world().material([x, y]));
        }
    }
    assert_eq!(materials_a, materials_b);

    let unique: HashSet<_> = materials_a.into_iter().flatten().collect();
    assert!(unique.contains(&Material::Grass.id()));
    assert!(unique.contains(&Material::Water.id()));
    assert!(unique.contains(&Material::Stone.id()));
    assert!(unique.len() >= 5);

    let center = env_a.player_position();
    assert_eq!(env_a.world().material(center), Some(Material::Grass.id()));
    for x in (center[0] - 2)..=(center[0] + 2) {
        for y in (center[1] - 2)..=(center[1] + 2) {
            let mid = env_a.world().material([x, y]).unwrap();
            assert!(Material::from_id(mid).map_or(false, |m| m.is_walkable()));
        }
    }
}
