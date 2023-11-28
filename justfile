test:
    cargo test
    cargo clippy -- -D warnings
    cargo build --example full

readme:
	cargo readme \
	    -r slowchop_console \
	    --no-title \
	    --no-indent-headings \
	    -o README.md

release: readme test
    cargo publish -p slowchop_console_derive
    cargo publish -p slowchop_console
