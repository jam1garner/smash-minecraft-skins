use serde::Deserialize;

#[derive(Deserialize)]
pub struct NameId {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize)]
pub struct Session {
    pub name: String,
    pub id: String,
    pub properties: Vec<Prop>,
}

#[derive(Deserialize)]
pub struct Prop {
    pub name: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct Textures {
    pub timestamp: usize,

    #[serde(rename = "profileId")]
    pub profile_id: String,

    #[serde(rename = "profileName")]
    pub profile_name: String,

    pub textures: TexturesInner,
}

#[derive(Deserialize)]
pub struct TexturesInner {
    #[serde(rename = "SKIN")]
    pub skin: TextureSkin,
}

#[derive(Deserialize)]
pub struct TextureSkin {
    pub url: String,
    pub metadata: Option<SkinMetadata>,
}

#[derive(Deserialize)]
pub struct SkinMetadata {
    pub model: String,
}
