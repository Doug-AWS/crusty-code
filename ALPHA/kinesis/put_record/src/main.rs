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

    #[structopt(short, long)]
    data: String,

    #[structopt(short, long)]
    key: String,

    #[structopt(short, long)]
    name: String,

    #[structopt(short, long)]
    verbose: bool,
}

/// Adds a record to an Amazon Kinesis data stream.
/// [Duck Duck Go](https://duckduckgo.com)
/// See [PutRecord] (https://docs.aws.amazon.com/kinesis/latest/APIReference/API_PutRecord.html)
/// in the Amazon Kinesis Streams API reference for further information.
/// # Arguments
///
/// * `-d DATA` - The data (text) to put into the record.
/// * `-k KEY` - The shard in the stream to which the data record is assigned.
/// * `-n NAME` - The name of the stream.
/// * `[-r REGION]` - The region containing the stream.
///   If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    let Opt {
        data,
        key,
        name,
        region,
        verbose,
    } = Opt::from_args();

    let region = EnvironmentProvider::new()
        .region()
        .or_else(|| region.as_ref().map(|region| Region::new(region.clone())))
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!("Kinesis client version: {}\n", kinesis::PKG_VERSION);
        println!("Region:      {:?}", &region);
        println!("Data:");
        println!("\n{}\n", data);
        println!("Partition key: {}", key);
        println!("Stream name:   {}", name);

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let config = Config::builder().region(region).build();

    let client = Client::from_conf(config);

    // blob is a Base64-encoded binary data object
    //    let data = base64::encode(data);
    let blob = kinesis::Blob::new(data);

    match client
        .put_record()
        .data(blob)
        .partition_key(key)
        .stream_name(name)
        .send()
        .await
    {
        Ok(_) => println!("Put data into stream."),
        Err(e) => {
            println!("Got an error putting record:");
            println!("{}", e);
            process::exit(1);
        }
    };
}
