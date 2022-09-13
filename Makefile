pre_commit:
	cargo fmt
	cargo +nightly build
	cargo +nightly clippy -- -D warnings
	cargo +nightly test

clean:
	cargo clean

release_build_mac:
	cargo build --release --target aarch64-apple-darwin --features "deploy"
	cargo build --release --target x86_64-apple-darwin --features "deploy"

release_build_linux:
	cargo build --release --target x86_64-unknown-linux-gnu --features "deploy"
	cargo build --release --target aarch64-unknown-linux-gnu --features "deploy"
	cargo build --release --target i686-unknown-linux-gnu --features "deploy"