/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};
use std::error::Error;
use std::process;

use dynamodb::model::AttributeValue;
use dynamodb::operation::Scan;
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
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("KEY")
                .help("Specifies the primary key of the table to create")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");
    let table = matches.value_of("table").unwrap_or("");
    let key = matches.value_of("key").unwrap_or("");

    if table == "" || key == "" {
        println!("You must supply a table and key name (-t TABLE -k KEY)");
        process::exit(1);
    }

    println!("Region: {}", region);
    println!("Table:  {}", table);

    println!("Value for region: {}", region);

    println!("DynamoDB client version: {}", dynamodb::PKG_VERSION);
    let config = dynamodb::Config::builder()
        .region(Region::from(region))
        .build();
    let client = aws_hyper::Client::https();

    let op = Scan::builder().table_name(table).build(&config);

    let output = client.call(op).await?;

    println!("\nItems in table {}:\n", table);

    for name in output.items.iter() {
        for my_item in name.iter() {
            for (key, value) in my_item {
                println!("{}", key);
                match value {
                    AttributeValue::S(val) => {
                        println!("  {}", val);
                    }
                    _ => {}
                }
            }

            println!("");
        }
    }

    Ok(())
}
