fn main() {
    #[cfg(feature = "input")]
    println!("cargo::rerun-if-changed=../input");

    #[cfg(feature = "input")]
    aoc::get_input_info_from_cargo(Some("../input".to_string()));
}
