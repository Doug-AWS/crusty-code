use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    id: String,
    date: String,
    amount: String,
}

fn main() -> io::Result<()> {
    let mut f = File::open("config.json")?;
    let mut buffer = String::new();

    f.read_to_string(&mut buffer)?;

    println!("Content from config.json:");
    println!("{}", buffer);

    // Empty struct in case we get an error
    let empty_config = Config {
        name: "".to_owned(),
        id: "".to_owned(),
        date: "".to_owned(),
        amount: "".to_owned(),
    };

    // Dump file content into struct
    let c: Config = match serde_json::from_str(&buffer) {
        Ok(c) => c,
        _ => empty_config,
    };

    // Barf out struct members.
    // The values should match what we displayed earlier.
    println!("Name:   {}", c.name);
    println!("ID:     {}", c.id);
    println!("Date:   {}", c.date);
    println!("Amount: {}", c.amount);

    Ok(())
}
