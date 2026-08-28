run:
	cargo fmt
	cargo clippy
	cargo run

t:
	cargo fmt
	cargo clippy
	cargo test -- --nocapture
