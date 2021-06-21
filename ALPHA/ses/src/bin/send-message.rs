/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;

use ses::{Client, Config, Error, Region};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The email address of the sender.
    #[structopt(short, long)]
    from_address: String,

    /// The message of the email.
    #[structopt(short, long)]
    message: String,

    /// The subject of the email.
    #[structopt(short, long)]
    subject: String,

    /// The email address of the recepient.
    #[structopt(short, long)]
    to_address: String,

    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Sends a message to the email address.
/// # Arguments
///
/// * `-f FROM-ADDRESS` - The email address of the sender.
/// * `-m MESSAGE` - The email message that is sent.
/// * `-s SUBJECT` - The subject of the email message.
/// * `-t TO-ADDRESS` - The email address of the recepient.
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let Opt {
        default_region,
        from_address,
        message,
        subject,
        to_address,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!("SES client version: {}", ses::PKG_VERSION);
        println!("Region:             {:?}", &region);
        println!("From address:       {}", &from_address);
        println!("To address:         {}", &to_address);
        println!("Subject:            {}", &subject);
        println!("Message:            {}", &message);
        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

    let new_contact = client
        .send_email()
        .from_email_address(from_address)
        .send()
        .await;
    match new_contact {
        Ok(_) => println!("Created contact"),
        Err(e) => eprintln!("Got error attemptint to create contact: {}", e),
    };

    Ok(())
}
