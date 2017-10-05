mod cli;
mod config;

use cli::{cli, get_value};
use config::{load_config};


fn main() {
    let cli = cli();

    let config_path = get_value(&cli,"config", "");
    //let ip = get_value(&cli,"ip", "127.0.0.1");
    //let port = get_value(&cli,"port", "8080");
    //let ssl_certificate = get_value(&cli,"cert", "");
    //let ssl_public_key = get_value(&cli,"key", "");

    let config = load_config(config_path);
    println!("{:?}", config);
}
