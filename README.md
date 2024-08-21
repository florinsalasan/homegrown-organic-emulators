# Emulators that I'm working on

## CHIP-8

Yes technically not an emulator, but skills I learn accomplishing this should
hopefully help with the other projects I want to work on afterwards.

This is mostly in a working state, all test ROMs I've found and ran passes all tests
however there are still strange behaviour happening in certain games that I've tried
like much higher frame rates than expected in tetris, and space invaders breaking
when hitting a specific spaceship.

## The NES

The NES, uses the Ricoh 2A03, which was based on the 6502, and I will be emulating the modified
6502 and the specific architecture of the NES rather than a generic 6502.

## TODO:

- [x] CHIP-8 interpreter has instructions implemented, needs to be debugged, some things like timers & keypad need to be double checked.
- [x] CHIP-8 is now passing all tests I can throw at it other than display wait, so I'm considering this project finished, could add a changeable game speed again though
- [ ] Get the NES emulator working
