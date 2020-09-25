mod hosted_plugins;

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

use std::fs;
use std::sync::Arc;
use std::path::Path;
use std::net::TcpListener;
use std::io::{self, prelude::*, BufReader};

use color_eyre::eyre;

use semver::Version;
use update_protocol::{InstallLocation, UpdateRequest, UpdateResponse, ResponseCode, UpdateFile};

struct PluginFile {
    install: InstallLocation,
    data: Arc<Vec<u8>>,
    port: u16,
    socket: TcpListener,
}

impl From<&PluginFile> for UpdateFile {
    fn from(file: &PluginFile) -> Self {
        UpdateFile {
            download_port: file.port.clone(),
            install_location: file.install.clone()
        }
    }
}

struct Plugin {
    pub name: String,
    pub plugin_version: Version,
    pub files: Vec<PluginFile>,
    pub skyline_version: Version,
}

const PORT_NUM: u16 = 45000;
const MAX_PORTS: u16 = 999;

fn setup_plugin_ports() -> eyre::Result<Vec<Plugin>> {
    let plugins = hosted_plugins::get()?;

    if plugins.len() > MAX_PORTS as usize {
        Err(eyre::eyre!("Too many files. Increase max ports."))
    } else {
        let mut i = 0usize;
        plugins.into_iter()
            .map(|plugin|{
                let hosted_plugins::Plugin {
                    name, plugin_version, files, skyline_version
                } = plugin;

                let files = files.into_iter()
                    .map(|(install, data)|{
                        i += 1;
                        let port = PORT_NUM + i as u16;
                        let socket = TcpListener::bind(("127.0.0.1", port))?;
                        socket.set_nonblocking(true)?;
                        Ok(PluginFile {
                            install,
                            port,
                            socket,
                            data: Arc::new(data),
                        })
                    })
                    .collect::<eyre::Result<_>>()?;

                Ok(Plugin {
                    name,
                    plugin_version,
                    skyline_version,
                    files
                })
            })
            .collect()
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    //hosted_plugins::print_default();

    let plugins_dir = Path::new("plugins");
    if !plugins_dir.exists() {
        fs::create_dir(plugins_dir)?;
    }

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

    watcher.watch("plugins", RecursiveMode::Recursive).unwrap();

    let mut plugins = setup_plugin_ports()?;
    let main_port = TcpListener::bind(("127.0.0.1", PORT_NUM))?;
    main_port.set_nonblocking(true)?;

    crossbeam::scope(move |scope|{
        loop {
            match rx.try_recv() {
                Ok(notify::DebouncedEvent::Error(err, Some(path))) => {
                    println!("File watch error at path {}: {}", path.display(), err);
                }
                Ok(notify::DebouncedEvent::Error(err, None)) => {
                    println!("File watch error: {}", err);
                }
                Ok(_) => {
                    println!("Change detected: refreshing plugins...");
                    // clear plugins (close sockets)
                    plugins = Vec::with_capacity(0);
                    // setup new plugins
                    plugins = setup_plugin_ports()?;
                },
                Err(_) => {}
            }

            while let Ok((socket, _)) = main_port.accept() {
                let mut socket = BufReader::new(socket);
                let plugins = &plugins;
                let mut packet = String::new();
                let _ = socket.read_line(&mut packet);
                let response = if let Ok(request) = serde_json::from_str::<UpdateRequest>(&packet) {
                    let plugin = plugins.iter().find(|plugin| plugin.name == request.plugin_name);

                    if let Some(plugin) = plugin {
                        if let Ok(current_version) = request.plugin_version.parse::<Version>() {
                            if current_version < plugin.plugin_version {
                                UpdateResponse {
                                    code: ResponseCode::Update,
                                    update_plugin: true,
                                    update_skyline: false,
                                    new_plugin_version: Some(plugin.plugin_version.to_string()),
                                    new_skyline_version: None,
                                    required_files: plugin.files.iter().map(|file| file.into()).collect()
                                }
                            } else {
                                UpdateResponse::no_update()
                            }
                        } else {
                            UpdateResponse::invalid_request()
                        }
                    } else {
                        UpdateResponse::plugin_not_found()
                    }
                } else {
                    UpdateResponse::invalid_request()
                };

                let mut socket = socket.into_inner();
                let _ = socket.write(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes());
                let _ = socket.shutdown(std::net::Shutdown::Both);
            }

            for plugin in &plugins {
                for file in &plugin.files {
                    match file.socket.accept() {
                        Ok((mut socket, _)) => {
                            let data = Arc::clone(&file.data);
                            scope.spawn(move |_| {
                                socket.write_all(&data[..])
                            });
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => (),
                        Err(e) => {
                            println!("Error occurred in accept: {}", e);
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }).unwrap()
}
