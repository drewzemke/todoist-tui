build:
	cargo build

test:
	cargo test

run:
	cargo run

install:
	git pull && cargo install --path .
