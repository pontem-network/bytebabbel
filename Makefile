pre_commit:
	cargo fmt
	cargo +nightly build
	cargo +nightly clippy -- -D warnings

clean:
	cargo clean
	rm -rf ./translator/test_infra/bin

rebuild: clean
	cargo +nightly build
