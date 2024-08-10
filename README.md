# Emulators that I'm working on

## CHIP-8

Yes technically not an emulator, but skills I learn accomplishing this should
hopefully help with the other projects I want to work on afterwards.

This is mostly in a working state, some things are wrong with the display that I can't
seem to track down, I also think I broke collision in a tank game when trying to fix some things

## The 8080

This is the first emulator that I was going to work on after I had found an overview
on how to write one on the website emulator101.com. This would be specific to space 
invaders for now, but it has been put on the backburner.

## The 6502 

This is the processing unit that the chip used in the NES, the Ricoh 2A03, was based on, 
emulating the 6502 from MOS Technology should be able to execute the instruction set that
the modified chip in the NES would have run. This will be the current focus for the project
after I get a CHIP-8 interpreter up and running.

## TODO:

- [ ] Pretty much everything, 8080 and 6502 are nowhere near running and chip-8 still requires instructions to be implemented
- [x] chip-8 interpreter has instructions implemented, needs to be debugged, some things like timers & keypad need to be double checked.
