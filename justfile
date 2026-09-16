in_nix := env_var_or_default("IN_NIX_SHELL","")

# warns if you forget to 'nix develop' first as most recipes here rely
# on the tools (probe-rs, minicom) that only exist inside the flake shell

_check_nix_shell:
	@if [ -z "{{in_nix}}" ]; then \
		echo "Warning: Not in a Nix shell. Consider using 'nix develop'.";\
	fi

build: _check_nix_shell
	cargo build --release #run this just command to build the .rs file

flash: _check_nix_shell
	cargo run --release #run this just comman to flash the FW

flash-first: _check_nix_shell
	cargo run --release -- --allow-erase-all #run this just command is the chip is fresh ot locked

monitor: _check_nix_shell
	minicom -D /dev/ttyACM1 -b 115200 #run this just command to load a minicom terminal

	# **Warning** this comman will delete the entire target directory
clean:
	cargo clean #run this just comman to remove build artificats and free up desk space

