use std::fs;
use toml::Value;

#[derive(Debug)]
pub struct IcingaConfig {
    pub config_path: String,
    pub api_url: String,
    pub api_user: String,
    pub api_password: String,
    pub debug: bool,
    pub pings: Vec<PingConfig>,
    pub sleep_duration: u64,
}

#[derive(Debug)]
pub struct PingConfig {
    pub name: String,
    pub host: String,
}

pub fn load_config() -> IcingaConfig {
    let mut config_paths = vec!["config.toml".to_string()];
    
    if let Some(home) = std::env::var_os("HOME") {
        let home = home.to_string_lossy();
        config_paths.push(format!("{}/.icinga_passive_checks.toml", home));
        config_paths.push(format!("{}/.config/icinga_passive_checks.toml", home));
    }
    
    config_paths.push("/etc/icinga_passive_checks.toml".to_string());

    let config_path = config_paths.into_iter()
        .find(|path| std::path::Path::new(path).exists())
        .unwrap_or_else(|| {
            println!("Error: No config file found in standard locations");
            std::process::exit(1);
        });

    let config_content = fs::read_to_string(&config_path)
        .expect(&format!("Failed to read config file: {}", config_path));
    let config_data: Value = toml::from_str(&config_content).expect("Failed to parse config file");

    // Rest of the function remains the same
    let icinga = match config_data["icinga"].as_table() {
        Some(table) => table,
        None => {
            println!("Error: Missing 'icinga' section in config file");
            std::process::exit(1);
        }
    };

    let api_url = match icinga.get("api_url").and_then(|v| v.as_str()) {
        Some(url) => url.to_string(),
        None => {
            println!("Error: Missing 'api_url' in the icinga section");
            std::process::exit(1);
        }
    };

    let api_user = match icinga.get("api_user").and_then(|v| v.as_str()) {
        Some(user) => user.to_string(),
        None => {
            println!("Error: Missing 'api_user' in the icinga section");
            std::process::exit(1);
        }
    };

    let api_password = match icinga.get("api_password").and_then(|v| v.as_str()) {
        Some(password) => password.to_string(),
        None => {
            println!("Error: Missing 'api_password' in the icinga section");
            std::process::exit(1);
        }
    };

    let debug = match config_data.get("command").and_then(|c| c.get("debug").and_then(|v| v.as_bool())) {
        Some(debug) => debug,
        None => false,
    };    

    let pings = match config_data.get("ping").and_then(|p| p.as_array()) {
        Some(ping_array) => ping_array.iter().map(|ping| {
            PingConfig {
                name: match ping.get("name").and_then(|v| v.as_str()) {
                    Some(name) => name.to_string(),
                    None => {
                        println!("Error: Missing 'name' in a ping section");
                        std::process::exit(1);
                    }
                },
                host: match ping.get("host").and_then(|v| v.as_str()) {
                    Some(host) => host.to_string(),
                    None => {
                        println!("Error: Missing 'host' in a ping section");
                        std::process::exit(1);
                    }
                },
            }
        }).collect(),
        None => Vec::new(),
    };

    let sleep_duration = match config_data.get("daemon")
        .and_then(|d| d.as_table())
        .and_then(|d| d.get("sleep_duration"))
        .and_then(|v| v.as_integer()) {
        Some(duration) => duration as u64,
        None => 60,
    };

    IcingaConfig {
        config_path,
        api_url,
        api_user,
        api_password,
        debug,
        pings,
        sleep_duration,
    }
}
