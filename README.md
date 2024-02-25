todo: update this readme with information.

# setup
```
rustup target install wasm32-unknown-unknown
cargo install wasm-server-runner
cargo install cargo-watch
```

# developing
```
cargo watch -cx "run --target wasm32-unknown-unknown"
```

or

```
cargo run --features bevy/dynamic_linking
```

# building for web
```
trunk build --public-url "http://vvn.space/bevy_walky/"
```

# template

I have started to graft parts of [bevy_game_template](https://github.com/NiklasEi/bevy_game_template) retroactively.