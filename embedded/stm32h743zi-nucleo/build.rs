fn main() {
    println!("cargo::rerun-if-changed=build.rs");

	println!("cargo::rustc-link-arg-bins=--nmagic");
	println!("cargo::rustc-link-arg-bins=-Tlink.x");
	println!("cargo::rustc-link-arg-bins=-Tdefmt.x");
}
