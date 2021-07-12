/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::{self, ProvideRegion};
use ec2::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The ID of the snapshot.
    #[structopt(short, long)]
    snapshot_id: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Deletes an EBS snapshot.
/// It must be `completed` before you can use the snapshot.
/// # Arguments
///
/// * `-s SNAPSHOT-ID` - The ID of the snapshot.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        snapshot_id,
        verbose,
    } = Opt::from_args();

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    println!();

    if verbose {
        println!("EC2 version: {}", PKG_VERSION);
        println!(
            "Region:      {}",
            region_provider.region().unwrap().as_ref()
        );
        println!("Snapshot ID: {}", snapshot_id);
        println!();
    }

    let config = Config::builder().region(region_provider).build();
    let client = Client::from_conf(config);

    client
        .delete_snapshot()
        .snapshot_id(snapshot_id)
        .send()
        .await?;

    println!("Deleted");

    Ok(())
}
