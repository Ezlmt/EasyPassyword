fn main() {
    println!("Checking lodepng availability...");
    let _ = lodepng::encode32(&[0, 0, 0, 0], 1, 1);
    println!("lodepng is available.");
}
