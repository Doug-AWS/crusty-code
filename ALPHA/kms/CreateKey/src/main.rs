/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};

use kms::Region;

use std::process;

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
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .value_name("VERBOSE")
                .help("Whether to display additional runtime information.")
                .takes_value(false),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");
    let verbose = matches.is_present("verbose");

    if verbose {
        println!("Region: {}", region);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    // Create client
    let config = kms::Config::builder().region(Region::from(region)).build();
    let client = kms::Client::from_conf_conn(config, aws_hyper::conn::Standard::https());

    match client.create_key().send().await {
        Ok(data) => match data.key_metadata {
            None => println!("No metadata found"),
            Some(x) => match x.key_id {
                None => println!("No key id"),
                Some(k) => println!("\n\nKey:\n{}", k),
            },
        },
        Err(_) => {
            println!("");
            process::exit(1);
        }
    };
    println!("");
}
