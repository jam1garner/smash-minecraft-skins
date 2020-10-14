#![feature(proc_macro_hygiene, new_uninit)]

use std::fs;
use std::time::Instant;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

use serde::Deserialize;
use parking_lot::Mutex;
use image::DynamicImage;
use skyline_web::Webpage;
use ramhorns::{Template, Content};

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
                    .filter_map(|(path, skin)| Some((&skin[..], fs::read(path).ok()?)))
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
            url => Skin::Custom(url[LOCALHOST.len()..].to_owned())
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

extern "C" fn steve_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_NUTEXB_FILES.iter().position(|&x| x == hash) {
        OPENED[slot].store(false, Ordering::SeqCst);
        let skin_path = SELECTED_SKINS[slot].lock();
        let skin_path: Option<&Path> = skin_path.as_deref();
        let skin_data = skin_path.map(|path| image::load_from_memory(&fs::read(path).unwrap()).unwrap())
                                .unwrap_or_else(|| image::load_from_memory(STEVE_PNG).unwrap());

        let mut skin_data = skin_data.to_rgba();

        for pixel in skin_data.iter_mut() {
            *pixel = (((*pixel) as usize) * 722 / 1000) as u8;
        }

        let mut data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);
        nutexb::writer::write_nutexb("steve_minecraft???", &DynamicImage::ImageRgba8(skin_data), &mut writer).unwrap();

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
    unsafe {
        for hash in &STEVE_BNTX_FILES {
            subscribe_callback_with_size(*hash, 0x10000, "nus3audio".as_ptr(), "nus3audio".len(), steve_ui_callback);
        }
        for hash in &STEVE_NUTEXB_FILES {
            subscribe_callback_with_size(*hash, 0x40b0, "nutexb".as_ptr(), "nutexb".len(), steve_callback);
        }
    }
}
