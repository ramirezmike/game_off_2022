# Adore's Antiques 

An entry for the [Github Game Off 2022 jam][jam] following the theme of "cliche." 

You can play the game [here][itch].

Other submissions for the jam can be seen [here][submissions] (please check them out!)

Adore's Antiques is singleplayer action game where you attempt to assist the owners of a small antique shop get ahead of their local competition.

Check out my other games [here][othergames]. Also, I'm always hanging out in the [bevy discord][bevy-discord], definitely feel free to @ramirezmike me and ask questions or criticize me :)


# Running the Game

To run the game locally

```
cargo r --features bevy/dynamic
```

You can also compile it for wasm with a little effort and host it locally

```
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --out-name bull_cliche --out-dir wasm --target web target/wasm32-unknown-unknown/release/bull_cliche.wasm
cd wasm
http-server .
```

[jam]: https://itch.io/jam/game-off-2022
[bevy]: https://bevyengine.org
[itch]: https://ramirezmike2.itch.io/adores-antiques
[submissions]: https://itch.io/jam/game-off-2022/entries
[othergames]: https://ramirezmike2.itch.io/
[bevy-discord]: https://discord.gg/bevy
