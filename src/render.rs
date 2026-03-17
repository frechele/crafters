use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use image::{ImageReader, Rgba, RgbaImage};

use crate::pillow_resize_16::PILLOW_NEAREST_16;
use crate::{Env, Frame};

pub fn render(env: &Env, size: Option<[usize; 2]>, noise_index: u64) -> Frame {
    let size = size.unwrap_or(env.config().size);
    let view = env.config().view;
    let item_rows = crate::ITEM_COUNT.div_ceil(view[0]);
    let local_rows = view[1].saturating_sub(item_rows);
    let unit = [(size[0] / view[0]).max(1), (size[1] / view[1]).max(1)];
    let view_pixels = [view[0] * unit[0], view[1] * unit[1]];
    let border = [
        (size[0].saturating_sub(view_pixels[0])) / 2,
        (size[1].saturating_sub(view_pixels[1])) / 2,
    ];

    let mut local_canvas = vec![127_u8; view[0] * unit[0] * local_rows * unit[1] * 3];
    let local_size = [view[0] * unit[0], local_rows * unit[1]];
    let mut item_canvas = vec![0_u8; view[0] * unit[0] * item_rows * unit[1] * 3];
    let item_size = [view[0] * unit[0], item_rows * unit[1]];

    draw_local_view(env, unit, local_rows, local_size, &mut local_canvas);
    apply_lighting(&mut local_canvas, local_size, env, noise_index);
    if env.player().sleeping() {
        apply_sleep_tint(&mut local_canvas);
    }
    draw_item_view(env, unit, item_size, &mut item_canvas);

    let mut pixels = vec![0_u8; size[0] * size[1] * 3];
    blit_rgb(&mut pixels, size, border, &local_canvas, local_size);
    blit_rgb(
        &mut pixels,
        size,
        [border[0], border[1] + local_size[1]],
        &item_canvas,
        item_size,
    );
    Frame::new(size[0], size[1], 3, pixels)
}

fn draw_local_view(
    env: &Env,
    unit: [usize; 2],
    local_rows: usize,
    canvas_size: [usize; 2],
    canvas: &mut [u8],
) {
    let textures = textures();
    let x_offset = env.config().view[0] / 2;
    let y_offset = local_rows / 2;

    for gx in 0..env.config().view[0] {
        for gy in 0..local_rows {
            let world_x = env.player_position()[0] as isize + gx as isize - x_offset as isize;
            let world_y = env.player_position()[1] as isize + gy as isize - y_offset as isize;
            if world_x < 0
                || world_y < 0
                || world_x >= env.world().area()[0] as isize
                || world_y >= env.world().area()[1] as isize
            {
                continue;
            }
            let Some(material) = env.world().material([world_x as usize, world_y as usize]) else {
                continue;
            };
            let texture = textures.get(material.texture_name(), unit);
            draw_texture_opaque(
                canvas,
                canvas_size,
                [gx * unit[0], gy * unit[1]],
                texture.as_ref(),
            );
        }
    }

    for handle in env.world().object_handles() {
        let Some((pos, _)) = env.world().object_position_and_kind(handle) else {
            continue;
        };
        let rel_x = pos[0] as isize - env.player_position()[0] as isize + x_offset as isize;
        let rel_y = pos[1] as isize - env.player_position()[1] as isize + y_offset as isize;
        if rel_x < 0
            || rel_y < 0
            || rel_x >= env.config().view[0] as isize
            || rel_y >= local_rows as isize
        {
            continue;
        }
        let Some(name) = env.world().object_texture_name(handle) else {
            continue;
        };
        let texture = textures.get(name, unit);
        draw_texture_alpha(
            canvas,
            canvas_size,
            [rel_x as usize * unit[0], rel_y as usize * unit[1]],
            texture.as_ref(),
        );
    }

    let player_texture = textures.get(env.player().texture_name(), unit);
    draw_texture_alpha(
        canvas,
        canvas_size,
        [
            x_offset * unit[0],
            y_offset.min(local_rows.saturating_sub(1)) * unit[1],
        ],
        player_texture.as_ref(),
    );
}

fn draw_item_view(
    env: &Env,
    unit: [usize; 2],
    canvas_size: [usize; 2],
    canvas: &mut [u8],
) {
    let textures = textures();
    let item_size = scaled_unit(unit, 0.8);
    let amount_size = scaled_unit(unit, 0.6);
    for (index, item) in crate::ITEM_ORDER.into_iter().enumerate() {
        let amount = env.player().item(item);
        if amount < 1 {
            continue;
        }
        let grid_pos = [index % env.config().view[0], index / env.config().view[0]];
        let item_pos = [
            grid_pos[0] * unit[0] + unit[0] / 10,
            grid_pos[1] * unit[1] + unit[1] / 10,
        ];
        let amount_pos = [
            grid_pos[0] * unit[0] + (unit[0] * 4) / 10,
            grid_pos[1] * unit[1] + (unit[1] * 4) / 10,
        ];
        let item_texture = textures.get(item.texture_name(), item_size);
        draw_texture_alpha(canvas, canvas_size, item_pos, item_texture.as_ref());
        let amount_name = amount_texture_name(amount);
        let amount_texture = textures.get(amount_name.as_str(), amount_size);
        draw_texture_alpha(canvas, canvas_size, amount_pos, amount_texture.as_ref());
    }
}

fn scaled_unit(unit: [usize; 2], scale: f32) -> [usize; 2] {
    [
        ((unit[0] as f32 * scale) as usize).max(1),
        ((unit[1] as f32 * scale) as usize).max(1),
    ]
}

fn amount_texture_name(amount: i32) -> String {
    if (1..=9).contains(&amount) {
        amount.to_string()
    } else {
        "unknown".to_string()
    }
}

fn blit_rgb(
    dest: &mut [u8],
    dest_size: [usize; 2],
    pos: [usize; 2],
    src: &[u8],
    src_size: [usize; 2],
) {
    for y in 0..src_size[1] {
        for x in 0..src_size[0] {
            let dest_x = pos[0] + x;
            let dest_y = pos[1] + y;
            if dest_x >= dest_size[0] || dest_y >= dest_size[1] {
                continue;
            }
            let src_index = (y * src_size[0] + x) * 3;
            let dest_index = (dest_y * dest_size[0] + dest_x) * 3;
            dest[dest_index..dest_index + 3].copy_from_slice(&src[src_index..src_index + 3]);
        }
    }
}

fn draw_texture_opaque(
    canvas: &mut [u8],
    canvas_size: [usize; 2],
    pos: [usize; 2],
    texture: &RgbaImage,
) {
    for y in 0..texture.height() as usize {
        for x in 0..texture.width() as usize {
            let canvas_x = pos[0] + x;
            let canvas_y = pos[1] + y;
            if canvas_x >= canvas_size[0] || canvas_y >= canvas_size[1] {
                continue;
            }
            let pixel = texture.get_pixel(x as u32, y as u32).0;
            let index = (canvas_y * canvas_size[0] + canvas_x) * 3;
            canvas[index] = pixel[0];
            canvas[index + 1] = pixel[1];
            canvas[index + 2] = pixel[2];
        }
    }
}

fn draw_texture_alpha(
    canvas: &mut [u8],
    canvas_size: [usize; 2],
    pos: [usize; 2],
    texture: &RgbaImage,
) {
    for y in 0..texture.height() as usize {
        for x in 0..texture.width() as usize {
            let canvas_x = pos[0] + x;
            let canvas_y = pos[1] + y;
            if canvas_x >= canvas_size[0] || canvas_y >= canvas_size[1] {
                continue;
            }
            let pixel = texture.get_pixel(x as u32, y as u32).0;
            let alpha = pixel[3] as f32 / 255.0;
            let index = (canvas_y * canvas_size[0] + canvas_x) * 3;
            for channel in 0..3 {
                let current = canvas[index + channel] as f32 / 255.0;
                let next = pixel[channel] as f32 / 255.0;
                let blended = alpha * next + (1.0 - alpha) * current;
                canvas[index + channel] = (255.0 * blended).clamp(0.0, 255.0) as u8;
            }
        }
    }
}

fn apply_lighting(pixels: &mut [u8], size: [usize; 2], env: &Env, noise_index: u64) {
    let daylight = env.world().daylight().clamp(0.0, 1.0) as f64;
    let mut night: Vec<f64> = pixels.iter().map(|&channel| channel as f64).collect();
    if daylight < 0.5 {
        let amount = 2.0 * (0.5 - daylight);
        apply_night_noise(&mut night, size, amount, 0.5, noise_seed(env, noise_index));
    }
    pillow_color_enhance(&mut night, 0.4);
    tint_pixels(&mut night, [0.0, 16.0, 64.0], 0.5);

    for (index, channel) in pixels.iter_mut().enumerate() {
        let base = *channel as f64;
        let lit = daylight * base + (1.0 - daylight) * night[index];
        *channel = lit.clamp(0.0, 255.0) as u8;
    }
}

fn apply_sleep_tint(pixels: &mut [u8]) {
    for rgb in pixels.chunks_exact_mut(3) {
        let gray = pil_grayscale(rgb[0], rgb[1], rgb[2]) as u16;
        rgb[0] = (gray / 2) as u8;
        rgb[1] = (gray / 2) as u8;
        rgb[2] = ((gray + 16) / 2) as u8;
    }
}

fn apply_night_noise(pixels: &mut [f64], size: [usize; 2], amount: f64, stddev: f64, seed: u64) {
    let mask = vignette(size, stddev);
    for y in 0..size[1] {
        for x in 0..size[0] {
            let noise = noise_value(seed, x as u64, y as u64);
            let factor = (amount * mask[y * size[0] + x]).clamp(0.0, 1.0);
            let index = (y * size[0] + x) * 3;
            for channel in 0..3 {
                pixels[index + channel] = (1.0 - factor) * pixels[index + channel] + factor * noise;
            }
        }
    }
}

fn pillow_color_enhance(pixels: &mut [f64], factor: f64) {
    for rgb in pixels.chunks_exact_mut(3) {
        let original = [rgb[0] as u8, rgb[1] as u8, rgb[2] as u8];
        let gray = pil_grayscale(original[0], original[1], original[2]) as f64;
        for channel in 0..3 {
            let blended = (1.0 - factor) * gray + factor * original[channel] as f64;
            rgb[channel] = (blended.clamp(0.0, 255.0) as u8) as f64;
        }
    }
}

fn pil_grayscale(r: u8, g: u8, b: u8) -> u8 {
    let value = 19595_u32 * r as u32 + 38470_u32 * g as u32 + 7471_u32 * b as u32;
    ((value + (1 << 15)) >> 16) as u8
}

fn tint_pixels(pixels: &mut [f64], tint: [f64; 3], amount: f64) {
    for rgb in pixels.chunks_exact_mut(3) {
        for channel in 0..3 {
            rgb[channel] = (1.0 - amount) * rgb[channel] + amount * tint[channel];
        }
    }
}

fn noise_seed(env: &Env, noise_index: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    env.episode().hash(&mut hasher);
    env.step_count().hash(&mut hasher);
    env.player_position().hash(&mut hasher);
    env.world().daylight().to_bits().hash(&mut hasher);
    noise_index.hash(&mut hasher);
    hasher.finish()
}

fn noise_value(seed: u64, x: u64, y: u64) -> f64 {
    let mut value =
        seed ^ x.wrapping_mul(0x9E37_79B1_85EB_CA87) ^ y.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    value ^= value >> 33;
    value = value.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    value ^= value >> 33;
    value = value.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    value ^= value >> 33;
    let unit = (value as u32) as f64 / u32::MAX as f64;
    32.0 + unit * 95.0
}

type VignetteCache = Mutex<HashMap<(usize, usize, u64), Arc<Vec<f64>>>>;

fn vignette(size: [usize; 2], stddev: f64) -> Arc<Vec<f64>> {
    static VIGNETTE_CACHE: OnceLock<VignetteCache> = OnceLock::new();
    let cache = VIGNETTE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (size[0], size[1], stddev.to_bits());
    if let Some(mask) = cache.lock().expect("vignette cache poisoned").get(&key) {
        return mask.clone();
    }

    let mut values = Vec::with_capacity(size[0] * size[1]);
    for y in 0..size[1] {
        let ys = if size[1] <= 1 {
            0.0
        } else {
            -1.0 + 2.0 * y as f64 / (size[1] - 1) as f64
        };
        for x in 0..size[0] {
            let xs = if size[0] <= 1 {
                0.0
            } else {
                -1.0 + 2.0 * x as f64 / (size[0] - 1) as f64
            };
            let gaussian = (-0.5 * (xs * xs + ys * ys) / (stddev * stddev)).exp();
            values.push(1.0 - gaussian);
        }
    }
    let values = Arc::new(values);
    cache
        .lock()
        .expect("vignette cache poisoned")
        .insert(key, values.clone());
    values
}

fn textures() -> &'static TextureAtlas {
    static TEXTURES: OnceLock<TextureAtlas> = OnceLock::new();
    TEXTURES.get_or_init(|| {
        TextureAtlas::load_from_dir(&default_texture_dir())
            .unwrap_or_else(|error| panic!("failed to load Crafter textures: {error}"))
    })
}

fn default_texture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("crafter")
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct TextureKey {
    name: String,
    width: u32,
    height: u32,
}

struct TextureAtlas {
    originals: HashMap<String, Arc<RgbaImage>>,
    resized: Mutex<HashMap<TextureKey, Arc<RgbaImage>>>,
}

impl TextureAtlas {
    fn load_from_dir(path: &Path) -> Result<Self, String> {
        let mut originals = HashMap::new();
        let entries = std::fs::read_dir(path).map_err(|error| error.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("png") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .ok_or_else(|| format!("invalid texture filename: {}", path.display()))?
                .to_string();
            let image = ImageReader::open(&path)
                .map_err(|error| error.to_string())?
                .decode()
                .map_err(|error| error.to_string())?
                .to_rgba8();
            originals.insert(stem, Arc::new(image));
        }
        if !originals.contains_key("unknown") {
            return Err("missing required unknown.png texture".to_string());
        }
        Ok(Self {
            originals,
            resized: Mutex::new(HashMap::new()),
        })
    }

    fn get(&self, name: &str, size: [usize; 2]) -> Arc<RgbaImage> {
        let width = size[0] as u32;
        let height = size[1] as u32;
        let actual_name = if self.originals.contains_key(name) {
            name
        } else {
            "unknown"
        };
        let key = TextureKey {
            name: actual_name.to_string(),
            width,
            height,
        };
        if let Some(texture) = self
            .resized
            .lock()
            .expect("texture cache poisoned")
            .get(&key)
            .cloned()
        {
            return texture;
        }
        let original = self
            .originals
            .get(actual_name)
            .or_else(|| self.originals.get("unknown"))
            .expect("unknown texture missing");
        let resized = resize_nearest(original.as_ref(), width, height);
        let resized = Arc::new(resized);
        self.resized
            .lock()
            .expect("texture cache poisoned")
            .insert(key, resized.clone());
        resized
    }
}

fn resize_nearest(source: &RgbaImage, width: u32, height: u32) -> RgbaImage {
    if source.width() == width && source.height() == height {
        return source.clone();
    }

    let mut resized = RgbaImage::new(width, height);
    for y in 0..height {
        let src_y = resize_index(source.height(), height, y);
        for x in 0..width {
            let src_x = resize_index(source.width(), width, x);
            let pixel: Rgba<u8> = *source.get_pixel(src_x, src_y);
            resized.put_pixel(x, y, pixel);
        }
    }
    resized
}

fn resize_index(source: u32, dest: u32, coord: u32) -> u32 {
    if source == 16 && (dest as usize) < PILLOW_NEAREST_16.len() {
        return PILLOW_NEAREST_16[dest as usize][coord as usize] as u32;
    }
    ((coord as u64 * source as u64) / dest as u64).min(source.saturating_sub(1) as u64) as u32
}
