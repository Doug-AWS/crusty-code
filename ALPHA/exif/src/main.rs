extern crate exif;

// use std::env;
//use std::fs::File;
//use std::io::BufReader;
// WAS: use std::path::{Path, PathBuf};
//use std::path::Path;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The filename
    #[structopt(short, long)]
    filename: String,
}

fn add_data_to_table(height: String, time: String, width: String, table: String) {
    println!("Added new entry to table {}:", table);
    println!("height: {}", height);
    println!("width:  {}", width);
    println!("time:   {}", time);
    println!();
}

fn main() -> Result<(), exif::Error> {
    let Opt { filename } = Opt::from_args();
    for path in &[&filename] {
        let file = std::fs::File::open(path)?;
        let mut bufreader = std::io::BufReader::new(&file);
        let exifreader = exif::Reader::new();

        let exif = match exifreader.read_from_container(&mut bufreader) {
            Ok(exif) => exif,
            Err(_) => {
                println!();
                println!("File does not contain ELIF data");
                add_data_to_table(
                    String::from(""),
                    String::from(""),
                    String::from(""),
                    String::from("table"),
                );
                continue;
            }
        };

        println!("Called.");
        let mut height = String::new();
        let mut time = String::new();
        let mut width = String::new();

        println!("{}", filename);
        for f in exif.fields() {
            // Only show values for a few fields
            match &*f.tag.to_string() {
                "ImageWidth" => height.push_str(&*f.display_value().to_string()),
                "ImageLength" => width.push_str(&*f.display_value().to_string()),
                "DateTimeOriginal" => time.push_str(&*f.display_value().to_string()),
                _ => {}
            }
        }

        add_data_to_table(height, time, width, String::from("good_table"));
    }

    Ok(())
}
