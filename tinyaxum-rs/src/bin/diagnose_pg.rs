use std::{
    thread,
    time::{Duration, Instant},
};

use chrono::Local;
use config::Config;
use postgres::{Client, NoTls};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting PostgreSQL diagnostic tool...");

    // Load configuration
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let db_type = settings.get_string("db.db_type")?;
    if db_type != "postgres" {
        eprintln!(
            "Error: This tool is for PostgreSQL only. Current db_type: {}",
            db_type
        );
        return Ok(());
    }

    let host = settings.get_string("db.url")?;
    let port = settings.get_int("db.port")?;
    let user = settings.get_string("db.username")?;
    let password = settings.get_string("db.password")?;
    let db_name = settings.get_string("db.db_name")?;

    let connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, db_name
    );

    println!(
        "Connecting to: host={} port={} dbname={}",
        host, port, db_name
    );

    let mut client = match Client::connect(&connection_string, NoTls) {
        Ok(client) => {
            println!("Successfully connected to database!");
            client
        }
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Ok(());
        }
    };

    println!("Starting heartbeat loop (interval: 60s)...");
    println!("Press Ctrl+C to stop.");

    let mut counter = 0;
    loop {
        counter += 1;
        let start = Instant::now();

        match client.query_one("SELECT 1", &[]) {
            Ok(_) => {
                let duration = start.elapsed();
                println!(
                    "[{}] Heartbeat #{} success. Duration: {:?}",
                    Local::now().format("%H:%M:%S"),
                    counter,
                    duration
                );
            }
            Err(e) => {
                eprintln!(
                    "[{}] Heartbeat #{} FAILED: {}",
                    Local::now().format("%H:%M:%S"),
                    counter,
                    e
                );
                eprintln!("Connection lost or database unresponsive.");

                // Attempt to reconnect
                println!("Attempting to reconnect...");
                match Client::connect(&connection_string, NoTls) {
                    Ok(new_client) => {
                        println!("Reconnection successful!");
                        client = new_client;
                    }
                    Err(re_err) => {
                        eprintln!("Reconnection failed: {}", re_err);
                    }
                }
            }
        }

        thread::sleep(Duration::from_secs(60));
    }
}
