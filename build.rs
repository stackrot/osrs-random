fn main() {
    println!("cargo:rerun-if-env-changed=OSRS_RANDOM_RELEASE_TAG");
    println!(
        "cargo:rustc-env=OSRS_RANDOM_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
