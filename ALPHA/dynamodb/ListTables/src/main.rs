/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};
use std::error::Error;

use dynamodb::operation::ListTables;
use dynamodb::Region; // dynamodb::{Credentials, Endpoint, Region};
use env_logger::Env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let matches = App::new("myapp")
        .arg(
            Arg::with_name("region")
                .short("r")
                .long("region")
                .value_name("REGION")
                .help("Specifies the region to create the table in")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");
    println!("Value for region: {}", region);

    println!("DynamoDB client version: {}", dynamodb::PKG_VERSION);
    let config = dynamodb::Config::builder()
        .region(Region::from(region))
        .build();
    let client = aws_hyper::Client::https();

    let op = ListTables::builder().build(&config);

    let tables = client.call(op).await?;

    let mut l = 0;

    for name in tables.table_names.iter() {
        for n in name.iter() {
            l = l + 1;
            println!("    {:?}", n);
        }
    }

    println!("\nFound {} tables in {} region.\n", l, region);

    Ok(())
}
