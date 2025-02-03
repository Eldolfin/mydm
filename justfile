export RUST_LOG := "debug"
export RUST_LOG_STYLE := "always"

weston-watch:
    weston -- just watch

watch:
    git ls-files | entr -ncr cargo run
