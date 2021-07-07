/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_snowball::{Config, Region, PKG_VERSION};
use aws_types::region;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), aws_sdk_snowball::Error> {
    tracing_subscriber::fmt::init();

    let Opt { region, verbose } = Opt::from_args();

    if verbose {
        println!("Snowball version: {}", PKG_VERSION);
    }

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-east-1"));

    let conf = Config::builder().region(region_provider).build();
    let client = aws_sdk_snowball::Client::from_conf(conf);

    let jobs = client.list_jobs().send().await?;
    for job in jobs.job_list_entries.unwrap() {
        println!("JobId: {:?}", job.job_id);
    }

    Ok(())
}
