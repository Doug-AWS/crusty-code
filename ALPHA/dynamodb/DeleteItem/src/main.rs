/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};
use std::collections::HashMap;
use std::error::Error;
use std::process;

use dynamodb::model::AttributeValue;
use dynamodb::operation::DeleteItem;
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
                .help("The primary key of the table")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("value")
                .short("v")
                .long("value")
                .value_name("VALUE")
                .help("The value of the primary key to delete")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");

    // TABLE AND KEY FOR TESTING ONLY. DELETE BEFORE RELEASING.
    let table = matches.value_of("table").unwrap_or("users");
    let key = matches.value_of("key").unwrap_or("");
    let value = matches.value_of("value").unwrap_or("");

    if table == "" || key == "" || value == "" {
        println!("You must supply a table, key name, and key value");
        println!("-t TABLE -k KEY -v VALUE)");
        process::exit(1);
    }

    println!("Region: {}", region);
    println!("Table:  {}", table);
    println!("Key:    {}", key);
    println!("VAlue:  {}", value);

    println!("DynamoDB client version: {}", dynamodb::PKG_VERSION);
    let config = dynamodb::Config::builder()
        .region(Region::from(region))
        .build();
    let client = aws_hyper::Client::https();

    let mut attrs: HashMap<String, AttributeValue> = HashMap::new();
    let value_av = AttributeValue::S(value.to_string());

    attrs.insert(key.to_owned(), value_av);

    let op = DeleteItem::builder()
        .table_name(table)
        .key(attrs)
        .build(&config);

    client.call(op).await?;

    println!("Deleted key {} with value {}", key, value);

    Ok(())
}
