# bevy_walky

This is a (so far) unnamed attempt to replicate movement similar to that of the 3d Sonic games in [Bevy](https://bevyengine.org/). I'm currently using [Bevy XPBD](https://github.com/Jondolf/bevy_xpbd) for collisions, the character movement is a kinematic object I've implementing here.

Note that it is **extremely** a work in progress.

This is deployed to github pages, you can try it in your browser: [https://vivlim.github.io/bevy_walky]

## Controls

| Action | Gamepad | Keyboard/Mouse |
|--------|---------|----------------|
| Move   | Left stick | Arrow keys (*not wasd atm*)|
| Look around | Right stick | Mouse movement, but it's currently commented out since it's annoying when I want to interact with the debug ui |
| Jump   | Button 0 (A on an xbox controller) | Not bound currently |

[Here are some toots I've written about this on Mastodon](https://snoot.tube/@viv/111961252199595732).

# template

I have started to graft parts of [bevy_game_template](https://github.com/NiklasEi/bevy_game_template) onto this retroactively.

# rough notes

Below is notes from the old readme I haven't cleaned up yet.

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
