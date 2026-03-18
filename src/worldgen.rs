use crate::game_rules::GameRules;
use crate::opensimplex::OpenSimplexNoise;
use crate::registry::MaterialId;
use crate::{Material, Position, World};

pub fn generate_world(world: &mut World, player_pos: Position, rules: &GameRules) {
    let simplex = OpenSimplexNoise::new(world.random_i64((1_i64 << 31) - 1));
    let mut tunnels = vec![false; world.area()[0] * world.area()[1]];
    for x in 0..world.area()[0] {
        for y in 0..world.area()[1] {
            set_material(world, [x, y], player_pos, &mut tunnels, &simplex);
        }
    }
    for x in 0..world.area()[0] {
        for y in 0..world.area()[1] {
            set_object(world, [x, y], player_pos, &tunnels, rules);
        }
    }
}

fn set_material(
    world: &mut World,
    pos: Position,
    player_pos: Position,
    tunnels: &mut [bool],
    simplex: &OpenSimplexNoise,
) {
    let x = pos[0] as f64;
    let y = pos[1] as f64;
    let px = player_pos[0] as f64;
    let py = player_pos[1] as f64;

    let mut start = 4.0 - ((x - px).powi(2) + (y - py).powi(2)).sqrt();
    start += 2.0 * simplex_sum(simplex, x, y, 8.0, &[(3.0, 1.0)], true);
    start = 1.0 / (1.0 + (-start).exp());

    let mut water = simplex_sum(simplex, x, y, 3.0, &[(15.0, 1.0), (5.0, 0.15)], false) + 0.1;
    water -= 2.0 * start;

    let mut mountain = simplex_sum(simplex, x, y, 0.0, &[(15.0, 1.0), (5.0, 0.3)], true);
    mountain -= 4.0 * start + 0.3 * water;

    let material: MaterialId = if start > 0.5 {
        Material::Grass.id()
    } else if mountain > 0.15 {
        if simplex_sum(simplex, x, y, 6.0, &[(7.0, 1.0)], true) > 0.15 && mountain > 0.3 {
            Material::Path.id()
        } else if simplex_sum(simplex, 2.0 * x, y / 5.0, 7.0, &[(3.0, 1.0)], true) > 0.4
            || simplex_sum(simplex, x / 5.0, 2.0 * y, 7.0, &[(3.0, 1.0)], true) > 0.4
        {
            tunnels[index(world.area(), pos)] = true;
            Material::Path.id()
        } else if simplex_sum(simplex, x, y, 1.0, &[(8.0, 1.0)], true) > 0.0
            && world.random_f64() > 0.85
        {
            Material::Coal.id()
        } else if simplex_sum(simplex, x, y, 2.0, &[(6.0, 1.0)], true) > 0.4
            && world.random_f64() > 0.75
        {
            Material::Iron.id()
        } else if mountain > 0.18 && world.random_f64() > 0.994 {
            Material::Diamond.id()
        } else if mountain > 0.3 && simplex_sum(simplex, x, y, 6.0, &[(5.0, 1.0)], true) > 0.35 {
            Material::Lava.id()
        } else {
            Material::Stone.id()
        }
    } else if water > 0.25
        && water <= 0.35
        && simplex_sum(simplex, x, y, 4.0, &[(9.0, 1.0)], true) > -0.2
    {
        Material::Sand.id()
    } else if water > 0.3 {
        Material::Water.id()
    } else if simplex_sum(simplex, x, y, 5.0, &[(7.0, 1.0)], true) > 0.0
        && world.random_f64() > 0.8
    {
        Material::Tree.id()
    } else {
        Material::Grass.id()
    };
    world.set_material(pos, material);
}

fn simplex_sum(
    simplex: &OpenSimplexNoise,
    x: f64,
    y: f64,
    z: f64,
    sizes: &[(f64, f64)],
    normalize: bool,
) -> f64 {
    let mut value = 0.0;
    let mut weight_sum = 0.0;
    for (size, weight) in sizes {
        value += *weight * simplex.noise3(x / *size, y / *size, z);
        weight_sum += *weight;
    }
    if normalize && weight_sum > 0.0 {
        value / weight_sum
    } else {
        value
    }
}

fn index(area: [usize; 2], pos: Position) -> usize {
    pos[0] * area[1] + pos[1]
}

fn set_object(
    world: &mut World,
    pos: Position,
    player_pos: Position,
    tunnels: &[bool],
    rules: &GameRules,
) {
    let dist = distance(pos, player_pos);
    let Some(material) = world.material(pos) else {
        return;
    };
    let walkable = Material::from_id(material).map_or(false, |m| m.is_walkable());
    if !walkable {
        return;
    }

    // Iterate over entity types that have worldgen config
    for def in &rules.entity_defs {
        let Some(ref wg) = def.worldgen else {
            continue;
        };
        // Check minimum distance from player
        if dist <= wg.min_dist {
            continue;
        }
        // Check material requirement
        if let Some(ref mat) = wg.material {
            if material != *mat {
                continue;
            }
        }
        // Check tunnel-only requirement
        if wg.tunnel_only && !tunnels[index(world.area(), pos)] {
            continue;
        }
        // Check threshold
        if world.random_f64() > wg.threshold {
            world.spawn_entity(pos, def.type_id, def.health);
            return;
        }
    }
}

fn distance(lhs: Position, rhs: Position) -> f64 {
    let dx = lhs[0] as f64 - rhs[0] as f64;
    let dy = lhs[1] as f64 - rhs[1] as f64;
    (dx.powi(2) + dy.powi(2)).sqrt()
}
