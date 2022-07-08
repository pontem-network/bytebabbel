pre_commit:
	cargo fmt
	cargo build
	cargo clippy

clean:
	cargo clean
	rm -rf ./tests/assets/bin
	rm -rf ./tests/assets/move/build

rebuild: clean
	cargo +nightly build
