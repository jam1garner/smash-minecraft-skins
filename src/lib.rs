#![feature(proc_macro_hygiene, new_uninit)]

use std::fs;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

use serde::Deserialize;
use parking_lot::Mutex;
use skyline_web::Webpage;
use image::{Pixel, DynamicImage, GenericImage};
use ramhorns::{Template, Content};
use percent_encoding::percent_decode_str;

mod keyboard;

lazy_static::lazy_static! {
    static ref SKINS: Mutex<Skins> = Mutex::new(Skins::from_cache().unwrap_or_default());
}

#[derive(Content)]
struct SkinIcon<'a> {
    path: &'a str,
    left: isize,
    top: isize,
    button_left: isize,
    button_top: isize,
}

#[derive(Content)]
struct Rendered<'a> {
    skins: Vec<SkinIcon<'a>>,
    add_left: isize,
    add_top: isize,
    add_button_left: isize,
    add_button_top: isize,
}

#[derive(Default)]
struct Skins {
    skins: Vec<String>,
    skin_files: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
enum Skin {
    Steve,
    Custom(String),
    Add,
}

const LOCALHOST: &str = "http://localhost/";
const CACHE_DIR: &str = "sd:/atmosphere/contents/01006A800016E000/romfs/minecraft_skins";

fn index_to_image_x(i: isize) -> isize {
    ((i % 6) * 225) - 200
}

fn index_to_image_y(i: isize) -> isize {
    ((i / 6) * 225) - 200
}

fn index_to_image_x_y(i: isize) -> (isize, isize) {
    (index_to_image_x(i), index_to_image_y(i))
}

fn index_to_button_x(i: isize) -> isize {
    ((i % 6) * 225)
}

fn index_to_button_y(i: isize) -> isize {
    ((i / 6) * 225)
}

fn index_to_button_x_y(i: isize) -> (isize, isize) {
    (index_to_button_x(i), index_to_button_y(i))
}

static STEVE_PNG: &[u8] = include_bytes!("popup/steve.png");

fn fix_png(path: &Path) -> Option<Vec<u8>> {
    let (width, height) = image::image_dimensions(path).ok()?;

    if width * 2 == height {
        let img = fs::read(path).ok()?;
        let image = image::load_from_memory_with_format(&img, image::ImageFormat::Png).ok()?;
        let mut image_buffer = Vec::with_capacity(img.len());
        DynamicImage::ImageRgba8(convert_to_modern_skin(&image.to_rgba())) 
            .write_to(&mut image_buffer, image::ImageFormat::Png)
            .ok()?;

        Some(image_buffer)
    } else {
        fs::read(path).ok()
    }
}

impl Skins {
    fn from_cache() -> Option<Self> {
        let _ = fs::create_dir_all(CACHE_DIR);

        let mut skins = vec![];
        let mut skin_files = vec![];
        for entry in fs::read_dir(CACHE_DIR).ok()? {
            let entry = entry.ok()?;

            let path = Path::new(CACHE_DIR).join(entry.path());
            if path.is_file() && path.extension().map(|x| x == "png").unwrap_or(false) {
                skins.push(path.file_name()?.to_string_lossy().into_owned());
                skin_files.push(path);
            }
        }

        Some(Skins { skins, skin_files })
    }

    fn render(&self) -> Rendered {
        let mut skins = vec![];

        let mut i = 1;

        for skin in &self.skins {
            let (left, top) = index_to_image_x_y(i);
            let (button_left, button_top) = index_to_button_x_y(i);
            skins.push(SkinIcon {
                path: &skin,
                left,
                top,
                button_left,
                button_top
            });

            i += 1;
        }

        let (add_left, add_top) = index_to_image_x_y(i);
        let (add_button_left, add_button_top) = index_to_button_x_y(i);

        Rendered { skins, add_top, add_left, add_button_left, add_button_top }
    }

    fn to_html(&self) -> String {
        let tpl = Template::new(include_str!("popup/index.html")).unwrap();
        tpl.render(&self.render())
    }

    fn show_menu(&self) -> Skin {
        let response = Webpage::new()
            .file("index.html", &self.to_html())
            .file("steve.png", STEVE_PNG)
            .file("plus_skin.png", &include_bytes!("popup/plus_skin.png")[..])
            .files(
                &self.skin_files
                    .iter()
                    .zip(self.skins.iter())
                    .filter_map(|(path, skin)| Some((&skin[..], fix_png(&path)?)))
                    .collect::<Vec<(&str, Vec<u8>)>>()
            )
            .background(skyline_web::Background::BlurredScreenshot)
            .boot_display(skyline_web::BootDisplay::BlurredScreenshot)
            .open()
            .unwrap();

        match response.get_last_url().unwrap() {
            "http://localhost/steve" => Skin::Steve,
            "http://localhost/add" => Skin::Add,
            url if !url.starts_with(LOCALHOST) => Skin::Steve,
            url => Skin::Custom(percent_decode_str(&url[LOCALHOST.len()..]).decode_utf8_lossy().into_owned())
        }
    }

    fn get_skin_path(&mut self) -> Option<PathBuf> {
        loop {
            match self.show_menu() {
                Skin::Steve => return None,
                Skin::Custom(custom) => return Some(Path::new(CACHE_DIR).join(custom)),
                Skin::Add => {
                    let username = keyboard::ShowKeyboardArg::new()
                        .header_text("Enter Minecraft Username")
                        .show();

                    if let Some(username) = username {
                        match self.download_skin(&username) {
                            Some(skin) => return Some(skin),
                            None => continue
                        }
                    } else {
                        continue
                    }
                }
            }
        }
    }

    fn download_skin(&mut self, username: &str) -> Option<PathBuf> {
        let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", username);
        let response: NameId = minreq::get(url)
            .send()
            .ok()?
            .json()
            .ok()?;

        let url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", response.id);
        let response: Session = minreq::get(url)
            .send()
            .ok()?
            .json()
            .ok()?;

        let textures_b64 = response.properties.into_iter().find(|prop| prop.name == "textures")?;
        let textures_json = base64::decode(textures_b64.value).ok()?;
        let textures: Textures = serde_json::from_slice(&textures_json[..]).ok()?;

        let url = textures.textures.skin.url;
        let png = minreq::get(url)
            .send()
            .ok()?
            .into_bytes();

        let path = Path::new(CACHE_DIR).join(format!("{}.png", username));
        fs::write(&path, &png).ok()?;

        self.skins.push(format!("{}.png", username));
        self.skin_files.push(path.clone());

        Some(path)
    }
}

#[derive(Deserialize)]
struct NameId {
    name: String,
    id: String,
}

#[derive(Deserialize)]
struct Session {
    name: String,
    id: String,
    properties: Vec<Prop>,
}

#[derive(Deserialize)]
struct Prop {
    name: String,
    value: String,
}

#[derive(Deserialize)]
struct Textures {
    timestamp: usize,
    #[serde(rename = "profileId")]
    profile_id: String,
    #[serde(rename = "profileName")]
    profile_name: String,

    textures: TexturesInner,
}

#[derive(Deserialize)]
struct TexturesInner {
    #[serde(rename = "SKIN")]
    skin: TextureSkin,
}

#[derive(Deserialize)]
struct TextureSkin {
    url: String,
    metadata: Option<SkinMetadata>,
}

#[derive(Deserialize)]
struct SkinMetadata {
    model: String,
}

/*use skyline::nn::fs::GetEntryType;
use std::ffi::CStr;

#[skyline::hook(replace = GetEntryType)]
fn get_entry_type(x: u64, y: *const i8) {
    let path = unsafe { CStr::from_ptr(y) };
    
    println!("{:?}", path.to_str().unwrap());

    original!()(x, y)
}

extern "C" {
    #[link_name = "_ZN2nn4diag6detail5AbortEPKNS_6ResultE"]
    fn abort(result: u32) -> !;
}

#[skyline::hook(replace = abort)]
fn abort_hook(result: u32) -> ! {
    panic!("Abort with result {}", result);
}*/

static STEVE_NUTEXB_FILES: [u64; 8] = [
    smash::hash40("fighter/pickel/model/body/c00/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c01/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c02/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c03/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c04/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c05/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c06/def_pickel_001_col.nutexb"),
    smash::hash40("fighter/pickel/model/body/c07/def_pickel_001_col.nutexb"),
];

static STEVE_BNTX_FILES: [u64; 8] = [
    smash::hash40("sound/bank/fighter/se_pickel_c00.nus3audio"),
    smash::hash40("sound/bank/fighter/se_pickel_c01.nus3audio"),
    smash::hash40("sound/bank/fighter/se_pickel_c02.nus3audio"),
    smash::hash40("sound/bank/fighter/se_pickel_c03.nus3audio"),
    smash::hash40("sound/bank/fighter/se_pickel_c04.nus3audio"),
    smash::hash40("sound/bank/fighter/se_pickel_c05.nus3audio"),
    smash::hash40("sound/bank/fighter/se_pickel_c06.nus3audio"),
    smash::hash40("sound/bank/fighter/se_pickel_c07.nus3audio"),
];

static SELECTED_SKINS: [Mutex<Option<PathBuf>>; 8] = [
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
    parking_lot::const_mutex(None),
];

static LAST_SELECTED: AtomicUsize = AtomicUsize::new(0xFF);
static LAST_OPENED: Mutex<Option<Instant>> = parking_lot::const_mutex(None);

extern "C" {
    #[link_name = "_ZN2nn5prepo10PlayReport3AddEPKcS3_"]
    fn prepo_add_play_report(a: u64, b: u64, c: u64) -> u64;
}

#[skyline::hook(replace = prepo_add_play_report)]
fn prepo_add_play_report_hook(a: u64, b: u64, c: u64) -> u64 {
    LAST_SELECTED.store(0xFF, Ordering::SeqCst);

    original!()(a, b, c)
}

static OPENED: [AtomicBool; 8] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

type ArcCallback = extern "C" fn(u64, *mut u8, usize) -> bool;

extern "C" {
    fn subscribe_callback(hash: u64, callback: ArcCallback);
    fn subscribe_callback_with_size(hash: u64, filesize: u32, extension: *const u8, extension_len: usize, callback: ArcCallback);
}

const MAX_HEIGHT: usize = 256;
const MAX_WIDTH: usize = 256;
const MAX_DATA_SIZE: usize = (MAX_HEIGHT * MAX_WIDTH * 4);
const MAX_FILE_SIZE: usize = MAX_DATA_SIZE + 0xb0;

/// Copy from one area to another, then flip the resulting area
fn copy_flipped(image: &mut image::RgbaImage, from_pos: (u32, u32), size: (u32, u32), to_pos: (u32, u32)) {
    let (x, y) = from_pos;
    let (width, height) = size;
    let (to_x, to_y) = to_pos;
    image.copy_within(image::math::Rect { x, y, width, height }, to_x, to_y);
    
    image::imageops::flip_horizontal_in_place(&mut image.sub_image(to_x, to_y, width, height));
}

/// Copy from one area to another, shift horizontally (with wrapping) N pixels, then flip the resulting area
fn copy_rotated_right_flipped(image: &mut image::RgbaImage, from_pos: (u32, u32), size: (u32, u32), to_pos: (u32, u32), shift: u32) {
    let (x, y) = from_pos;
    let (width, height) = size;
    let (to_x, to_y) = to_pos;
    let shift = shift % width;
    
    image.copy_within(image::math::Rect { x, y, width: width - shift, height }, to_x + shift, to_y);
    image.copy_within(image::math::Rect { x: (width - shift), y, width: shift, height }, to_x, to_y);
    
    image::imageops::flip_horizontal_in_place(&mut image.sub_image(to_x, to_y, width, height));
}

fn convert_to_modern_skin(skin_data: &image::RgbaImage) -> image::RgbaImage {
    let mut new_skin = image::RgbaImage::new(64, 64);

    new_skin.copy_from(skin_data, 0, 0).unwrap();

    const ARM_SIZE: (u32, u32) = (4, 4);

    // Copy and flip the top of leg
    copy_flipped(&mut new_skin, (4, 16), ARM_SIZE, (20, 48));

    // Copy and flip the bottom of leg
    copy_flipped(&mut new_skin, (8, 16), ARM_SIZE, (24, 48));

    // Copy and flip the top of arm
    copy_flipped(&mut new_skin, (44, 16), ARM_SIZE, (36, 48));

    // Copy and flip the bottom of arm
    copy_flipped(&mut new_skin, (48, 16), ARM_SIZE, (40, 48));

    // Copy the leg sides
    copy_rotated_right_flipped(&mut new_skin, (0, 20), (16, 12), (16, 52), 4);
    
    // Copy the arm sides
    copy_rotated_right_flipped(&mut new_skin, (40, 20), (16, 12), (32, 52), 4);

    new_skin
}

extern "C" fn steve_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_NUTEXB_FILES.iter().position(|&x| x == hash) {
        OPENED[slot].store(false, Ordering::SeqCst);
        let skin_path = SELECTED_SKINS[slot].lock();
        let skin_path: Option<&Path> = skin_path.as_deref();

        let skin_data = if let Some(path) = skin_path {
            image::load_from_memory(&fs::read(path).unwrap()).unwrap()
        } else {
            return false
        };

        let mut skin_data = skin_data.to_rgba();

        if skin_data.dimensions() == (64, 32) {
            skin_data = convert_to_modern_skin(&skin_data);
        }

        for row in skin_data.rows_mut() {
            for pixel in row {
                let channels = pixel.channels_mut();
                // 0..3 - don't apply to alpha channel
                for i in 0..3 {
                    let pixel = &mut channels[i];
                    
                    // gamma brightening by a factor of 1.385, bounded to [0, 184]
                    *pixel = ((((*pixel as f64) / 255.0).powf(1.0f64 / 1.385f64) * 255.0) * 184.0 / 255.0) as u8;
                }
            }
        }

        let real_size = (skin_data.height() as usize * skin_data.width() as usize * 4) + 0xb0;

        let mut data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);
        nutexb::writer::write_nutexb("steve_minecraft???", &DynamicImage::ImageRgba8(skin_data), &mut writer).unwrap();

        let mut data_out = writer.into_inner();

        if real_size != MAX_FILE_SIZE {
            let start_of_header = real_size - 0xb0;

            let (from, to) = data_out.split_at_mut(MAX_DATA_SIZE);
            to.copy_from_slice(&from[start_of_header..real_size]);
        }

        true
    } else {
        false
    }
}

extern "C" fn steve_ui_callback(hash: u64, _: *mut u8, _: usize) -> bool {
    if let Some(slot) = STEVE_BNTX_FILES.iter().position(|&x| x == hash) {
        // Don't offer a selection for the same skin multiple times in a row
        if LAST_SELECTED.swap(slot, Ordering::SeqCst) != slot {
            if LAST_OPENED.lock().map(|time| time.elapsed().as_millis() > 100).unwrap_or(true) {
                if !OPENED[slot].swap(true, Ordering::SeqCst) {
                    *SELECTED_SKINS[slot].lock() = SKINS.lock().get_skin_path();
                }
            }

            *LAST_OPENED.lock() = Some(Instant::now());
        }
    }

    false
}

#[skyline::main(name = "minecraft_skins")]
pub fn main() {
    skyline::install_hooks!(prepo_add_play_report_hook);

    unsafe {
        for hash in &STEVE_BNTX_FILES {
            subscribe_callback_with_size(*hash, 0x10000, "nus3audio".as_ptr(), "nus3audio".len(), steve_ui_callback);
        }
        for hash in &STEVE_NUTEXB_FILES {
            subscribe_callback_with_size(*hash, MAX_FILE_SIZE as _, "nutexb".as_ptr(), "nutexb".len(), steve_callback);
        }
    }
}
