/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};
// use serde::{Deserialize, Serialize};
// use std::error::Error;
use std::process;

use kms::operation::GenerateRandom;
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
            Arg::with_name("text")
                .short("t")
                .long("text")
                .value_name("TEXT")
                .help("Specifies the text to encode")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("key_id")
                .short("k")
                .long("key_id")
                .value_name("KEY-ID")
                .help("Specifies the name of the key")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");
    let text = matches.value_of("text").unwrap_or("");
    let key_id = matches.value_of("key_id").unwrap_or("");

    if text == "" || key_id == "" {
        println!("You must supply text and a key (-t \"TEXT\" -k KEY)");
        process::exit(1);
    }

    println!("Region: {}", region);
    println!("Text:  {}", text);
    println!("Key:    {}", key_id);

    SubscriberBuilder::default()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let config = kms::Config::builder().region(Region::from(region)).build();

    let client = aws_hyper::Client::https();

    let data = client
        .call(GenerateRandom::builder().number_of_bytes(64).build(&config))
        .await
        .expect("failed to generate random data");

    println!("{:?}", data);

    assert_eq!(data.plaintext.expect("should have data").as_ref().len(), 64);
}
