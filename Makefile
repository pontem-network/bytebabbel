pre_commit:
	cargo fmt
	cargo build
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test -- --nocapture

clean:
	cargo clean

release_build_mac:
	cargo build --release --target aarch64-apple-darwin --features "deploy"
	cargo build --release --target x86_64-apple-darwin --features "deploy"
	cd target/aarch64-apple-darwin/release; zip e2m-MacOSX-aarch64.zip e2m && cp e2m-MacOSX-aarch64.zip ../../.
	cd target/x86_64-apple-darwin/release; zip e2m-MacOSX-x86_64.zip e2m && cp e2m-MacOSX-x86_64.zip ../../.

release_build_linux:
	cargo build --release --target x86_64-unknown-linux-gnu --features "deploy"
	cargo build --release --target aarch64-unknown-linux-gnu --features "deploy"
	cargo build --release --target i686-unknown-linux-gnu --features "deploy"
	cd target/x86_64-unknown-linux-gnu/release; zip e2m-Linux-x86_64.zip e2m && cp e2m-Linux-x86_64.zip ../../.
	cd target/aarch64-unknown-linux-gnu/release; zip e2m-Linux-aarch64.zip e2m && cp e2m-Linux-aarch64.zip ../../.
	cd target/i686-unknown-linux-gnu/release; zip e2m-Linux-i686.zip e2m && cp e2m-Linux-i686.zip ../../.
