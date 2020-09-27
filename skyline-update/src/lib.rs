use std::path::PathBuf;
use std::io::prelude::*;
use std::net::{TcpStream, IpAddr};

use update_protocol::{UpdateRequest, UpdateResponse, ResponseCode};

const PORT: u16 = 45000;

#[cfg(target_os = "switch")]
fn should_update(response: &UpdateResponse) -> bool {
    true
}

#[cfg(not(target_os = "switch"))]
fn should_update(_: &UpdateResponse) -> bool {
    true
}

fn update(ip: IpAddr, response: UpdateResponse) {
    for file in response.required_files {
        if let Ok(mut stream) = TcpStream::connect((ip, file.download_port)) {
            let mut buf = vec![];
            if let Err(e) = stream.read_to_end(&mut buf) {
                println!("[updater] Error downloading file: {}", e);
                continue
            }
            let path: PathBuf = match file.install_location {
                update_protocol::InstallLocation::AbsolutePath(path) => path.into(),
                _ => continue
            };

            // Don't actually install things if we're not running on switch
            #[cfg(not(target_os = "switch"))] {
                println!("Installing {} bytes to path {}", buf.len(), path.display());

                if let Ok(string) = String::from_utf8(buf) {
                    println!("As string: {:?}", string);
                }
            }

            #[cfg(target_os = "switch")]
            if let Err(e) = std::fs::write(path, buf) {
                println!("[updater] Error writing file to sd: {}", e);
            }
        } else {
            println!("[updater] Failed to connect to port {}", file.download_port);
        }
    }
}

pub fn check_update(ip: IpAddr, name: &str, version: &str, allow_beta: bool) {
    if let Ok(mut stream) = TcpStream::connect((ip, PORT)) {
        if let Ok(packet) = serde_json::to_string(&UpdateRequest {
            beta: Some(allow_beta),
            plugin_name: name.to_owned(),
            plugin_version: version.to_owned(),
        }) {
            let _ = stream.write_fmt(format_args!("{}\n", packet));
            let mut string = String::new();
            let _ = stream.read_to_string(&mut string);

            if let Ok(response) = serde_json::from_str::<UpdateResponse>(&string) {
                match response.code {
                    ResponseCode::NoUpdate => return,
                    ResponseCode::Update => {
                        if should_update(&response) {
                            update(ip, response);
                        }
                    }
                    ResponseCode::InvalidRequest => {
                        println!("[{} updater] Failed to send a valid request to the server", name);
                    }
                    ResponseCode::PluginNotFound => {
                        println!("Plugin '{}' could not be found on the update server", name);
                    }
                }
            } else {
                println!("[{} updater] Failed to parse update server response: {:?}", name, string);
            }
        }
    } else {
        println!("[{} updater] Failed to connect to update server {}", name, ip);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_install() {
        check_update("127.0.0.1".parse().unwrap(), "test_plugin", "0.9.0", true);
    }
}
