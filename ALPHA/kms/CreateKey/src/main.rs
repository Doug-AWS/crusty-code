/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use std::error::Error;
// use std::process;

use kms::operation::CreateKey;
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
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");

    println!("Region: {}", region);

    SubscriberBuilder::default()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let config = kms::Config::builder().region(Region::from(region)).build();

    let client = aws_hyper::Client::https();

    let data = client
        .call(CreateKey::builder().build(&config))
        .await
        .expect("failed to create key");

    // println!("");
    //println!("{:?}", data.key_metadata);
    println!("");

    match data.key_metadata {
        None => println!("No metadata found"),
        Some(x) => match x.key_id {
            None => println!("No key id"),
            Some(k) => println!("{}", k),
        },
    }
    /*
        for item in data.key_metadata.iter() {
            println!("{:?}", item)
        }

        // parse into generic JSON value
        //let json_string: String = data.key_metadata.key_id;
        let root: Value = serde_json::from_str("");

        // access element using .get()

            {
          "data": [
            {
              "hostname": "a hostname"
            }
          ]
        }

        let hostname: Option<&str> = root
            .get("data")
            .and_then(|value| value.get(0))
            .and_then(|value| value.get("hostname"))
            .and_then(|value| value.as_str());

        // hostname is Some(string_value) if .data[0].hostname is a string,
        // and None if it was not found
        println!("hostname = {:?}", hostname); // = Some("a hostname")
    */
}
