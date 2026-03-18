use crafter_rs::{Env, Material};

#[test]
fn render_produces_non_empty_rgb_frame_and_inventory_changes() {
    let mut env = Env::default();
    let obs = env.reset();
    assert_eq!(obs.width(), 64);
    assert_eq!(obs.height(), 64);
    assert_eq!(obs.channels(), 3);
    assert!(obs.pixels().iter().any(|pixel| *pixel != 0));

    let wood_id = env.rules().registry.item_id("wood").unwrap();
    env.player_mut().set_item(wood_id, 3);
    let rendered = env.render(None);
    assert_ne!(obs.pixels(), rendered.pixels());
}

#[test]
fn semantic_view_distinguishes_player_and_hostiles() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();

    let player = env.player_position();
    let skel_id = env.rules().entity_type_id("skeleton").unwrap();
    let shealth = env.rules().entity_def(skel_id).health;
    env.world_mut().spawn_entity([player[0] + 1, player[1]], skel_id, shealth);
    let semantic = env.semantic_view();
    let unique: std::collections::HashSet<_> = semantic.cells().iter().copied().collect();
    assert!(unique.len() >= 3);
}

#[test]
fn render_uses_textured_player_sprite_instead_of_flat_block() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();

    let frame = env.render(Some([144, 144]));
    let unit = 16;
    let tile_x = 4 * unit;
    let tile_y = 3 * unit;
    let mut unique = std::collections::HashSet::new();
    for y in tile_y..(tile_y + unit) {
        for x in tile_x..(tile_x + unit) {
            let index = (y * frame.width() + x) * frame.channels();
            unique.insert([
                frame.pixels()[index],
                frame.pixels()[index + 1],
                frame.pixels()[index + 2],
            ]);
        }
    }
    assert!(unique.len() > 8);
}

#[test]
fn night_render_darkens_edges_more_than_inner_tiles() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();

    env.world_mut().set_daylight(1.0);
    let day = env.render(Some([144, 144]));
    env.world_mut().set_daylight(0.0);
    let night = env.render(Some([144, 144]));

    let edge_day = average_tile_luma(&day, 16, 0, 0);
    let inner_day = average_tile_luma(&day, 16, 3, 2);
    let edge_night = average_tile_luma(&night, 16, 0, 0);
    let inner_night = average_tile_luma(&night, 16, 3, 2);

    assert!((edge_day - inner_day).abs() < 3.0);
    assert!(inner_night - edge_night > 5.0);
}

#[test]
fn night_render_adds_noise_so_identical_tiles_are_not_pixel_identical() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();
    env.world_mut().set_daylight(0.0);

    let frame = env.render(Some([144, 144]));
    let tile_a = tile_pixels(&frame, 16, 0, 0);
    let tile_b = tile_pixels(&frame, 16, 1, 0);
    assert_ne!(tile_a, tile_b);
}

#[test]
fn night_render_resamples_noise_between_frames() {
    let mut env = Env::default();
    env.reset();
    env.world_mut().fill(Material::Grass.id());
    env.world_mut().clear_objects();
    env.world_mut().set_daylight(0.0);

    let first = env.render(Some([144, 144]));
    let second = env.render(Some([144, 144]));
    assert_ne!(
        tile_pixels(&first, 16, 0, 0),
        tile_pixels(&second, 16, 0, 0)
    );
}

fn average_tile_luma(frame: &crafter_rs::Frame, unit: usize, tile_x: usize, tile_y: usize) -> f32 {
    let pixels = tile_pixels(frame, unit, tile_x, tile_y);
    let mut total = 0.0;
    let mut count = 0.0;
    for pixel in pixels.chunks_exact(3) {
        total += 0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;
        count += 1.0;
    }
    total / count
}

fn tile_pixels(frame: &crafter_rs::Frame, unit: usize, tile_x: usize, tile_y: usize) -> Vec<u8> {
    let start_x = tile_x * unit;
    let start_y = tile_y * unit;
    let mut out = Vec::with_capacity(unit * unit * frame.channels());
    for y in start_y..(start_y + unit) {
        for x in start_x..(start_x + unit) {
            let index = (y * frame.width() + x) * frame.channels();
            out.extend_from_slice(&frame.pixels()[index..index + frame.channels()]);
        }
    }
    out
}
