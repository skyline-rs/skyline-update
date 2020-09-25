use std::{io, fs};
use semver::Version;
use std::path::{Path, PathBuf};
use update_protocol::InstallLocation;
use serde::{Serialize, Deserialize};

use color_eyre::eyre;

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginFile {
    pub install_location: InstallLocation,
    pub filename: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PluginToml {
    #[serde(with = "version_parse")]
    pub version: Version,

    pub name: String,

    pub files: Vec<PluginFile>,

    #[serde(with = "version_parse")]
    pub skyline_version: Version,
}

mod version_parse {
    use core::fmt;
    use semver::Version;
    use serde::{Serializer, Deserializer, de::{self, Visitor}};

    pub fn serialize<S>(ver: &Version, ser: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        ser.collect_str(ver)
    }

    struct VerVisitor;

    impl<'de> Visitor<'de> for VerVisitor {
        type Value = Version;

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
                E: de::Error, {
            v.parse().map_err(|_| E::custom("Failed to parse version"))
        }

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a valid semver version string")
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Version, D::Error>
        where D: Deserializer<'de>
    {
        de.deserialize_string(VerVisitor)
    }
}

pub struct Plugin {
    pub name: String,
    pub plugin_version: Version,
    pub files: Vec<(InstallLocation, Vec<u8>)>,
    pub skyline_version: Version,
}

fn to_file(PluginFile { install_location, filename }: PluginFile, dir: &Path) -> eyre::Result<(InstallLocation, Vec<u8>)> {
    let path = if filename.is_absolute() {
        filename
    } else {
        dir.join(filename)
    };

    Ok((install_location, fs::read(path)?))
}

pub fn folder_to_plugin(dir: io::Result<fs::DirEntry>) -> eyre::Result<Option<Plugin>> {
    let path = dir?.path();
    if !path.is_dir() {
        return Ok(None)
    }
    let toml_path = path.join("plugin.toml");

    let plugin: PluginToml = toml::from_str(&fs::read_to_string(toml_path)?)?;

    let PluginToml { version, name, files, skyline_version } =  plugin;

    let files = files.into_iter().map(|file| to_file(file, &path)).collect::<eyre::Result<_>>()?;

    Ok(Some(Plugin {
        name,
        plugin_version: version,
        files,
        skyline_version
    }))
}

pub fn get() -> eyre::Result<Vec<Plugin>> {
    Ok(
        fs::read_dir("plugins")?
        .filter_map(|entry| {
            match folder_to_plugin(entry) {
                Ok(x) => x,
                Err(e) => {
                    println!("{}", e);
                    None
                }
            }
        })
        .collect()
    )
}

pub fn print_default() {
    println!("{}", toml::to_string_pretty(&PluginToml {
        name: "name".to_owned(),
        version: "1.0.0".parse().unwrap(),
        files: vec![],
        skyline_version: "0.0.0".parse().unwrap()
    }).unwrap());
}
