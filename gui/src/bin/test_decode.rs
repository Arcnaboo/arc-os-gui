use image::GenericImageView;

fn main() {
    let bytes = include_bytes!("../../assets/icons/Terminal.png");
    match image::load_from_memory(bytes) {
        Ok(img) => {
            println!("Success! Dimensions: {:?}", img.dimensions());
        }
        Err(e) => {
            println!("Error decoding PNG: {:?}", e);
        }
    }
}
