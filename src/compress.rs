use image::ImageReader;
use mime_guess::from_path;
use std::fs::{self, File};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

fn remove_extension(file_name: &str) -> String {
    let path = Path::new(file_name);
    match path.file_stem() {
        Some(stem) => stem.to_string_lossy().into_owned(),
        None => file_name.to_string(), // Return the original name if no stem found
    }
}

// Example function to process a file from a local folder
pub async fn process_file_from_local(
    local_file_path: &Path,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // let file_type = format!("");
    let extension = match local_file_path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => panic!(),
    };
    let file_type = from_path(local_file_path)
        .first_or_octet_stream()
        .to_string();

    let name = remove_extension(file_name);
    // Define paths for max and min files
    let destination_max = format!("./output-test/{}_max.{}", name, extension);
    let destination_min = format!("./output-test/{}.{}", name, extension);

    // Check if the local file exists
    let local_file = Path::new(local_file_path);
    if !local_file.exists() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found",
        )));
    }

    // Open the local file
    // let temp_file = File::open(&local_file)?;

    // Compress the file
    compress_file(
        local_file.to_path_buf(),
        &file_type,
        &extension,
        &destination_max,
        &destination_min,
    )?;

    Ok(())
}

// Function for compressing the file
fn compress_file(
    local_file: PathBuf,
    // temp_file: File,
    file_type: &str,
    extension: &str,
    destination_max: &str,
    destination_min: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the "max" file (larger or higher quality version)
    let mut buffer_max = Cursor::new(Vec::new());

    // Process the file based on its type
    if file_type.starts_with("image/") {
        // Decode the image from the file
        println!("filename {}", file_type);
        println!("destination_max {}", destination_max);

        // Attempt to decode the image from the file
        let image = match ImageReader::open(&local_file)
            .and_then(|reader| Ok(reader.decode()))
            .unwrap()
        {
            Ok(img) => img,
            Err(e) => {
                eprintln!("Error decoding image: {}", e);
                return Ok(()); // Continue to the next work, don't stop the process
            }
        };

        // Resize image to "max" size (e.g., 80% of original size)
        let max_width = (image.width() as f32 * 0.5) as u32;
        let max_height = (image.height() as f32 * 0.5) as u32;
        let resized_image_max =
            image.resize(max_width, max_height, image::imageops::FilterType::Lanczos3);

        // Write "max" version based on the file extension
        write_resized_image(&resized_image_max, &mut buffer_max, extension)?;

        let mut dest_file_max = File::create(&destination_max)?;
        dest_file_max.write_all(buffer_max.get_ref())?;

        // Create the "min" file (smaller or lower quality version)
        let mut buffer_min = Cursor::new(Vec::new());
        let min_width = (800 as f32) as u32;
        let min_height = (800 as f32) as u32;
        let resized_image_min =
            image.resize(min_width, min_height, image::imageops::FilterType::Lanczos3);

        // Write "min" version based on the file extension
        write_resized_image(&resized_image_min, &mut buffer_min, extension)?;

        let mut dest_file_min = File::create(&destination_min)?;
        dest_file_min.write_all(buffer_min.get_ref())?;
    } else if file_type.starts_with("video/") {
        // Move the video to the destination without further processing
        fs::rename(&local_file, &destination_max)?;
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unsupported file format",
        )));
    }

    Ok(())
}

// Helper function to write resized image to buffer
fn write_resized_image(
    image: &image::DynamicImage,
    buffer: &mut Cursor<Vec<u8>>,
    extension: &str,
) -> Result<(), image::ImageError> {
    match extension {
        "jpg" | "jpeg" => image.write_to(buffer, image::ImageFormat::Jpeg),
        "png" => image.write_to(buffer, image::ImageFormat::Png),
        "webp" => image.write_to(buffer, image::ImageFormat::WebP),
        "gif" => image.write_to(buffer, image::ImageFormat::Gif),
        _ => panic!(),
    }
}
