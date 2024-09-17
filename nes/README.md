Requires Rust and Cargo to be installed

To build run the following in the root nes folder:

    cargo build

This creates a target directory which you can ignore.
To run the emulator run

    cargo run

and if you make changes and want to test it automatically run 

    cargo test

The code is provided as is and has not been thoroughly tested, this was just a side project 
that I wanted to run for myself, SDL2 needs to be installed on your machine to use this

## Resources used along the way 
- Bugzmanov's [Writing NES Emulator in Rust](https://bugzmanov.github.io/nes_ebook/) guide
- nesdev.org has been a massive [help](https://www.nesdev.org) plenty of info throughout the site

## TODO/BROKEN:
- [ ] Get the emulator running the test rom
- [ ] Remove the test rom from hard coded in
- [ ] Allow users to run whatever NES rom they want by loading it into the emulator
- [ ] Just getting the thing running and then adding a bunch of tests
