# skyline-update

A server/library combo for updating skyline plugins written in Rust.

### Basic library usage

```rust
// Get a stable update of plugin "plugin_name" from IP `127.0.0.1` using the version in `Cargo.toml` to display the version of the current plugin
skyline_update::check_update("127.0.0.1".parse().unwrap(), "plugin_name", env!("CARGO_PKG_VERSION"), false);
```

### Basic server usage

Simply run the server in the background on the IP specified in the plugin. Plugins are located in the `plugins` folder of the current working directory. The structure of a plugin looks like so:

```
plugins
    L my_mod_name
        L plugin.toml
        L my_mod_name.nro
        L my_mod_file_dependency.txt
```

Things to note:

1. Each plugin is a subdirectory in the plugins folder (the name of the folder can be anything)
2. Each plugin folder must contain a `plugin.toml`
3. Each plugin folder should contain any other relevant files needed to be served

A `plugin.toml` looks like so:

```toml
version = "1.0.0"
name = "my_mod_name"
files = [
    { install_location = "sd:/atmosphere/contents/01006A800016E000/exefs/my_mod_name.nro", filename = "my_mod_name.nro" },
    { install_location = "sd:/my_mod_file_dependency.txt", filename = "my_mod_file_dependency.txt" }
]
```

#### Fields

* `version` - string, a valid semver version string representing the version of the plugin currently present in the folder. It is highly recommended this match your `Cargo.toml` of your plugin. Updates will only be shown to users if a newer version is present on the server.
* `name` - string, an identifier for your plugin. Must match the name provided in `skyline_update::check_update`, otherwise the plugin will not be found when attempting to update.
* `files` - A list of files to be installed if the user chooses to update.
  * `install_location` - where on the switch's SD card to install the update
  * `filename` - name of the file in the server. If the path is relative, it will be relative to the plugin folder.
* `skyline_version` (optional) - Minimum skyline version to use. Will update to the server's skyline if the current one is too low. (Currently supported)
* `beta` (optional) - Whether or not to treat this plugin as a beta version. The server can have multiple copies of the same plugin, however the highest version will always be installed. Whether or not beta versions are included is based on the boolean passed to `skyline_update::check_update`. If the stable version of a plugin has a higher version than the beta, . Defaults to `false`.

An example setup of the plugin server can be found in [`update-server/plugins`](https://github.com/skyline-rs/skyline-update/tree/master/update-server/plugins). It contains a single plugin with both a stable and a beta branch. 
