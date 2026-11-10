//! Embeds the build timestamp so `blazers --version` can report it.

fn main() {
    let now = chrono::Utc::now();
    println!(
        "cargo:rustc-env=BLAZERS_BUILD_TIME={}",
        now.format("%Y.%m.%d.%H.%M.%S")
    );
}
