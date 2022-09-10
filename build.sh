# GLIBC must be statically linked in order for Stuart to run on Cloudflare Pages.
# This script will build a statically linked binary of Stuart suitable for this use case.

RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu