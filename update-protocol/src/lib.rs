use std::fmt;
use serde::{Serializer, Deserializer};
use serde::{Serialize, Deserialize, de::{self, Visitor}};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionInfo {
    pub plugin_name: String,
    pub plugin_version: String,
    pub skyline_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseCode {
    NoUpdate,
    Update,
    PluginNotFound,
    InvalidRequest,
}

impl Default for ResponseCode {
    fn default() -> Self {
        Self::NoUpdate
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateResponse {
    pub code: ResponseCode,
    pub update_plugin: bool,
    pub update_skyline: bool,
    pub new_plugin_version: Option<String>,
    pub new_skyline_version: Option<String>,
    pub required_files: Vec<UpdateFile>,
}

impl UpdateResponse {
    pub fn no_update() -> Self {
        Default::default()
    }

    pub fn plugin_not_found() -> Self {
        Self {
            code: ResponseCode::PluginNotFound,
            ..Default::default()
        }
    }

    pub fn invalid_request() -> Self {
        Self {
            code: ResponseCode::InvalidRequest,
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateFile {
    #[serde(deserialize_with = "deserialize_field_kind")]
    pub install_location: InstallLocation,

    pub download_port: u16,
    pub size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateRequest {
    pub plugin_name: String,
    pub plugin_version: String,
    pub beta: Option<bool>,
}

// For allowing deserialization of unknown
fn deserialize_field_kind<'de, D>(deserializer: D) -> Result<InstallLocation, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(InstallLocation::deserialize(deserializer).unwrap_or(InstallLocation::Unknown))
}

#[derive(Debug, Clone)]
pub enum InstallLocation {
    AbsolutePath(String),
    Unknown,
}

struct InstallLocationVisitor;

impl Serialize for InstallLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
            S: Serializer {
        match self {
            InstallLocation::AbsolutePath(path) => serializer.serialize_str(path),
            _ => todo!()
        }
    }
}

impl<'de> Visitor<'de> for InstallLocationVisitor {
    type Value = InstallLocation;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
            E: de::Error, {
        Ok(InstallLocation::AbsolutePath(v.to_owned()))
    }

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid semver version string")
    }
}

impl<'de> Deserialize<'de> for InstallLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
            D: Deserializer<'de> {
        deserializer.deserialize_string(InstallLocationVisitor)
    }
}
