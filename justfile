default:
  just --list
test:
    cargo fmt -- --check
    cargo test
    cargo clippy -- -W clippy::pedantic

run:
    cargo run --features bevy/dynamic

build:
    . ./env.sh && cargo build --release --all-features

# installs butler (only works on linux tho)
butler:
    cp "$(which butler)" . || (curl -L -o butler.zip https://broth.itch.ovh/butler/linux-amd64/LATEST/archive/default && unzip butler.zip && chmod +x butler)
    ./butler -V

cleanup:
    rm -rf butler butler.zip 7z.so libc7zip.so

package: build
    rm -rf dist
    mkdir -p dist
    mkdir -p assets
    just {{os()}}


linux:
    mkdir -p dist/assets
    cargo install copydeps
    cp -r assets dist/
    cp .itch.toml dist/
    cp target/release/ppan dist/ppan.{{arch()}}
    cp ppan.sh dist/ppan
    mkdir -p dist/{{arch()}}
    -. ./env.sh && copydeps --search-dir $DISCORD_GAME_SDK_PATH/lib/x86_64 target/release/ppan dist/{{arch()}}


macos:
    cargo install cargo-bundle
    cargo bundle --release
    dylibbundler -od -b -x target/release/bundle/osx/ppan.app/Contents/MacOS/ppan -d target/release/bundle/osx/ppan.app/Contents/discord_game_sdk
    cp -r assets target/release/bundle/osx/ppan.app/Contents/MacOS/assets
    cp .itch.toml dist/
    cp -r target/release/bundle/osx/ppan.app dist/


# publishes to itch
publish version arch="64": butler package
    ./butler push dist "jabster28/ppan:{{os()}}-{{arch}}-bit" --userversion {{version}}
    just cleanup

# publishes beta to itch
publish-beta version arch="64": butler package
    ./butler push dist "jabster28/ppan:{{os()}}-{{arch}}-bit-(beta)" --userversion {{version}}
    just cleanup


win:
    rm -rf dist
    mkdir -p dist
    apt-get install mingw-w64 -qq
    rustup target add x86_64-pc-windows-gnu
    cargo build --target x86_64-pc-windows-gnu --release
    cp -r assets dist/
    cp .itch.toml dist/
    cp target/x86_64-pc-windows-gnu/release/ppan.exe dist/

publish-win version: win butler
    ./butler push dist "jabster28/ppan:windows-64-bit" --userversion {{version}}
    just cleanup
publish-beta-win version: win butler
    ./butler push dist "jabster28/ppan:windows-64-bit-(beta)" --userversion {{version}}
    just cleanup

discord_sdk:
    rm -rf discord_game_sdk
    rm -f env.sh
    mkdir -p discord_game_sdk
    echo "#!/bin/bash" > env.sh
    echo "# source this file (run '. ./env.sh') to load the discord sdk library" >> env.sh
    wget https://dl-game-sdk.discordapp.net/latest/discord_game_sdk.zip -O dgs.zip
    unzip -o dgs.zip -d discord_game_sdk
    rm dgs.zip
    export DISCORD_GAME_SDK_PATH=$(pwd)/discord_game_sdk/
    echo "export DISCORD_GAME_SDK_PATH=$(pwd)/discord_game_sdk" | tee -a env.sh
    just discord{{os()}}

discordlinux:
    -echo $LD_LIBRARY_PATH
    cp $(pwd)/discord_game_sdk/lib/x86_64/{,lib}discord_game_sdk.so
    echo "export LD_LIBRARY_PATH=/usr/lib:${LD_LIBRARY_PATH:+${LD_LIBRARY_PATH}:}\$DISCORD_GAME_SDK_PATH/lib/x86_64" | tee -a env.sh

discordmacos:
    -echo $DYLD_LIBRARY_PATH
    cp $(pwd)/discord_game_sdk/lib/x86_64/{,lib}discord_game_sdk.dylib
    echo "export DYLD_LIBRARY_PATH=/usr/lib:${DYLD_LIBRARY_PATH:+${DYLD_LIBRARY_PATH}:}\$DISCORD_GAME_SDK_PATH/lib/x86_64" | tee -a env.sh
