/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::{env, process};

use dynamodb::Region;

use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(default_value = "", short, long)]
    region: String,

    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let mut opt = Opt::from_args();

    let mut default_region = match env::var("AWS_DEFAULT_REGION") {
        Ok(val) => val,
        Err(_e) => String::from(""),
    };

    if default_region == "" {
        default_region = String::from("us-west-2");
    }

    if opt.region == "" {
        opt.region = default_region;
    }

    if opt.verbose {
        println!("DynamoDB client version: {}\n", dynamodb::PKG_VERSION);
        println!("Region: {}", opt.region);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let r = &opt.region;

    let config = dynamodb::Config::builder()
        .region(Region::new(String::from(r)))
        .build();

    let client = dynamodb::Client::from_conf_conn(config, aws_hyper::conn::Standard::https());

    match client.list_tables().send().await {
        Ok(resp) => {
            println!("Tables in {}", opt.region);
            let mut l = 0;

            for name in resp.table_names.iter() {
                for n in name.iter() {
                    l = l + 1;
                    println!("    {:?}", n);
                }
            }

            println!("\nFound {} tables in {} region.\n", l, opt.region);
        }
        Err(e) => {
            println!("Got an error listing tables:");
            println!("{:?}", e);
            process::exit(1);
        }
    };
}
