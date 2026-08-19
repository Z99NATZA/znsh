run:
	cargo fmt
	cargo clippy
	cargo run

tests:
	cargo test -- --nocapture