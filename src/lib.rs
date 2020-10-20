#![feature(proc_macro_hygiene, new_uninit)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::Mutex;
use image::DynamicImage;

use skyline::hooks::InlineCtx;
use smash::lib::lua_const::FIGHTER_KIND_PICKEL;

mod keyboard;
mod skin_menu;
mod skin_files;
mod modern_skin;
mod minecraft_api;
mod color_correct;
mod stock_generation;

use skin_files::*;
use modern_skin::convert_to_modern_skin;

use color_correct::color_correct;

lazy_static::lazy_static! {
    static ref SKINS: Mutex<skin_menu::Skins> = Mutex::new(
        skin_menu::Skins::from_cache().unwrap_or_default()
    );
}

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

static RENDERS: [Mutex<Option<image::RgbaImage>>; 8] = [
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

extern "C" {
    #[link_name = "_ZN2nn5prepo10PlayReport3AddEPKcS3_"]
    fn prepo_add_play_report(a: u64, b: u64, c: u64) -> u64;
}

#[skyline::hook(replace = prepo_add_play_report)]
fn prepo_add_play_report_hook(a: u64, b: u64, c: u64) -> u64 {
    LAST_SELECTED.store(0xFF, Ordering::SeqCst);

    original!()(a, b, c)
}

type ArcCallback = extern "C" fn(u64, *mut u8, usize) -> bool;

extern "C" {
    fn subscribe_callback_with_size(hash: u64, filesize: u32, extension: *const u8, extension_len: usize, callback: ArcCallback);
}

const MAX_HEIGHT: usize = 1024;
const MAX_WIDTH: usize = 1024;
const MAX_DATA_SIZE: usize = MAX_HEIGHT * MAX_WIDTH * 4;
const MAX_FILE_SIZE: usize = MAX_DATA_SIZE + 0xb0;

extern "C" fn steve_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_NUTEXB_FILES.iter().position(|&x| x == hash) {
        let skin_path = SELECTED_SKINS[slot].lock();
        let skin_path: Option<&Path> = skin_path.as_deref();
        
        let data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);

        let skin_data = if let Some(path) = skin_path {
            image::load_from_memory(&fs::read(path).unwrap()).unwrap()
        } else {
            // load skin for arcrop, temp fix, TODO: change back to "return false" after arcrop works
            let data = match fs::read(Path::new("sd:/ultimate/mods/minecraft_2_layer").join(STEVE_NUTEXB_FILES_STR[slot])) {
                Ok(data) => data,
                Err(_) => return false,
            };
            
            use std::io::Write;

            let real_size = data.len();

            writer.write_all(&data).unwrap();
            let data_out = writer.into_inner();
            if real_size != MAX_FILE_SIZE {
                let start_of_header = real_size - 0xb0;

                let (from, to) = data_out.split_at_mut(MAX_DATA_SIZE);
                to.copy_from_slice(&from[start_of_header..real_size]);
            }

            return true
        };

        let mut skin_data = skin_data.to_rgba();

        let (width, height) = skin_data.dimensions();
        if width == height * 2 {
            skin_data = convert_to_modern_skin(&skin_data);
        }

        color_correct(&mut skin_data);

        //skin_data.save("sd:/test.png");

        let real_size = (skin_data.height() as usize * skin_data.width() as usize * 4) + 0xb0;

        nutexb::writer::write_nutexb("steve_minecraft???", &DynamicImage::ImageRgba8(skin_data), &mut writer).unwrap();

        let data_out = writer.into_inner();

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

extern "C" fn steve_stock_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_STOCK_ICONS.iter().position(|&x| x == hash) {
        let skin_path = SELECTED_SKINS[slot].lock();
        let skin_path: Option<&Path> = skin_path.as_deref();

        let skin_data = if let Some(path) = skin_path {
            image::load_from_memory(&fs::read(path).unwrap()).unwrap()
        } else {
            return false
        };

        let skin = skin_data.to_rgba();
        let (width, height) = skin.dimensions();
        let skin = if width == height * 2 {
            convert_to_modern_skin(&skin)
        } else {
            skin
        };
        let stock_icon = stock_generation::gen_stock_image(&skin);

        let data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);

        bntx::BntxFile::from_image(DynamicImage::ImageRgba8(stock_icon), "steve")
            .write(&mut writer)
            .unwrap();

        true
    } else {
        false
    }
}

#[derive(Debug)]
struct UnkPtr1 {
    ptrs: [&'static u64; 7],
}

#[derive(Debug)]
struct UnkPtr2 {
    bunch_bytes: [u8; 0x20],
    bunch_bytes2: [u8; 0x20]
}

#[derive(Debug)]
#[repr(C)]
pub struct FighterInfo {
    unk_ptr1: &'static UnkPtr1,
    unk_ptr2: &'static UnkPtr2,
    unk1: [u8; 0x20],
    unk2: [u8; 0x20],
    unk3: [u8; 0x8],
    fighter_id: u8,
    unk4: [u8;0xB],
    fighter_slot: u8,
}

#[skyline::hook(offset = 0x661acc, inline)]
fn css_fighter_selected(ctx: &InlineCtx) {
    let infos = unsafe { &*(ctx.registers[0].bindgen_union_field as *const FighterInfo) };

    let is_steve = *FIGHTER_KIND_PICKEL == infos.fighter_id as i32;

    if is_steve {
        let slot = infos.fighter_slot as usize;

        let path = SKINS.lock().get_skin_path();
        
        *SELECTED_SKINS[slot].lock() = path.clone();

        let mut render = RENDERS[slot].lock();
        let mut skin_data = if let Some(path) = path {
            image::load_from_memory(&fs::read(path).unwrap())
                .unwrap()
                .into_rgba()
        } else {
            *render = None;
            return
        };
        
        color_correct(&mut skin_data);
        
        *render = Some(minecraft_render::create_render(&skin_data));
    }
}

const MAX_STOCK_ICON_SIZE: u32 = 0x9c68;
const MAX_CHARA_3_SIZE: u32 = 0x727068;
const MAX_CHARA_4_SIZE: u32 = 0x2d068;
const MAX_CHARA_6_SIZE: u32 = 0x81068;

static CHARA_3_MASK: &[u8] = include_bytes!("chara_3_mask.png");
static CHARA_4_MASK: &[u8] = include_bytes!("chara_4_mask.png");
static CHARA_6_MASK: &[u8] = include_bytes!("chara_6_mask.png");

use parking_lot::{MutexGuard, MappedMutexGuard};

fn get_render<'a>(slot: usize) -> Option<MappedMutexGuard<'a, image::RgbaImage>> {
    let lock = RENDERS[slot].lock();
    if lock.is_none() {
        None
    } else {
        Some(MutexGuard::map(lock, |x| x.as_mut().unwrap()))
    }
}

extern "C" fn chara_3_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_CHARA_3.iter().position(|&x| x == hash) {
        let output = if let Some(render) = get_render(slot) {
            render
        } else {
            return false
        };

        let data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);

        let chara_3_mask = image::load_from_memory_with_format(CHARA_3_MASK, image::ImageFormat::Png)
            .unwrap()
            .into_rgba();

        let chara_3 = minecraft_render::create_chara_image(
            &output,
            &chara_3_mask,
            1.28451252f32,
            -456.55612f32,
            11.757321f32,
        );

        bntx::BntxFile::from_image(DynamicImage::ImageRgba8(chara_3), "steve")
            .write(&mut writer)
            .unwrap();

        true
    } else {
        false
    }
}

extern "C" fn chara_4_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_CHARA_4.iter().position(|&x| x == hash) {
        let output = if let Some(render) = get_render(slot) {
            render
        } else {
            return false
        };
        
        let data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);

        let chara_4_mask = image::load_from_memory_with_format(CHARA_4_MASK, image::ImageFormat::Png)
            .unwrap()
            .into_rgba();

        let chara_4 = minecraft_render::create_chara_image(
            &output,
            &chara_4_mask,
            0.232882008f32,
            -90.16959f32,
            9.084564f32,
        );

        bntx::BntxFile::from_image(DynamicImage::ImageRgba8(chara_4), "steve")
            .write(&mut writer)
            .unwrap();

        true
    } else {
        false
    }
}

extern "C" fn chara_6_callback(hash: u64, data: *mut u8, size: usize) -> bool {
    if let Some(slot) = STEVE_CHARA_6.iter().position(|&x| x == hash) {
        let output = if let Some(render) = get_render(slot) {
            render
        } else {
            return false
        };
        
        let data_out = unsafe { std::slice::from_raw_parts_mut(data, size) };
        let mut writer = std::io::Cursor::new(data_out);

        let chara_6_mask = image::load_from_memory_with_format(CHARA_6_MASK, image::ImageFormat::Png)
            .unwrap()
            .into_rgba();

        let chara_6 = minecraft_render::create_chara_image(
            &output,
            &chara_6_mask,
            0.938028f32,
            -480.87906f32,
            -96.13269f32,
        );

        bntx::BntxFile::from_image(DynamicImage::ImageRgba8(chara_6), "steve")
            .write(&mut writer)
            .unwrap();

        true
    } else {
        false
    }
}

#[skyline::main(name = "minecraft_skins")]
pub fn main() {
    skyline::install_hooks!(prepo_add_play_report_hook, css_fighter_selected);

    unsafe {
        for hash in &STEVE_NUTEXB_FILES {
            subscribe_callback_with_size(*hash, MAX_FILE_SIZE as _, "nutexb".as_ptr(), "nutexb".len(), steve_callback);
        }

        for hash in &STEVE_STOCK_ICONS {
            subscribe_callback_with_size(*hash, MAX_STOCK_ICON_SIZE as _, "bntx".as_ptr(), "bntx".len(), steve_stock_callback);
        }

        for hash in &STEVE_CHARA_3 {
            subscribe_callback_with_size(*hash, MAX_CHARA_3_SIZE as _, "bntx".as_ptr(), "bntx".len(), chara_3_callback);
        }

        for hash in &STEVE_CHARA_4 {
            subscribe_callback_with_size(*hash, MAX_CHARA_4_SIZE as _, "bntx".as_ptr(), "bntx".len(), chara_4_callback);
        }

        for hash in &STEVE_CHARA_6 {
            subscribe_callback_with_size(*hash, MAX_CHARA_6_SIZE as _, "bntx".as_ptr(), "bntx".len(), chara_6_callback);
        }
    }
}
