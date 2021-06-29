/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use dynamodb::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    #[structopt(short, long)]
    table: String,

    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the items in a DynamoDB table.
/// # Arguments
///
/// * `-t TABLE` - The name of the table.
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        table,
        default_region,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!();

    if verbose {
        println!("DynamoDB version: {}", PKG_VERSION);
        println!("Region:           {:?}", &region);
        println!("Table:            {}", table);
        println!();
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    let t = &table;

    let resp = client
        .scan()
        .table_name(t)
        .send()
        .await
        .expect("Could not retrieve items");

    println!("Items in table {}:", table);

    let items = resp.items.unwrap_or_default();

    for item in items {
        println!("   {:?}", item);
    }

    Ok(())
}
