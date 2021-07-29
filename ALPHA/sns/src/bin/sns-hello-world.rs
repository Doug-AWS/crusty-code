/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
use aws_sdk_sns::{Client, Config, Error, Region, PKG_VERSION};
use aws_types::region;
use aws_types::region::ProvideRegion;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Specifies the email address to subscribe to the topic.
    #[structopt(short, long)]
    email_address: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Subscribes an email address and publishes a message to a topic.
/// If the email address has not been confirmed for the topic,
/// a confirmation request is also sent to the email address.
/// # Arguments
///
/// * `-e EMAIL_ADDRESS` - The email address of a user subscribing to the topic.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        email_address,
        verbose,
    } = Opt::from_args();

    let region = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    println!();

    if verbose {
        println!("SNS client version:   {}", PKG_VERSION);
        println!(
            "Region:               {}",
            region.region().unwrap().as_ref()
        );
        println!("Email address:        {}", &email_address);
        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

    let topics = client.list_topics().send().await?;
    let mut topics = topics.topics.unwrap_or_default();
    let topic_arn = &topics.pop().unwrap().topic_arn;
    let topic_string = topic_arn.as_deref().unwrap_or_default();

    println!("Receiving on topic with ARN: `{}`", topic_string);

    let rsp = client
        .subscribe()
        .topic_arn(topic_string)
        .protocol("email")
        .endpoint(email_address)
        .send()
        .await?;

    println!("Added a subscription: {:?}", rsp);

    let rsp = client
        .publish()
        .topic_arn(topic_string)
        .message("hello sns!")
        .send()
        .await?;

    println!("Published message: {:?}", rsp);

    Ok(())
}
