pre_commit:
	cargo fmt
	cargo +nightly build
	cargo +nightly clippy -- -D warnings

clean:
	cargo clean
	rm -rf ./tests/assets/bin
	rm -rf ./tests/assets/move/build

rebuild: clean
	cargo +nightly build
