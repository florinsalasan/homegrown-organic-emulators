To build run the following in the root chip8 folder:

    cmake -B BUILD_DIRECTORY -S . 

This will create the BUILD_DIRECTORY and sets the source to the root of the chip8 folder

To run, go to the BUILD_DIRECTORY, and run the following:

if you haven't built the binary yet run:
    make

if the binary already exists run:
    ./mygame TIME_BETWEEN_FRAMES_IN_MS PATH_TO_CHIP8_ROM

Whenever you want to change anything in the source code just go and rerun make in the build
directory to rebuild the binary with the new changes

The code is provided as is and has not been thoroughly tested, this was just a side project 
that I wanted to run for myself, SDL needs to be installed on your machine to use this, the cmake 
file might need to be adjusted based on the SDL installation. 
