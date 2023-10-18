#!/usr/bin/env just --justfile

linux:
    cross build --target x86_64-unknown-linux-musl --release

mac:
    cross build --target x86_64-apple-darwin --release

windows:
    cross build --target x86_64-pc-windows-gnu --release

release: mac linux windows
