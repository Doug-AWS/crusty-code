/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use clap::{App, Arg};
use std::collections::HashMap;
use std::error::Error;
use std::process;

use dynamodb::model::AttributeValue;
use dynamodb::operation::PutItem;
use dynamodb::Region;

use env_logger::Env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // account_type, age, first_name, last_name

    let matches = App::new("myapp")
        .arg(
            Arg::with_name("type")
                .short("y")
                .long("type")
                .value_name("TYPE")
                .help("The type of account: standard_user or admin")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("age")
                .short("a")
                .long("age")
                .value_name("AGE")
                .help("The age of the user")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("first")
                .short("f")
                .long("first")
                .value_name("FIRST")
                .help("The user's first name")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("last")
                .short("l")
                .long("last")
                .value_name("LAST")
                .help("The user's last name")
                .takes_value(true),
        )
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
            Arg::with_name("username")
                .short("u")
                .long("username")
                .value_name("USERNAME")
                .help("The user name, which is the primary key of the table")
                .takes_value(true),
        )
        .get_matches();

    let region = matches.value_of("region").unwrap_or("us-west-2");

    // TABLE AND KEY FOR TESTING ONLY. DELETE BEFORE RELEASING.
    let table = matches.value_of("table").unwrap_or("users");
    let username = matches.value_of("username").unwrap_or("");
    let a_type = matches.value_of("type").unwrap_or("");
    let age = matches.value_of("age").unwrap_or("");
    let first = matches.value_of("first").unwrap_or("");
    let last = matches.value_of("last").unwrap_or("");

    if table == "" || username == "" || a_type == "" || age == "" || first == "" || last == "" {
        println!("You must supply a table name, user name, type, age, and first and last names");
        println!("-t TABLE -u USER-NAME -y TYPE -a AGE -f FIRST-NAME =l LAST-NAME)");
        process::exit(1);
    }

    println!("Region: {}", region);
    println!("Table:  {}", table);
    println!("User:   {}", username);
    println!("Type:   {}", a_type);
    println!("Age:    {}", age);
    println!("First:  {}", first);
    println!("Last:   {}", last);

    println!("DynamoDB client version: {}", dynamodb::PKG_VERSION);
    let config = dynamodb::Config::builder()
        .region(Region::from(region))
        .build();
    let client = aws_hyper::Client::https();

    // Create hashmap(string, attributevalue)
    // using "username",
    let mut attrs: HashMap<String, AttributeValue> = HashMap::new();
    let user_av = AttributeValue::S(username.to_string());
    let type_av = AttributeValue::S(a_type.to_string());
    let age_av = AttributeValue::S(age.to_string());
    let first_av = AttributeValue::S(first.to_string());
    let last_av = AttributeValue::S(last.to_string());

    attrs.insert("username".to_owned(), user_av);
    attrs.insert("account_type".to_owned(), type_av);
    attrs.insert("age".to_owned(), age_av);
    attrs.insert("first_name".to_owned(), first_av);
    attrs.insert("last_name".to_owned(), last_av);

    let op = PutItem::builder()
        .table_name(table)
        .item(attrs)
        .build(&config);

    client.call(op).await?;

    println!(
        "Added user {}, {} {}, age {} as {} user",
        username, first, last, age, a_type
    );

    Ok(())
}
