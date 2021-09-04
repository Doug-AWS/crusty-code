extern crate exif;

use aws_sdk_dynamodb::model::AttributeValue;
use aws_types::region;
use aws_types::region::ProvideRegion;
use std::process;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The S3 bucket.
    #[structopt(short, long)]
    bucket: String,

    /// The filename.
    #[structopt(short, long)]
    filename: String,

    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The DynamoDB table.
    #[structopt(short, long)]
    table: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

#[derive(Clone, Debug)]
struct Litem {
    name: String,
    confidence: f32,
}

#[derive(Debug)]
struct Edata {
    height: String,
    width: String,
    created: String,
}

#[derive(Debug)]
struct Ldata {
    label1: String,
    value1: String,
    label2: String,
    value2: String,
    label3: String,
    value3: String,
}

async fn add_file_to_bucket(client: &aws_sdk_s3::Client, bucket: &str, filename: &str) {
    let body = aws_sdk_s3::ByteStream::from_path(std::path::Path::new(filename)).await;

    match body {
        Ok(b) => {
            match client
                .put_object()
                .bucket(bucket)
                .key(filename)
                .body(b)
                .send()
                .await
            {
                Err(e) => {
                    println!("Got an error uploading file to bucket:");
                    println!("{}", e);
                    process::exit(1);
                }
                Ok(_) => {
                    println!("Uploaded file to bucket.");
                }
            }
        }
        Err(e) => {
            println!("Got an error uploading file to bucket:");
            println!("{}", e);
            process::exit(1);
        }
    }
}

fn get_exif_data(filename: &str) -> Edata {
    let height: String = "".to_owned();
    let width: String = "".to_owned();
    let created: String = "".to_owned();
    let mut edata = Edata {
        height,
        width,
        created,
    };

    let file = std::fs::File::open(&filename).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();

    match exifreader.read_from_container(&mut bufreader) {
        Ok(exif) => {
            println!("{}", &filename);

            for f in exif.fields() {
                // Get EXIF values for image width, height, and when image was created.
                match &*f.tag.to_string() {
                    "ImageWidth" => edata.height.push_str(&*f.display_value().to_string()),
                    "ImageLength" => edata.width.push_str(&*f.display_value().to_string()),
                    "DateTimeOriginal" => edata.created.push_str(&*f.display_value().to_string()),
                    _ => {}
                }
            }
        }
        Err(_) => {
            println!();
            println!("File does not contain ELIF data");
        }
    };

    edata
}

async fn add_data_to_table(
    verbose: bool,
    client: &aws_sdk_dynamodb::Client,
    table: &str,
    filename: &str,
    edata: Edata,
    labels: Vec<Litem>,
) {
    if verbose {
        println!("Added ")
    }
    let filename_string = AttributeValue::S(filename.to_string());
    let height_string = AttributeValue::S(edata.height);
    let created_string = AttributeValue::S(edata.created);
    let width_string = AttributeValue::S(edata.width);
    let label1_label = AttributeValue::S(labels[0].name.to_string());
    let label1_value = AttributeValue::S(labels[0].confidence.to_string());
    let label2_label = AttributeValue::S(labels[1].name.to_string());
    let label2_value = AttributeValue::S(labels[1].confidence.to_string());
    let label3_label = AttributeValue::S(labels[2].name.to_string());
    let label3_value = AttributeValue::S(labels[2].confidence.to_string());

    match client
        .put_item()
        .table_name(table)
        .item("filename", filename_string) // Table key.
        .item("height", height_string)
        .item("width", width_string)
        .item("created", created_string)
        .item("Label1", label1_label)
        .item("Value1", label1_value)
        .item("Label2", label2_label)
        .item("Value2", label2_value)
        .item("Label3", label3_label)
        .item("Value3", label3_value)
        .send()
        .await
    {
        Err(e) => {
            println!("Got an error adding data to table:");
            println!("{}", e);
            process::exit(1);
        }
        Ok(_) => {
            println!("Added info to table.");
        }
    }
}

async fn get_label_data(
    rekog_client: &aws_sdk_rekognition::Client,
    bucket: &str,
    key: &str,
) -> Vec<Litem> {
    let s3_obj = aws_sdk_rekognition::model::S3Object::builder()
        .bucket(bucket)
        .name(key)
        .build();

    let s3_img = aws_sdk_rekognition::model::Image::builder()
        .s3_object(s3_obj)
        .build();

    let resp = rekog_client.detect_labels().image(s3_img).send().await;

    let labels = resp.unwrap().labels.unwrap_or_default();

    // Create vector of Labels.
    let mut label_vec: Vec<Litem> = vec![];

    for label in labels {
        let name = label.name.as_deref().unwrap_or_default();
        let confidence = label.confidence.unwrap();

        let label = Litem {
            name: name.to_string(),
            confidence,
        };
        label_vec.push(label);
    }

    // Sort label items by confidence.
    label_vec.sort_by(|b, a| a.confidence.partial_cmp(&b.confidence).unwrap());

    // Return the first three items.
    label_vec[0..3].to_vec()
}

#[tokio::main]
async fn main() -> Result<(), exif::Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        bucket,
        filename,
        region,
        table,
        verbose,
    } = Opt::from_args();

    let s3_provider = region::ChainProvider::first_try(region.clone().map(aws_sdk_s3::Region::new))
        .or_default_provider()
        .or_else(aws_sdk_s3::Region::new("us-west-2"));

    let dynamo_provider =
        region::ChainProvider::first_try(region.clone().map(aws_sdk_dynamodb::Region::new))
            .or_default_provider()
            .or_else(aws_sdk_dynamodb::Region::new("us-west-2"));

    let rekognition_provider =
        region::ChainProvider::first_try(region.clone().map(aws_sdk_rekognition::Region::new))
            .or_default_provider()
            .or_else(aws_sdk_rekognition::Region::new("us-west-2"));

    println!();

    if verbose {
        println!(
            "DynamoDB client version:    {}",
            aws_sdk_dynamodb::PKG_VERSION
        );
        println!(
            "Rekognition client version: {}",
            aws_sdk_dynamodb::PKG_VERSION
        );
        println!(
            "Region:                     {}",
            dynamo_provider.region().unwrap().as_ref()
        );
        println!("Filename:                   {}", &filename);
        println!("Bucket:                     {}", &bucket);
        println!("Table:                      {}", &table);
        println!();
    }

    let s3_conf = aws_sdk_s3::Config::builder().region(s3_provider).build();
    let s3_client = aws_sdk_s3::Client::from_conf(s3_conf);

    add_file_to_bucket(&s3_client, &bucket, &filename).await;

    let dynamo_conf = aws_sdk_dynamodb::Config::builder()
        .region(dynamo_provider)
        .build();
    let dynamo_client = aws_sdk_dynamodb::Client::from_conf(dynamo_conf);

    // Pass String values to the function as &str values.

    /*
      From https://doc.rust-lang.org/rust-by-example/std/str.html:
      A String is stored as a vector of bytes (Vec<u8>),
      but guaranteed to always be a valid UTF-8 sequence.
      String is heap allocated, growable and not null terminated.

      &str is a slice (&[u8]) that always points to a valid UTF-8 sequence,
      and can be used to view into a String, just like &[T] is a view into Vec<T>.
    */

    let edata = get_exif_data(&filename);

    /*
    let h: &str = &*height; // You can also use just &height
    let w: &str = &*width; // as the compiler knows how many
    let t: &str = &*time; // *'s to insert to satisfy the type (&str).
    */

    //add_exif_data_to_table(verbose, &dynamo_client, &table, &filename, edata).await;

    let rekog_conf = aws_sdk_rekognition::Config::builder()
        .region(rekognition_provider)
        .build();
    let rekog_client = aws_sdk_rekognition::Client::from_conf(rekog_conf);

    let labels = get_label_data(&rekog_client, &bucket, &filename).await;

    // Add data to table.
    add_data_to_table(verbose, &dynamo_client, &table, &filename, edata, labels).await;
    /*
        async fn add_data_to_table(
        verbose: bool,
        client: &aws_sdk_dynamodb::Client,
        table: &str,
        filename: &str,
        edata: Edata,
        labels: Labels,
    )
    */

    Ok(())
}
