To build run the following in the root chip8 folder:

    cmake -B BUILD_DIRECTORY -S . 

This will create the BUILD_DIRECTORY and sets the source to the root of the chip8 folder

To run, go to the BUILD_DIRECTORY, and run the following:

if you haven't built the binary yet run:
    make

if the binary already exists run:
    ./mygame PATH_TO_CHIP8_ROM

Whenever you want to change anything in the source code just go and rerun make in the build
directory to rebuild the binary with the new changes

The code is provided as is and has not been thoroughly tested, this was just a side project 
that I wanted to run for myself, SDL needs to be installed on your machine to use this, the cmake 
file might need to be adjusted based on the SDL installation. 

## TODO/BROKEN:
- [x] Display clipping gives err2 on the quirks test ROM from timendus' test suite, now passes
- [x] Display wait gives slow on the same quirks test ROM
- [x] Display for numbers on certain ROMs are completely broken including pong scores and tank values for what I assume are angles and power
- [x] FX0A instruction doesn't wait for the key to be released to continue
- [ ] Add game speed back as an adjustable argument to pass in
- [ ] Entire test suite that I have passes, but playing space invaders is still broken
