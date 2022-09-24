default:
  just --list
test:
    cargo test

build: test
    cargo build --release

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
    cargo install copydeps
    cp -r assets dist/
    cp target/release/ppan dist/ppan.{{arch()}}
    cp ppan.sh dist/ppan
    mkdir -p dist/{{arch()}}
    -copydeps target/release/ppan dist/{{arch()}}


macos:
    cargo install cargo-bundle
    cargo bundle --release
    dylibbundler -od -b -x target/release/bundle/osx/ppan.app/Contents/MacOS/ppan -d target/release/bundle/osx/ppan.app/Contents/libs
    cp -r assets target/release/bundle/osx/ppan.app/Contents/MacOS/
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
    cp target/x86_64-pc-windows-gnu/release/ppan.exe dist/

publish-win version: win butler
    ./butler push dist "jabster28/ppan:windows-64-bit" --userversion {{version}}
    just cleanup
publish-beta-win version: win butler
    ./butler push dist "jabster28/ppan:windows-64-bit-(beta)" --userversion {{version}}
    just cleanup
