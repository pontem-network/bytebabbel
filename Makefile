pre_commit:
	cargo fmt
	cargo +nightly build
	cargo +nightly clippy -- -D warnings
	cargo +nightly test

clean:
	cargo clean
	rm -rf ./translator/test_infra/bin

rebuild: clean
	cargo +nightly build
