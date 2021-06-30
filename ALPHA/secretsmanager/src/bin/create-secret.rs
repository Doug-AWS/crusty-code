/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use secretsmanager::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The name of the secret.
    #[structopt(short, long)]
    name: String,

    /// The value of the secret.
    #[structopt(short, long)]
    secret_value: String,

    /// Whether to display additonal information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Creates a secret.
/// # Arguments
///
/// * `-n NAME` - The name of the secret.
/// * `-s SECRET_VALUE` - The secret value.
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt {
        name,
        default_region,
        secret_value,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!();

    if verbose {
        println!("SecretsManager version: {}", PKG_VERSION);
        println!("Region:                 {:?}", &region);
        println!("Secret name:            {}", &name);
        println!("Secret value:           {}", &secret_value);
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    client
        .create_secret()
        .name(name)
        .secret_string(secret_value)
        .send()
        .await?;
    println!("Created secret");

    Ok(())
}
