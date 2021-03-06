/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_lambda::{Blob, Client, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The AWS Lambda function's Amazon Resource Name (ARN).
    #[structopt(short, long)]
    arn: String,

    /// The file containing JSON provided to the Lambda function as input.
    #[structopt(short, long)]
    payload: String,

    /// Whether to display additional runtime information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Invokes a Lambda function by its ARN.
/// # Arguments
///
/// * `-a ARN` - The ARN of the Lambda function.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let Opt {
        arn,
        payload,
        region,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    if verbose {
        println!("Lambda client version: {}", PKG_VERSION);
        println!(
            "Region:                {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!("Lambda function ARN:   {}", arn);
        println!("Payload:               {}", payload);
        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let blob = std::fs::read(payload).unwrap();

    client
        .invoke()
        .function_name(arn)
        .payload(Blob::new(blob))
        .send()
        .await?;

    println!("Invoked function.");

    Ok(())
}
