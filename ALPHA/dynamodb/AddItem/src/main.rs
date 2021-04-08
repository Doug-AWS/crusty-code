/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::collections::HashMap;
use std::process;

use dynamodb::model::AttributeValue;
use dynamodb::Region;

use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
#[structopt(name = "example", about = "An example of StructOpt usage.")]
struct Opt {
    /// The permission type of the user, standard_user or admin
    #[structopt(short, long)]
    p_type: String,

    /// The user's age
    #[structopt(short, long)]
    age: String,

    /// The user's username
    #[structopt(short, long)]
    username: String,

    /// The user's first name
    #[structopt(short, long)]
    first: String,

    /// The user's last name
    #[structopt(short, long)]
    last: String,

    /// The table name
    #[structopt(short, long)]
    table: String,

    /// The region
    #[structopt(default_value = "us-west-2", short, long)]
    region: String,

    /// Activate verbose mode    
    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    // account_type, age, first_name, last_name

    if opt.table == ""
        || opt.username == ""
        || opt.p_type == ""
        || opt.age == ""
        || opt.first == ""
        || opt.last == ""
    {
        println!("\nYou must supply a table name, user name, permission type, age, and first and last names");
        println!("-t TABLE -u USER-NAME -p PERMISSION-TYPE (admin or standard_user) -a AGE -f FIRST-NAME -l LAST-NAME)\n");
        process::exit(1);
    }

    if opt.p_type != "standard_user" && opt.p_type != "admin" {
        println!("\n{} is not a valid permission type", opt.p_type);
        println!("You must specify a permission type value of 'admin' or 'standard_user':");
        println!("-p PERMISSION-TYPE\n");
        process::exit(1);
    }

    if opt.verbose {
        println!("DynamoDB client version: {}\n", dynamodb::PKG_VERSION);
        println!("Region: {}", opt.region);
        println!("Table:  {}", opt.table);
        println!("User:   {}", opt.username);
        println!("Type:   {}", opt.p_type);
        println!("Age:    {}", opt.age);
        println!("First:  {}", opt.first);
        println!("Last:   {}\n", opt.last);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let config = dynamodb::Config::builder()
        .region(Region::new(opt.region))
        .build();

    let client = dynamodb::Client::from_conf_conn(config, aws_hyper::conn::Standard::https());

    let mut attrs: HashMap<String, AttributeValue> = HashMap::new();
    let user_av = AttributeValue::S(opt.username.to_string());
    let type_av = AttributeValue::S(opt.p_type.to_string());
    let age_av = AttributeValue::S(opt.age.to_string());
    let first_av = AttributeValue::S(opt.first.to_string());
    let last_av = AttributeValue::S(opt.last.to_string());

    attrs.insert("username".to_owned(), user_av);
    attrs.insert("account_type".to_owned(), type_av);
    attrs.insert("age".to_owned(), age_av);
    attrs.insert("first_name".to_owned(), first_av);
    attrs.insert("last_name".to_owned(), last_av);

    match client
        .put_item()
        .table_name(opt.table)
        .item(attrs)
        .send()
        .await
    {
        Ok(_) => println!(
            "Added user {}, {} {}, age {} as {} user",
            opt.username, opt.first, opt.last, opt.age, opt.p_type
        ),
        Err(e) => {
            println!("Got an error adding item:");
            println!("{:?}", e);
            process::exit(1);
        }
    };
}
