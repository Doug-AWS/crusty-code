/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::process;

use kinesis::{Client, Config, Region};

use aws_types::region::{EnvironmentProvider, ProvideRegion};

use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information
    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let Opt { region, verbose } = Opt::from_args();

    let region = EnvironmentProvider::new()
        .region()
        .or_else(|| region.as_ref().map(|region| Region::new(region.clone())))
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!("Kinesis client version: {}\n", kinesis::PKG_VERSION);
        println!("Region:      {:?}", &region);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let config = Config::builder().region(region).build();

    let client = Client::from_conf_conn(config, aws_hyper::conn::Standard::https());

    match client.list_streams().send().await {
        Ok(resp) => match resp.stream_names {
            None => println!("Did not find any streams"),
            Some(names) => {
                let l = names.len();

                println!("Streams:");

                for name in names {
                    println!("    {:?}", name);
                }

                println!("Found {} streams.", l);
            }
        },
        Err(e) => {
            println!("Got an error listing stream names:");
            println!("{:?}", e);
            process::exit(1);
        }
    };
}
