    //Todo https://docs.rs/image/latest/image/index.html
    //pub fn thumbnail_exact(&self, nwidth: u32, nheight: u32) -> DynamicImage

    use std::path::Path;

    use image::{ImageFormat, ImageReader};


    //ittle_exif
    /*// Read in the metadata again & print it
	println!("\nPNG read result:");
	for tag in &Metadata::new_from_path(png_path).unwrap()
	{
		println!("{:?}", tag);
	} */

fn stringify(x: std::io::Error) -> String { format!("error code: {x}") }
fn stringify_image(x: image::ImageError) -> String { format!("error code: {x}") }

pub struct ImageSize{
    pub width : u32,
    pub heigth : u32
}

pub fn make_thumbnail<P: AsRef<Path>>(path: P, save_to: P) -> Result<ImageSize,String> {

    let img = ImageReader::open(path)
        .map_err(stringify)?
        .with_guessed_format().map_err(stringify)?
        .decode().map_err(stringify_image)?;

    //Todo hier geht das Seiten Verhältnis kaput
    let img_tbumbnail = img.thumbnail_exact(128,128);

    img_tbumbnail.save_with_format(save_to, ImageFormat::Jpeg).map_err(stringify_image)?;

    let result = ImageSize {
        width : img.width(),
        heigth: img.height()
    };

    Ok(result)
}
