in_nix := env_var_or_default("IN_NIX_SHELL","")

_check_nix_shell:
	@if [ -z "{{in_nix}}" ]; then \
		echo "Warning: Not in a Nix shell. Consider using 'nix develop'.";\
	fi

build: _check_nix_shell
	cargo build --release

flash: _check_nix_shell
	cargo run --release

flash-first: _check_nix_shell
	cargo run --release -- --allow-erase-all

monitor: _check_nix_shell
	minicom -D /dev/ttyACM1 -b 115200

clean:
	cargo clean
