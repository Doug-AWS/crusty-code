/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::io::{stdin, /*stdout,*/ Read /*, Write*/};
use std::{iter, process};

use std::{thread, time};

use dynamodb::model::{
    AttributeDefinition, AttributeValue, KeySchemaElement, KeyType, ProvisionedThroughput,
    ScalarAttributeType, Select,
};

use dynamodb::{Client, Config, Region};

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

/// Create a random, n-length string
fn random_string(n: usize) -> String {
    let mut rng = thread_rng();
    iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(n)
        .collect()
}

/// Create a new table. It's remotely possible the random table name exists.
async fn create_table(client: &dynamodb::Client, table: &str, key: &str) {
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

    match client
        .create_table()
        .table_name(table)
        .key_schema(ks)
        .attribute_definitions(ad)
        .provisioned_throughput(pt)
        .send()
        .await
    {
        Ok(_) => println!(""),
        Err(e) => {
            println!("Got an error creating the table:");
            println!("{}", e);
            process::exit(1);
        }
    }
}

/// Add an item to the table.
async fn add_item(
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

    match client
        .put_item()
        .table_name(table)
        .item(key, user_av)
        .item("account_type", type_av)
        .item("age", age_av)
        .item("first_name", first_av)
        .item("last_name", last_av)
        .send()
        .await
    {
        Ok(_) => println!(""),
        Err(e) => {
            println!("Got an error adding item to table:");
            println!("{}", e);
            process::exit(1);
        }
    }
}

/// Scan the table for an item matching the input values.
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

    if !found_match {
        println!("Did not find matching entry in table");
    }
}

/// Delete an item from the table.
async fn delete_item(client: &dynamodb::Client, table: &str, key: &str, value: &str) {
    let user_av = AttributeValue::S(String::from(value));
    match client
        .delete_item()
        .table_name(table)
        .key(key, user_av)
        .send()
        .await
    {
        Ok(_) => println!(""),
        Err(e) => {
            println!("Got an error trying to delete item:");
            println!("{}", e);
            process::exit(1);
        }
    }
}

/// Delete the table.
async fn delete_table(client: &dynamodb::Client, table: &str) {
    match client.delete_table().table_name(table).send().await {
        Ok(_) => println!(""),
        Err(e) => {
            println!("Got an error deleting table:");
            println!("{}", e);
            process::exit(1);
        }
    }
}

/// Wait 1, 2, 4, ... seconds for the table to exist.
async fn wait_for_table(client: &dynamodb::Client, table: &str) {
    let mut delay = 10;
    let mut total = delay;

    println!(
        "Waiting {} seconds for {} table to finish creation",
        delay, table
    );
    println!();

    // Sleep one second by default
    // Wait for delay seconds and try again
    thread::sleep(time::Duration::from_secs(delay));

    loop {
        match client.describe_table().table_name(table).send().await {
            Ok(resp) => {
                match resp.table {
                    None => {
                        println!("Waiting {} second(s)", delay);
                        total += delay;
                        // Wait for delay seconds and try again
                        thread::sleep(time::Duration::from_secs(delay));
                        delay = delay * 2;
                    }
                    Some(t) => {
                        match t.table_name {
                            None => {
                                println!("Waiting {} second(s)", delay);
                                total += delay;
                                // Wait for delay seconds and try again
                                thread::sleep(time::Duration::from_secs(delay));
                                delay = delay * 2;
                            }
                            Some(t) => {
                                if t == table {
                                    println!("Waited {} second(s)", total);
                                    return;
                                } else {
                                    println!("Waiting {} second(s)", delay);
                                    total += delay;
                                    // Wait for delay seconds and try again
                                    thread::sleep(time::Duration::from_secs(delay));
                                    delay = delay * 2;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Got an error listing tables:");
                println!("{}", e);
                process::exit(1);
            }
        }
    }
}

/// Wait for the user to press Enter.
fn pause() {
    //let mut stdout = stdout();
    println!();
    //    stdout.write(b"Press Enter to continue...").unwrap();
    println!("Press Enter to continue");
    //    stdout.flush().unwrap();
    println!();
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

    let r = region.clone();

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    /* Create table */
    println!();
    println!("Creating table {} in {:?}", table, r);
    create_table(&client, &table, &key).await;

    wait_for_table(&client, &table).await;

    if interactive {
        pause();
    }

    println!();
    println!("Adding item to table");

    add_item(
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

    age = "44";

    /* Update the item */
    println!("Modifying table item");

    add_item(
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

    /* Get item and compare it with the one we added */
    println!("Comparing table item to original value");

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

    /* Delete item */
    println!();
    println!("Deleting item");
    delete_item(&client, &table, &key, &value).await;

    if interactive {
        pause();
    }

    /* Delete table */
    println!("Deleting table");
    delete_table(&client, &table).await;
}
