use std::fs;
use std::path::{Path, PathBuf};

use image::DynamicImage;
use skyline_web::Webpage;
use ramhorns::{Template, Content};
use percent_encoding::percent_decode_str;

use crate::minecraft_api::*;
use crate::keyboard::ShowKeyboardArg;
use crate::modern_skin::convert_to_modern_skin;

const LOCALHOST: &str = "http://localhost/";
const CACHE_DIR: &str = "sd:/atmosphere/contents/01006A800016E000/romfs/minecraft_skins";

static STEVE_PNG: &[u8] = include_bytes!("popup/steve.png");

#[derive(Default)]
pub struct Skins {
    skins: Vec<String>,
    skin_files: Vec<PathBuf>,
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

#[derive(Debug, Clone, PartialEq)]
enum Skin {
    Steve,
    Custom(String),
    Add,
}

impl Skins {
    pub fn from_cache() -> Option<Self> {
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

    pub fn get_skin_path(&mut self) -> Option<PathBuf> {
        loop {
            match self.show_menu() {
                Skin::Steve => return None,
                Skin::Custom(custom) => return Some(Path::new(CACHE_DIR).join(custom)),
                Skin::Add => {
                    let username = ShowKeyboardArg::new()
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
    (i % 6) * 225
}

fn index_to_button_y(i: isize) -> isize {
    (i / 6) * 225
}

fn index_to_button_x_y(i: isize) -> (isize, isize) {
    (index_to_button_x(i), index_to_button_y(i))
}

fn fix_png(path: &Path) -> Option<Vec<u8>> {
    let (width, height) = image::image_dimensions(path).ok()?;

    if width == height * 2 {
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
