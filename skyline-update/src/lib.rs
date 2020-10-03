use std::path::PathBuf;
use std::io::prelude::*;
use std::net::{TcpStream, IpAddr};

use update_protocol::{Request, ResponseCode};

pub use update_protocol::UpdateResponse;

const PORT: u16 = 45000;

pub struct DefaultInstaller;

#[cfg(not(target_os = "switch"))]
impl Installer for DefaultInstaller {
    fn should_update(&self, _: &UpdateResponse) -> bool {
        true
    }

    fn install_file(&self, path: PathBuf, buf: Vec<u8>) -> Result<(), ()> {
        println!("Installing {} bytes to path {}", buf.len(), path.display());

        if let Ok(string) = String::from_utf8(buf) {
            println!("As string: {:?}", string);
        }

        Ok(())
    }
}

#[cfg(target_os = "switch")]
impl Installer for DefaultInstaller {
    fn should_update(&self, response: &UpdateResponse) -> bool {
        skyline_web::Dialog::yes_no(format!(
            "An update for {} has been found.\nWould you like to download it?",
            response.plugin_name
        ))
    }

    fn install_file(&self, path: PathBuf, buf: Vec<u8>) -> Result<(), ()> {
        let _ = std::fs::create_dir_all(path.parent().ok_or(())?);
        if let Err(e) = std::fs::write(path, buf) {
            println!("[updater] Error writing file to sd: {}", e);
            Err(())
        } else {
            Ok(())
        }
    }
}

/// An installer for use with custom_check_update
pub trait Installer {
    fn should_update(&self, response: &UpdateResponse) -> bool;
    fn install_file(&self, path: PathBuf, buf: Vec<u8>) -> Result<(), ()>;
}

fn update<I>(ip: IpAddr, response: UpdateResponse, installer: &I) -> bool
    where I: Installer,
{
    for file in response.required_files {
        if let Ok(mut stream) = TcpStream::connect((ip, file.download_port)) {
            let mut buf = vec![];
            if let Err(e) = stream.read_to_end(&mut buf) {
                println!("[updater] Error downloading file: {}", e);
                return false
            }
            let path: PathBuf = match file.install_location {
                update_protocol::InstallLocation::AbsolutePath(path) => path.into(),
                _ => return false
            };

            if installer.install_file(path, buf).is_err() {
                return false
            }
        } else {
            println!("[updater] Failed to connect to port {}", file.download_port);
            return false
        }
    }
    println!("[updater] finished updating plugin.");
    true
}

/// Install an update with a custom installer implementation
pub fn custom_check_update<I>(ip: IpAddr, name: &str, version: &str, allow_beta: bool, installer: &I) -> bool
    where I: Installer,
{
    match TcpStream::connect((ip, PORT)) {
        Ok(mut stream) =>  {
            if let Ok(packet) = serde_json::to_string(&Request::Update {
                beta: Some(allow_beta),
                plugin_name: name.to_owned(),
                plugin_version: version.to_owned(),
                options: None,
            }) {
                let _ = stream.write_fmt(format_args!("{}\n", packet));
                let mut string = String::new();
                let _ = stream.read_to_string(&mut string);

                if let Ok(response) = serde_json::from_str::<UpdateResponse>(&string) {
                    match response.code {
                        ResponseCode::NoUpdate => return false,
                        ResponseCode::Update => {
                            if installer.should_update(&response) {
                                let success = update(ip, response, installer);

                                if !success {
                                    println!("[{} updater] Failed to install update, files may be left in a broken state.", name);
                                }

                                success
                            } else {
                                false
                            }
                        }
                        ResponseCode::InvalidRequest => {
                            println!("[{} updater] Failed to send a valid request to the server", name);
                            false
                        }
                        ResponseCode::PluginNotFound => {
                            println!("Plugin '{}' could not be found on the update server", name);
                            false
                        }
                        _ => {
                            println!("Unexpected response");
                            false
                        }
                    }
                } else {
                    println!("[{} updater] Failed to parse update server response: {:?}", name, string);
                    false
                }
            } else {
                println!("[{} updater] Failed to encode packet", name);
                false
            }
        }
        Err(e) => {
            println!("[{} updater] Failed to connect to update server {}", name, ip);
            println!("[{} updater] {:?}", name, e);
            false
        }
    }
}

/// Install an update using the default installer
///
/// ## Args
/// * ip - IP address of server
/// * name - name of plugin to update
/// * version - current version of plugin
/// * allow_beta - allow beta versions to be offered
pub fn check_update(ip: IpAddr, name: &str, version: &str, allow_beta: bool) -> bool {
    custom_check_update(ip, name, version, allow_beta, &DefaultInstaller)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_install() {
        check_update("127.0.0.1".parse().unwrap(), "test_plugin", "0.9.0", true);
    }
}
