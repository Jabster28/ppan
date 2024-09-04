set shell := ["bash", "-uc"]
default:
  just --list
test:
    cargo +nightly-2024-02-01 fmt -- --check
    cargo +nightly-2024-02-01 test
    cargo +nightly-2024-02-01 clippy -- -W clippy::pedantic

run:
    cargo +nightly-2024-02-01 run --features bevy/dynamic_linking

build:
    @touch discord_game_sdk/c/discord_game_sdk.h || just discord_sdk
    . ./env.sh && cargo +nightly-2024-02-01 build --release --all-features

# installs butler (only works on linux tho)
butler:
    cp "$(which butler)" . || (curl -L -o butler.zip https://broth.itch.ovh/butler/linux-amd64/LATEST/archive/default && unzip butler.zip && chmod +x butler)
    ./butler -V

cleanup:
    sudo rm -rf butler butler.zip 7z.so libc7zip.so AppDir appimage-builder appimage-build discord_game_sdk

# make a portable build ready for itch.io
package: build
    rm -rf dist
    mkdir -p dist
    mkdir -p assets
    just {{target_os}}


linux:
    mkdir -p dist/assets
    cargo +nightly-2024-02-01 install copydeps
    cp -r assets dist/
    cp itch/linux.itch.toml dist/.itch.toml
    cp target/release/ppan dist/ppan.{{arch()}}
    cp ppan.sh dist/ppan
    mkdir -p dist/{{arch()}}
    -. ./env.sh && copydeps --search-dir $DISCORD_GAME_SDK_PATH/lib/x86_64 target/release/ppan dist/{{arch()}}
    cd dist/{{arch()}}
    rm -f libc.so.* libm.so.* libdl.so.* librt.so.* libpthread.so.* libgcc_s.so.*
    cd ../..


# you'll need to install dylibbundler (brew install dylibbundler)
macos:
    cargo +nightly-2024-02-01 install cargo-bundle
    magick ppan.png -sample 1028x1028 512x512@2x.png
    . ./env.sh && cargo +nightly-2024-02-01 bundle --release --all-features
    rm 512x512@2x.png
    . ./env.sh && dylibbundler --search-path $DISCORD_GAME_SDK_PATH/lib/x86_64  -od -b -x target/release/bundle/osx/ppan.app/Contents/MacOS/ppan -d target/release/bundle/osx/ppan.app/Contents/Frameworks -p @executable_path/../Frameworks/
    cp -r assets target/release/bundle/osx/ppan.app/Contents/MacOS/assets
    cp itch/macos.itch.toml dist/.itch.toml
    cp -r target/release/bundle/osx/ppan.app dist/
win:
    rm -rf dist
    mkdir -p dist
    apt-get install mingw-w64 -qq
    rustup target add x86_64-pc-windows-gnu
    cargo +nightly-2024-02-01 build --target x86_64-pc-windows-gnu --release
    cp -r assets dist/
    cp itch/win.itch.toml dist/.itch.toml
    cp target/x86_64-pc-windows-gnu/release/ppan.exe dist/


# publishes to itch
publish version arch="64": butler package
    ./butler push dist "jabster28/ppan:{{target_os}}-{{arch}}-bit" --userversion {{version}}


# publishes beta to itch
publish-beta version arch="64": butler package
    ./butler push dist "jabster28/ppan:{{target_os}}-{{arch}}-bit-(beta)" --userversion {{version}}

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

# make a installer for people not using the itch.io app
installer: package
    just installer{{target_os}}

# you'll need to install create-dmg (brew install create-dmg)

installermacos:
    pkgbuild --component dist/ppan.app dist/ppan.pkg --install-location /Applications
    create-dmg --volname "ppɒŋ" --hide-extension "ppan.app" --app-drop-link 600 185 --skip-jenkins dist/ppan.dmg dist/ppan.app 

installerlinux:
    sudo rm -rf AppDir/ appimage-build ppan.AppImage
    test -f appimage-builder || wget -O appimage-builder https://github.com/AppImageCrafters/appimage-builder/releases/download/v1.0.0-beta.1/appimage-builder-1.0.0-677acbd-x86_64.AppImage
    chmod +x appimage-builder
    mkdir -p AppDir/
    cp -r dist/* AppDir/
    mkdir -p AppDir/usr/share/icons/hicolor/32x32/apps/
    cp ppan.png AppDir/usr/share/icons/hicolor/32x32/apps/
    mkdir -p AppDir/lib/x86_64
    mv AppDir/x86_64/ AppDir/lib
    sudo ./appimage-builder
    mv ppan-latest-x86_64.AppImage ppan.AppImage
    -sudo chown -R $USER:$USER ppan.AppImage AppDir appimage-build
    chmod +x ppan.AppImage
    mv ppan.AppImage dist/ppan.AppImage
    rm -rf AppDir/ appimage-build
# TODO: installer for wangblows
installerwin:


target_os := if env_var_or_default("WINDOWS", "0") == "1" { "win" } else {
    ```if [ "$(uname -s)" = "Linux" ]; then
        echo "linux"
    elif [ "$(uname -s)" = "Darwin" ]; then
        echo "macos"
    else
        echo "unknown"
    fi```
}
