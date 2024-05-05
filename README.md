# Airbnb hosting reservations scraper

For cross building (MacOS -> Windows) install:

```zsh
cargo install cross
docker pull --platform linux/amd64 ghcr.io/cross-rs/x86_64-pc-windows-gnu:0.2.5
cross build --target x86_64-pc-windows-gnu --release
```
