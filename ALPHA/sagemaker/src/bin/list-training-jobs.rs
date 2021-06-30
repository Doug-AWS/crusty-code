/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use sagemaker::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the your SageMaker jobs in an AWS Region.
/// /// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.#[tokio::main]
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
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
        println!("SageMaker version: {}", PKG_VERSION);
        println!("Region:            {:?}", &region);
        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);
    let job_details = client.list_training_jobs().send().await?;

    println!("Job Name\tCreation DateTime\tDuration\tStatus");
    for j in job_details.training_job_summaries.unwrap_or_default() {
        let name = j.training_job_name.as_deref().unwrap_or_default();
        let creation_time = j.creation_time.unwrap().to_chrono();
        let training_end_time = j.training_end_time.unwrap().to_chrono();

        let status = j.training_job_status.unwrap();
        let duration = training_end_time - creation_time;

        println!(
            "{}\t{}\t{}\t{:#?}",
            name,
            creation_time.format("%Y-%m-%d@%H:%M:%S"),
            duration.num_seconds(),
            status
        );
    }

    Ok(())
}
