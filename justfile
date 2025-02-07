export RUST_LOG := "debug"
export RUST_LOG_STYLE := "always"
export QEMU_OPTS := "-display default,show-cursor=on -enable-kvm"

weston-watch:
    weston -- just watch

watch:
    git ls-files | entr -ncr cargo run

test-nix:
    nix build .\#checks.x86_64-linux.launch |& nom

test-nix-interactive:
    set -xeu -o pipefail
    nix build .\#checks.x86_64-linux.launch.driverInteractive && ./result/bin/nixos-test-driver
