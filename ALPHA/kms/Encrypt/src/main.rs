/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};

use std::process;
// use std::str;

use kms::operation::Encrypt;
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
                .help("Specifies the text to encrypt")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");
    let key = matches.value_of("key").unwrap_or("");
    let text = matches
        .value_of("text")
        .unwrap_or("This is text to encrypt");

    if region == "" || key == "" || text == "" {
        println!("You must supply a value for region, key, and text (-r REGION -k KEY -t \"TEXT\"");

        process::exit(1);
    }

    println!("Region: {}", region);
    println!("Key:    {}", key);
    println!("Text:   {}", text);

    SubscriberBuilder::default()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let config = kms::Config::builder().region(Region::from(region)).build();

    let client = aws_hyper::Client::https();

    let blob = Blob::new(text.as_bytes());

    let resp = client
        .call(
            Encrypt::builder()
                .key_id(key)
                .plaintext(blob)
                .build(&config),
        )
        .await
        .expect("failed to encrypt text");

    let inner = resp.ciphertext_blob.unwrap();

    let bytes = inner.as_ref();

    /* Get an error:

    let s = match str::from_utf8(&bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    */

    println!("Bytes: {:?}", bytes);

    let s = String::from_utf8_lossy(bytes);

    println!("String: {:?}", s);
}
