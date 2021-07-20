use aws_types::region;
use aws_types::region::ProvideRegion;
use config::model::{
    ConfigSnapshotDeliveryProperties, ConfigurationRecorder, DeliveryChannel,
    MaximumExecutionFrequency, RecordingGroup, ResourceType,
};
use config::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The ARN of the Amazon SNS topic.
    #[structopt(short, long)]
    sns_arn: String,

    /// The name of the Amazon bucket.
    #[structopt(short, long)]
    bucket: String,

    /// The ARN of the IAM role.
    #[structopt(short, long)]
    iam_arn: String,

    /// The name of the configuration.
    #[structopt(default_value = "default", short, long)]
    name: String,

    /// The type of resource to record info about.
    #[structopt(default_value = "AWS::DynamoDB::Table", short, long)]
    type_: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Enables AWS Config for a resource type, in the Region.
///
/// # Arguments
///
/// * `-b BUCKET` - The name of the Amazon bucket to which AWS Config delivers configuration snapshots and configuration history files.
/// * `-i IAM-ARN` - The ARN of the IAM role that used to describe the AWS resources associated with the account.
/// * `-s SNS-ARN` - The ARN of the Amazon SNS topic to which AWS Config sends notifications about configuration changes.
/// * `[-t TYPE]` - The type of resource for AWS Config to support.
///   If not supplied, defaults to `AWS::DynamoDB::Table` (DynamoDB tables).
/// * `[-n NAME]` - The name of the configuration.
///   If not supplied, defaults to `default`.
/// * `[-r REGION]` - The Region in which the client is created.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt {
        region,
        bucket,
        iam_arn,
        name,
        sns_arn,
        type_,
        verbose,
    } = Opt::from_args();

    let region = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    println!();

    if verbose {
        println!("Config client version:          {}", PKG_VERSION);
        println!(
            "Region:                         {}",
            region.region().unwrap().as_ref()
        );
        println!("Resource type:                  {}", type_);
        println!("Config (delivery channel) name: {}", name);
        println!("Bucket:                         {}", bucket);
        println!("SNS ARN:                        {}", sns_arn);
        println!("IAM ARN:                        {}", iam_arn);
        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

    if !verbose {
        println!("You won't see any output if you don't have any resources defined in the region.");
    }

    let mut resource_types: Vec<ResourceType> = Vec::new();
    resource_types.push(ResourceType::Topic);

    let rec_group = RecordingGroup::builder()
        .set_resource_types(Some(resource_types))
        .build();

    let cfg_recorder = ConfigurationRecorder::builder()
        .name(&name)
        .role_arn(iam_arn)
        .set_recording_group(Some(rec_group))
        .build();

    client
        .put_configuration_recorder()
        .configuration_recorder(cfg_recorder)
        .send()
        .await?;

    println!("Configured recorder.");

    // put-delivery-channel --delivery-channel file://deliveryChannel.json
    // Create delivery channel
    let snapshot_props = ConfigSnapshotDeliveryProperties::builder()
        .delivery_frequency(MaximumExecutionFrequency::TwelveHours)
        .build();

    let delivery_channel = DeliveryChannel::builder()
        .name(name)
        .s3_bucket_name(bucket)
        .sns_topic_arn(sns_arn)
        .config_snapshot_delivery_properties(snapshot_props)
        .build();

    client
        .put_delivery_channel()
        .delivery_channel(delivery_channel)
        .send()
        .await?;

    println!("Configured delivery channel.");

    Ok(())
}
