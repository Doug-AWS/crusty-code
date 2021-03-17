/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};
// use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process;

use dynamodb::input::create_table_input;
use dynamodb::model::{
    AttributeDefinition, /*AttributeValue, */ KeySchemaElement, KeyType,
    ProvisionedThroughput, ScalarAttributeType, /*TableStatus*/
};
use dynamodb::operation::CreateTable;

use dynamodb::Region; // dynamodb::{Credentials, Endpoint, Region};
use env_logger::Env;

// #[derive(Serialize, Deserialize)]
// struct TableNames {
//     names: Vec<String>,
// }

fn create_table(table_name: &str, key_name: &str) -> create_table_input::Builder {
    CreateTable::builder()
        .table_name(table_name)
        .key_schema(vec![KeySchemaElement::builder()
            .attribute_name(key_name)
            .key_type(KeyType::Hash)
            .build()])
        .attribute_definitions(vec![AttributeDefinition::builder()
            .attribute_name(key_name)
            .attribute_type(ScalarAttributeType::S)
            .build()])
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(10)
                .write_capacity_units(10)
                .build(),
        )
}

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
        println!("You must supply a table and key (-t TABLE -k KEY)");
        process::exit(1);
    }

    println!("Region: {}", region);
    println!("Table:  {}", table);
    println!("Key:    {}", key);

    println!("DynamoDB client version: {}", dynamodb::PKG_VERSION);
    let config = dynamodb::Config::builder()
        .region(Region::from(region))
        .build();

    let client = aws_hyper::Client::https();

    client
        .call(create_table(table, key).build(&config))
        .await
        .expect("failed to create table");

    println!(
        "Created table {} with key {} in {} region",
        table, key, region
    );

    Ok(())
}
