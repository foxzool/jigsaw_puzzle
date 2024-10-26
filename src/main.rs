use bevy_jigsaw_puzzle::build_jigsaw_template;

fn main() {
    let img = image::open("images/raw.jpeg").expect("Failed to open image");
    let new_image = build_jigsaw_template(img, 9, 6, None, None, None);
    new_image
        .save("images/jigsaw_template.png")
        .expect("Failed to save image");
}
