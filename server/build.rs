// The web app is embedded by rust-embed; rebuild when `pnpm generate` changes it.
fn main() {
    println!("cargo:rerun-if-changed=../web/.output/public");
}
