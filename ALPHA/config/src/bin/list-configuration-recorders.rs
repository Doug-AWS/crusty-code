use aws_types::region;
use aws_types::region::ProvideRegion;
//use config::model::ResourceType;
use config::{Client, Config, Error, Region, PKG_VERSION};
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

/// Lists the AWS Config configuration recorders in the Region.
///
/// # Arguments
///
/// * `[-r REGION]` - The Region in which the client is created.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt { region, verbose } = Opt::from_args();

    let region = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    println!();

    if verbose {
        println!("Config client version: {}", PKG_VERSION);
        println!(
            "Region:                {}",
            region.region().unwrap().as_ref()
        );

        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

    let resp = client.describe_configuration_recorders().send().await?;

    let recorders = resp.configuration_recorders.unwrap_or_default();

    let num_recorders = recorders.len();

    if num_recorders == 0 {
        println!("You have no configuration recorders")
    } else {
        for recorder in recorders {
            println!("Recorder: {}", recorder.name.as_deref().unwrap_or_default());
        }
    }

    println!();

    Ok(())
}
