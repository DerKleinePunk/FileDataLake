use crate::helper::string_ify_ioerror;
use image::{ImageFormat, ImageReader};
use little_exif::u8conversion::U8conversion;
use little_exif::{exif_tag::ExifTag, metadata::Metadata};
use std::{collections::HashMap, path::Path};

//ittle_exif
/*// Read in the metadata again & print it
println!("\nPNG read result:");
for tag in &Metadata::new_from_path(png_path).unwrap()
{
    println!("{:?}", tag);
} */

fn stringify_image(x: image::ImageError) -> String {
    format!("error code: {x}")
}

pub struct ImageSize {
    pub width: u32,
    pub heigth: u32,
}

pub fn make_thumbnail<P: AsRef<Path>>(path: P, save_to: P) -> Result<ImageSize, String> {
    let img = ImageReader::open(path)
        .map_err(string_ify_ioerror)?
        .with_guessed_format()
        .map_err(string_ify_ioerror)?
        .decode()
        .map_err(stringify_image)?;

    //Todo hier geht das Seiten Verhältnis kaput
    let img_tbumbnail = img.thumbnail_exact(128, 128);

    img_tbumbnail
        .save_with_format(save_to, ImageFormat::Jpeg)
        .map_err(stringify_image)?;

    let result = ImageSize {
        width: img.width(),
        heigth: img.height(),
    };

    Ok(result)
}

pub fn read_image_tags(path: &Path) -> Result<HashMap<String, String>, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    let meta_result = &Metadata::new_from_path(&path);
    match meta_result {
        Err(error) => {
            log::error!("{:?}", error);
            return Err("read_image_tags failed".to_string());
        }
        Ok(meta_result_ok) => {
            /*for tag in meta_result_ok {
                println!("{:?}", tag);
                //result.insert(k, v)
            }*/

            let endian = &meta_result_ok.get_endian();
            let image_description_by_tag = meta_result_ok
                .get_tag(&ExifTag::ImageDescription(String::new()))
                .next();
            if image_description_by_tag != None {
                let image_description_string = String::from_u8_vec(
                    &image_description_by_tag.unwrap().value_as_u8_vec(endian),
                    endian,
                );

                println!("{:?}", image_description_string);
                result.insert("ImageDescription".to_string(), image_description_string);
            }

            let image_description_by_tag = meta_result_ok
                .get_tag(&ExifTag::Model(String::new()))
                .next();
            if image_description_by_tag != None {
                let image_description_string =
                    String::from_u8_vec(&image_description_by_tag.unwrap().value_as_u8_vec(endian), endian);
                println!("{:?}", image_description_string);

                result.insert("Model".to_string(), image_description_string);
            }

            let image_description_by_tag = meta_result_ok
                .get_tag(&ExifTag::GPSDestLatitude(Vec::new()))
                .next();
            if image_description_by_tag != None {
                let image_description_string =
                    String::from_u8_vec(&image_description_by_tag.unwrap().value_as_u8_vec(endian), endian);
                println!("{:?}", image_description_string);

            }
        }
    }

    Ok(result)
}
