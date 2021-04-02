/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};

use std::fs;
use std::process;
use std::str;

use kms::operation::Decrypt;
use kms::Blob;
use kms::Region;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[tokio::main]
async fn main() {
    let matches = App::new("myapp")
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .value_name("REGION")
                .help("Specifies the region")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("KEY")
                .help("Specifies the encryption key")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("text")
                .short("t")
                .long("text")
                .value_name("TEXT")
                .help("Specifies file with encrypted text to decrypt")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");
    let key = matches
        .value_of("key")
        .unwrap_or("0249d461-754b-4cbd-b874-372b294fa354");
    let text = matches.value_of("text").unwrap_or("");

    if region == "" || key == "" || text == "" {
        println!("You must supply a value for region, key, and file containing encrypted text (-r REGION -k KEY -t TEXT-FILE)");

        process::exit(1);
    }

    println!("Region:    {}", region);
    println!("Key:       {}", key);
    println!("Text file: {}", text);

    SubscriberBuilder::default()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let config = kms::Config::builder().region(Region::from(region)).build();

    let client = aws_hyper::Client::https();

    // Vector to hold the string parts/u8 values
    let mut v_bytes: Vec<u8> = Vec::new();
    let mut i = 0;
    //let mut num: u8;

    // Open text file and get contents as a string
    match fs::read_to_string(text) {
        Ok(input) => {
            // Split the string into parts by comma
            let parts = input.split(",");
            for s in parts {
                // Each part should be a u8
                // Print out the string so we can tell if it's a legit u8
                // Trim any trailing line feed
                let mut my_string = String::from(s);
                /*
                if my_string.ends_with('\n') {
                    println!("Dropping \\n from string");
                    my_string.pop();
                    if my_string.ends_with('\r') {
                        println!("Dropping \\r from string");
                        my_string.pop();
                    }
                }
                */

                let len = my_string.trim_end_matches(&['\r', '\n'][..]).len();
                my_string.truncate(len);

                println!("String at {}: '{}'", i, my_string);

                match my_string.parse::<u8>() {
                    Ok(num) => v_bytes.push(num),
                    Err(e) => {
                        println!("Got an error parsing '{}'", s);
                        panic!("{}", e)
                    }
                }
                i = i + 1;
            }
        }
        Err(_) => println!("Could not parse {} as a string", text),
    }

    // Format of the file:
    //   BYTE1, ..., BYTEN

    //let mut buf = text.as_bytes();

    /*
        if text == "" {
            buf = &[
                1u8, 2u8, 2u8, 0u8, 120u8, 45u8, 38u8, 92u8, 185u8, 80u8, 21u8, 159u8, 165u8, 76u8,
                198u8, 157u8, 254u8, 232u8, 51u8, 65u8, 96u8, 186u8, 181u8, 89u8, 108u8, 127u8, 9u8,
                91u8, 193u8, 213u8, 185u8, 211u8, 65u8, 89u8, 139u8, 47u8, 89u8, 1u8, 121u8, 251u8,
                127u8, 140u8, 106u8, 72u8, 73u8, 77u8, 86u8, 149u8, 106u8, 180u8, 51u8, 247u8, 242u8,
                203u8, 0u8, 0u8, 0u8, 115u8, 48u8, 113u8, 6u8, 9u8, 42u8, 134u8, 72u8, 134u8, 247u8,
                13u8, 1u8, 7u8, 6u8, 160u8, 100u8, 48u8, 98u8, 2u8, 1u8, 0u8, 48u8, 93u8, 6u8, 9u8,
                42u8, 134u8, 72u8, 134u8, 247u8, 13u8, 1u8, 7u8, 1u8, 48u8, 30u8, 6u8, 9u8, 96u8,
                134u8, 72u8, 1u8, 101u8, 3u8, 4u8, 1u8, 46u8, 48u8, 17u8, 4u8, 12u8, 127u8, 158u8,
                249u8, 221u8, 238u8, 76u8, 50u8, 186u8, 19u8, 227u8, 217u8, 171u8, 2u8, 1u8, 16u8,
                128u8, 48u8, 231u8, 36u8, 43u8, 64u8, 19u8, 78u8, 101u8, 237u8, 115u8, 30u8, 241u8,
                85u8, 102u8, 164u8, 168u8, 199u8, 193u8, 102u8, 118u8, 138u8, 27u8, 8u8, 194u8, 100u8,
                85u8, 81u8, 160u8, 18u8, 78u8, 23u8, 33u8, 27u8, 12u8, 84u8, 209u8, 165u8, 100u8, 31u8,
                194u8, 179u8, 197u8, 122u8, 182u8, 183u8, 95u8, 247u8, 219u8, 162u8,
            ];
        }
    */
    let blob = Blob::new(v_bytes);

    let resp = client
        .call(
            Decrypt::builder()
                .key_id(key)
                .ciphertext_blob(blob)
                .build(&config),
        )
        .await
        .expect("failed to decrypt text");

    let inner = resp.plaintext.unwrap();

    let bytes = inner.as_ref();

    let s = match str::from_utf8(&bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    println!("String: {}", s)
}
