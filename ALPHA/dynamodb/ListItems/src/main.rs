/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::process;

use dynamodb::{Client, Config, Region};

use aws_types::region::{EnvironmentProvider, ProvideRegion};

use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region
    #[structopt(short, long)]
    region: Option<String>,

    #[structopt(short, long)]
    table: String,

    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let Opt {
        table,
        region,
        verbose,
    } = Opt::from_args();

    let region = EnvironmentProvider::new()
        .region()
        .or_else(|| region.as_ref().map(|region| Region::new(region.clone())))
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!("DynamoDB client version: {}\n", dynamodb::PKG_VERSION);
        println!("Region: {:?}", &region);
        println!("Table:  {}", table);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }
    let t = &table;

    let config = Config::builder().region(region).build();

    let client = Client::from_conf_conn(config, aws_hyper::conn::Standard::https());

    match client.scan().table_name(String::from(t)).send().await {
        Ok(resp) => {
            println!("Items in table {}:", table);

            for item in resp.items.iter() {
                for n in item.iter() {
                    println!("   {:?}", n);
                }
            }
        }
        Err(e) => {
            println!("Got an error listing items:");
            println!("{:?}", e);
            process::exit(1);
        }
    };
}
