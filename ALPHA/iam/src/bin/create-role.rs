use aws_types::region;
use aws_types::region::ProvideRegion;
//use iam::model::ResourceType;
use iam::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
  /// The AWS Region.
  #[structopt(short, long)]
  region: Option<String>,

  /// Your account ID.
  #[structopt(short, long)]
  account: String,

  /// The name of the bucket
  #[structopt(short, long)]
  bucket: String,

  /// The name of the role.
  #[structopt(short, long)]
  name: String,

  /// Whether to display additional information.
  #[structopt(short, long)]
  verbose: bool,
}

/// Creates an IAM role so AWS Config can access an S3 bucket.
///
/// # Arguments
///
/// * `-a ACCOUNT-ID` - Your account ID.
/// * `-b BUCKET` - The name of the bucket where Config stores information about resources.
/// * `-n NAME` - The name of the role.
/// * `[-r REGION]` - The Region in which the client is created.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display information.
#[tokio::main]
async fn main() -> Result<(), Error> {
  tracing_subscriber::fmt::init();
  let Opt {
    account,
    bucket,
    name,
    region,
    verbose,
  } = Opt::from_args();

  let region = region::ChainProvider::first_try(region.map(Region::new))
    .or_default_provider()
    .or_else(Region::new("us-west-2"));

  println!();

  // let doc = String::from("{\"Version\": \"2012-10-17\",\"Statement\": [{\"Sid\": \"AWSConfigBucketPermissionsCheck\",\"Effect\": \"Allow\",\"Principal\": {\"Service\": \"config.amazonaws.com\"},\"Action\": \"s3:GetBucketAcl\",\"Resource\": \"arn:aws:s3:::") + &bucket + "\"},{\"Sid\": \"AWSConfigBucketExistenceCheck\",\"Effect\": \"Allow\",\"Principal\": {\"Service\": \"config.amazonaws.com\"},\"Action\": \"s3:ListBucket\",\"Resource\": \"arn:aws:s3:::" + &bucket + "\"},{\"Sid\": \"AWSConfigBucketDelivery\",\"Effect\": \"Allow\",\"Principal\": {\"Service\": \"config.amazonaws.com\"},\"Action\": \"s3:PutObject\",\"Resource\": \"arn:aws:s3:::" + &bucket + "/AWSLogs/" + &account + "/Config/*\",\"Condition\": {\"StringEquals\": {\"s3:x-amz-acl\": \"bucket-owner-full-control\"}}}]}";

  let doc = "{
      \"Version\":\"2012-10-17\",
      \"Statement\":[
        {
          \"Effect\":\"Allow\",
          \"Principal\":{
            \"Service\":[\"config.amazonaws.com\"]
          },
          \"Action\":[\"sts:AssumeRole\"]
        }
        ]
      }";

  if verbose {
    println!("IAM client version: {}", PKG_VERSION);
    println!("Region:             {}", region.region().unwrap().as_ref());
    println!("Account ID:         {}", &account);
    println!("Bucket:             {}", &bucket);
    println!("Role name:          {}", &name);
    println!("Policy doc:");
    println!();
    println!("{}", doc);
    println!();

    println!();
  }

  let conf = Config::builder().region(region).build();
  let client = Client::from_conf(conf);

  let resp = client
    .create_role()
    .assume_role_policy_document(doc)
    .role_name(name)
    .send()
    .await?;

  println!("Created role with ARN {}", resp.role.unwrap().arn.unwrap());

  Ok(())
}
