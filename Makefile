target/debug/libpike_mod.so: src/lib.rs Cargo.toml
	cargo build

make: target/debug/libpike_mod.so