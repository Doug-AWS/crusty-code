/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use ssm::model::ParameterType;
use ssm::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The parameter name.
    #[structopt(short, long)]
    name: String,

    /// The parameter value.
    #[structopt(short, long)]
    parameter_value: String,

    /// The parameter title (description).
    #[structopt(short, long)]
    title: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Creates a new AWS Systems Manager parameter.
/// # Arguments
///
/// * `-n NAME` - The name of the parameter.
/// * `-p PARAMETER_VALUE` - The value of the parameter.
/// * `-t TITLE` - The description of the parameter.
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt {
        name,
        parameter_value,
        title,
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
        println!("SSM version:          {}", PKG_VERSION);
        println!("Region:               {:?}", &region);
        println!("Parameter name:       {}", &name);
        println!("Paramter value:       {}", &parameter_value);
        println!("Paramter description: {}", &title);
        println!();
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    let resp = client
        .put_parameter()
        .overwrite(true)
        .r#type(ParameterType::String)
        .name(name)
        .value(parameter_value)
        .description(title)
        .send()
        .await?;

    println!("Success! Parameter now has version: {}", resp.version);

    Ok(())
}
