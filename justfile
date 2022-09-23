default:
  just --list
test:
    cargo test

build: test
    cargo build --release

# installs butler
butler:
    cp "$(which butler)" . || (curl -L -o butler.zip https://broth.itch.ovh/butler/linux-amd64/LATEST/archive/default && unzip butler.zip && chmod +x butler)
    ./butler -V

cleanup:
    rm -rf butler butler.zip 7z.so

package: build butler
    rm -rf dist
    mkdir -p dist
    mkdir -p assets
    just {{os()}}

linux:
    cp -r assets dist/
    cp target/release/ppan dist

macos:
    cargo install cargo-bundle
    cargo bundle --release
    dylibbundler -od -b -x target/release/bundle/osx/ppan.app/Contents/MacOS/ppan -d target/release/bundle/osx/ppan.app/Contents/libs
    cp -r assets target/release/bundle/osx/ppan.app/Contents/MacOS/
    cp -r target/release/bundle/osx/ppan.app dist/


# publishes beta to itch
publish-beta version arch="64": package
    ./butler push dist "jabster28/ppan:{{os()}}-{{arch}}-bit-(beta)" --userversion {{version}}
    just cleanup

# publishes to itch
publish version arch="64": package
    ./butler push dist "jabster28/ppan:{{os()}}-{{arch}}-bit" --userversion {{version}}
    just cleanup
