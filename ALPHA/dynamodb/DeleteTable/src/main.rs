/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};

use std::error::Error;
use std::process;

use dynamodb::operation::DeleteTable;
use dynamodb::Region;

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
        .arg(
            Arg::with_name("table")
                .short("t")
                .long("table")
                .value_name("TABLE")
                .help("Specifies the table to create")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");

    // TABLE AND KEY FOR TESTING ONLY. DELETE BEFORE RELEASING.
    let table = matches.value_of("table").unwrap_or("users");
    if table == "" {
        println!("You must supply a table name");
        println!("-t TABLE)");
        process::exit(1);
    }

    println!("Region: {}", region);

    println!("DynamoDB client version: {}", dynamodb::PKG_VERSION);
    let config = dynamodb::Config::builder()
        .region(Region::from(region))
        .build();
    let client = aws_hyper::Client::https();

    let op = DeleteTable::builder().table_name(table).build(&config);

    client.call(op).await?;

    println!("Deleted table {}", table);

    Ok(())
}
