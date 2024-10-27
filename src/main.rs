mod compress;

use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;
use futures::stream::{self, StreamExt};
use std::fs;
use std::path::Path;
use dotenv::dotenv;
use compress::process_file_from_local;

// Asynchronously upload a file to S3 (or another storage)
pub async fn upload_to_s3(
    filename: &String,
    extension: &String,
    store_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let bucket = "riverbase".to_string();
    let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    // Step 2: Set the custom endpoint (for example, MinIO or another S3-compatible service)
    let custom_endpoint = "https://fsgw.sabay.com"; // Replace this with your custom endpoint

    let config = aws_sdk_s3::config::Builder::from(&aws_config)
        .endpoint_url(custom_endpoint)
        .force_path_style(true)
        .build();

    let s3_client = Client::from_conf(config);

    let file_path = format!("backup/6690a7d28092ed3c326403af/{}", filename);
    let f = &format!("backup/6690a7d28092ed3c326403af/{}", filename);
    println!("file_name {}", file_path);

    let file_name: &str = std::path::Path::new(&f)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();

    let body = tokio::fs::read(file_path.clone()).await?;

    s3_client
        .put_object()
        .bucket(bucket)
        .key(format!("{}/{}", store_id, file_name))
        .content_type(extension)
        .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
        .body(body.into())
        .send()
        .await?;

    Ok(())
}
// Recursively visit directories and upload files
async fn visit_dirs(client: &Client, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if dir.is_dir() {
        let entries = fs::read_dir(dir)?
            .filter_map(Result::ok)
            .filter(|e| e.path().is_file() || e.path().is_dir())
            .collect::<Vec<_>>();

        // Parallel file uploads using async streams
        let upload_futures = stream::iter(entries.into_iter())
            .map(|entry| {
                // let client = client.clone(); // Clone the client to use inside async block
                async move {
                    let path = entry.path();

                    if path.is_dir() {
                        println!("Entering directory: {:?}", path.display());
                        visit_dirs(&client, &path).await // Recursively visit directories
                    } else if path.is_file() {
                        if let Some(file_name) = path.file_name() {
                            println!("Found file: {:?}", file_name);
                            // Convert OsStr to &str
                            if let Some(file_name_str) = file_name.to_str() {
                                // process_file_from_local(&path, file_name_str).await
                                let extension = match path.extension() {
                                    Some(ext) => ext.to_string_lossy().to_string(),
                                    None => panic!(),
                                };
                                upload_to_s3(
                                    &file_name_str.to_string(),
                                    &extension.to_string(),
                                    &"6690a7d28092ed3c326403af".to_string(),
                                )
                                .await // Upload file asynchronously
                            } else {
                                Ok(())
                            }
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }
            })
            .buffer_unordered(10) // Limit concurrent uploads to 10
            .collect::<Vec<_>>()
            .await;

        for result in upload_futures {
            println!("result {:?}", result);
            if let Err(e) = result {
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}

// Main function: setup S3 client and visit directories
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let aws_config = aws_config::defaults(BehaviorVersion::latest()).load().await;
    // Step 2: Set the custom endpoint (for example, MinIO or another S3-compatible service)
    let custom_endpoint = "https://fsgw.sabay.com"; // Replace this with your custom endpoint

    let config = aws_sdk_s3::config::Builder::from(&aws_config)
        .endpoint_url(custom_endpoint)
        .force_path_style(true)
        .build();

    let s3_client = Client::from_conf(config);

    let dir_path = Path::new("./backup/6690a7d28092ed3c326403af");
    visit_dirs(&s3_client, dir_path).await?;

    Ok(())
}
