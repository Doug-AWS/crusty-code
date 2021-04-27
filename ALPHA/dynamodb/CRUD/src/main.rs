/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::io::{stdin, stdout, Read, Write};
use std::{iter, process};

use aws_http::AwsErrorRetryPolicy;
use aws_hyper::{SdkError, SdkSuccess};
//use dynamodb::client::fluent_builders::Query;
use dynamodb::error::DescribeTableError;
use dynamodb::input::DescribeTableInput;
use dynamodb::model::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
    ScalarAttributeType, Select, TableStatus,
};

use dynamodb::operation::DescribeTable;
use dynamodb::output::DescribeTableOutput;
use dynamodb::{Client, Config, Region};
//use serde_json::Value;
use smithy_http::operation::Operation;
use smithy_http::retry::ClassifyResponse;
use smithy_types::retry::RetryKind;
// use std::collections::HashMap;
use std::time::Duration;

use aws_types::region::{EnvironmentProvider, ProvideRegion};

use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Whether to run in interactive mode (you have to press return between operations)
    #[structopt(short, long)]
    interactive: bool,

    /// The region
    #[structopt(short, long)]
    region: Option<String>,

    /// Activate verbose mode    
    #[structopt(short, long)]
    verbose: bool,
}

fn random_string(n: usize) -> String {
    let mut rng = thread_rng();
    /*let chars: String = */
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(n)
        .collect() //;

    //    chars
}

/// Hand-written waiter to retry every second until the table is out of `Creating` state
#[derive(Clone)]
struct WaitForReadyTable<R> {
    inner: R,
}

impl<R> ClassifyResponse<SdkSuccess<DescribeTableOutput>, SdkError<DescribeTableError>>
    for WaitForReadyTable<R>
where
    R: ClassifyResponse<SdkSuccess<DescribeTableOutput>, SdkError<DescribeTableError>>,
{
    fn classify(
        &self,
        response: Result<&SdkSuccess<DescribeTableOutput>, &SdkError<DescribeTableError>>,
    ) -> RetryKind {
        match self.inner.classify(response) {
            RetryKind::NotRetryable => (),
            other => return other,
        };
        match response {
            Ok(SdkSuccess { parsed, .. }) => {
                if parsed
                    .table
                    .as_ref()
                    .unwrap()
                    .table_status
                    .as_ref()
                    .unwrap()
                    == &TableStatus::Creating
                {
                    RetryKind::Explicit(Duration::from_secs(1))
                } else {
                    RetryKind::NotRetryable
                }
            }
            _ => RetryKind::NotRetryable,
        }
    }
}

fn create_table(client: &dynamodb::Client, table: &str, key: &str) {
    println!("Creating table");
    let ad = AttributeDefinition::builder()
        .attribute_name(key)
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(key)
        .key_type(KeyType::Hash)
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    client
        .create_table()
        .table_name(table)
        .key_schema(ks)
        .attribute_definitions(ad)
        .provisioned_throughput(pt);
}

fn add_item(
    client: &dynamodb::Client,
    table: &str,
    key: &str,
    value: &str,
    first_name: &str,
    last_name: &str,
    age: &str,
    utype: &str,
) {
    println!("Adding item to table");

    let user_av = AttributeValue::S(String::from(value));
    let type_av = AttributeValue::S(String::from(utype));
    let age_av = AttributeValue::S(String::from(age));
    let first_av = AttributeValue::S(String::from(first_name));
    let last_av = AttributeValue::S(String::from(last_name));

    client
        .put_item()
        .table_name(table)
        .item(key, user_av)
        .item("account_type", type_av)
        .item("age", age_av)
        .item("first_name", first_av)
        .item("last_name", last_av);
}

async fn scan(
    client: &dynamodb::Client,
    table: &str,
    key: &str,
    value: &str,
    first_name: &str,
    last_name: &str,
    age: &str,
    utype: &str,
) {
    let user_av = AttributeValue::S(String::from(value));
    let type_av = AttributeValue::S(String::from(utype));
    let age_av = AttributeValue::S(String::from(age));
    let first_av = AttributeValue::S(String::from(first_name));
    let last_av = AttributeValue::S(String::from(last_name));

    let mut found_match = true;

    let resp = client
        .scan()
        .table_name(table)
        .select(Select::AllAttributes)
        .send()
        .await;

    match resp {
        Ok(r) => {
            let items = r.items.unwrap_or_default();
            for item in items {
                // Do key values match?
                match item.get(&String::from(key)) {
                    None => found_match = false,
                    Some(v) => {
                        if v != &user_av {
                            found_match = false;
                        }
                    }
                }

                // Do age values match?
                match item.get(&String::from("age")) {
                    None => found_match = false,
                    Some(v) => {
                        if v != &age_av {
                            found_match = false;
                        }
                    }
                }

                // Do first name values match?
                match item.get(&String::from("first_name")) {
                    None => found_match = false,
                    Some(v) => {
                        if v != &first_av {
                            found_match = false;
                        }
                    }
                }

                // Do last name values match?
                match item.get(&String::from("last_name")) {
                    None => found_match = false,
                    Some(v) => {
                        if v != &last_av {
                            found_match = false;
                        }
                    }
                }

                // Do account type values match?
                match item.get(&String::from("account_type")) {
                    None => found_match = false,
                    Some(v) => {
                        if v != &type_av {
                            found_match = false;
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Got an error scanning the table:");
            println!("{}", e);
            process::exit(1);
        }
    };

    if found_match {
        println!("Found matching entry in table");
    } else {
        println!("Did not find matching entry in table");
    }
}

fn delete_item(client: &dynamodb::Client, table: &str, key: &str, value: &str) {
    println!("Deleting item from table");

    let user_av = AttributeValue::S(String::from(value));
    client.delete_item().table_name(table).key(key, user_av);
}

fn delete_table(client: &dynamodb::Client, table: &str) {
    println!("Deleting table");

    client.delete_table().table_name(table);
}

fn wait_for_ready_table(
    table_name: &str,
    conf: &Config,
) -> Operation<DescribeTable, WaitForReadyTable<AwsErrorRetryPolicy>> {
    let operation = DescribeTableInput::builder()
        .table_name(table_name)
        .build(&conf)
        .expect("valid input");
    let waiting_policy = WaitForReadyTable {
        inner: operation.retry_policy().clone(),
    };
    operation.with_retry_policy(waiting_policy)
}

fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

#[tokio::main]
async fn main() {
    let Opt {
        interactive,
        region,
        verbose,
    } = Opt::from_args();

    let region = EnvironmentProvider::new()
        .region()
        .or_else(|| region.as_ref().map(|region| Region::new(region.clone())))
        .unwrap_or_else(|| Region::new("us-west-2"));

    // Create 10-charater random table name
    let table = random_string(10);

    // Create a 6-character random key name
    let key = random_string(6);

    // Create a 12-character random key value
    let value = random_string(12);

    // Specify first name, last name, age, and type
    let first_name = "DummyFirstName";
    let last_name = "DummyLastName";
    let mut age = "33";
    let utype = "standard_user";

    if verbose {
        println!("DynamoDB client version: {}\n", dynamodb::PKG_VERSION);
        println!("Table:  {}", table);
        println!("Key:    {}\n", key);
        println!("Value:  {}", value);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    /* Create table */
    create_table(&client, &table, &key);

    if interactive {
        pause();
    }
    /*
    println!("Creating table");
    let ad = AttributeDefinition::builder()
        .attribute_name(key.clone())
        .attribute_type(ScalarAttributeType::S)
        .build();

    let ks = KeySchemaElement::builder()
        .attribute_name(key.clone())
        .key_type(KeyType::Hash)
        .build();

    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(10)
        .write_capacity_units(5)
        .build();

    match client
        .create_table()
        .table_name(table.clone())
        .key_schema(ks)
        .attribute_definitions(ad)
        .provisioned_throughput(pt)
        .send()
        .await
    {
        Ok(_) => println!("Created table {} with key {}", table, key),
        Err(e) => {
            println!("Got an error creating table:");
            println!("{}", e);
            process::exit(1);
        }
    };
    */

    /* Wait for table to be created */
    println!("Waiting for table to be ready to use");
    let raw_client = aws_hyper::Client::https();
    raw_client
        .call(wait_for_ready_table(&table, client.conf()))
        .await
        .expect("table should become ready");

    /* Add an item to the table */
    /*
    println!("Adding an item to the table");
    let user_av = AttributeValue::S(value.clone());
    let type_av = AttributeValue::S(String::from(utype));
    let age_av = AttributeValue::S(String::from(age));
    let first_av = AttributeValue::S(String::from(first_name));
    let last_av = AttributeValue::S(String::from(last_name));

    match client
        .put_item()
        .table_name(table.clone())
        .item(key.clone(), user_av)
        .item("account_type", type_av)
        .item("age", age_av)
        .item("first_name", first_av)
        .item("last_name", last_av)
        .send()
        .await
    {
        Ok(_) => println!(
            "Added user {}, {} {}, age {} as standard_user user",
            value, first_name, last_name, age
        ),
        Err(e) => {
            println!("Got an error adding item:");
            println!("{}", e);
            process::exit(1);
        }
    };
    */
    add_item(
        &client,
        &table,
        &key,
        &value,
        &first_name,
        &last_name,
        &age,
        &utype,
    );

    if interactive {
        pause();
    }

    age = "44";

    /* Update the item */
    add_item(
        &client,
        &table,
        &key,
        &value,
        &first_name,
        &last_name,
        &age,
        &utype,
    );

    if interactive {
        pause();
    }
    /*
    println!("Updating the item's age to 44");
    age = "44";
    let user_av = AttributeValue::S(value.clone());
    let type_av = AttributeValue::S(String::from(utype));
    let age_av = AttributeValue::S(String::from(age));
    let first_av = AttributeValue::S(String::from(first_name));
    let last_av = AttributeValue::S(String::from(last_name));

    match client
        .put_item()
        .table_name(table.clone())
        .item(key.clone(), user_av)
        .item("account_type", type_av)
        .item("age", age_av)
        .item("first_name", first_av)
        .item("last_name", last_av)
        .send()
        .await
    {
        Ok(_) => println!(
            "Updated user {}, {} {}, age {} as standard_user user",
            value, first_name, last_name, age
        ),
        Err(e) => {
            println!("Got an error adding item:");
            println!("{}", e);
            process::exit(1);
        }
    };
    */

    /* Get item and compare it with the one we added */
    println!("Getting the item from the table");

    scan(
        &client,
        &table,
        &key,
        &value,
        &first_name,
        &last_name,
        &age,
        &utype,
    )
    .await;

    if interactive {
        pause();
    }
    /*
    match client.scan().table_name(table.clone()).send().await {
        Ok(_) => println!(
            "Updated user {}, {} {}, age {} as standard_user user",
            value, first_name, last_name, age
        ),
        Err(e) => {
            println!("Got an error adding item:");
            println!("{}", e);
            process::exit(1);
        }
    };
    */

    /* Delete item */
    delete_item(&client, &table, &key, &value);

    if interactive {
        pause();
    }

    /* Delete table */
    delete_table(&client, &table);
}
