#include <stdio.h>
#include <stdbool.h>

#define SCREEN_HEIGHT 32
#define SCREEN_WIDTH 64

void clear_screen(bool[SCREEN_HEIGHT*SCREEN_WIDTH]);
void push_to_stack(unsigned short*, short, unsigned char*);
short pop_from_stack(unsigned short*, unsigned char*);
void run_instruction();

// Font set
unsigned char fontset[80] = {
    0xF0, 0x90, 0x90, 0x90, 0xF0,  // 0
    0x20, 0x60, 0x20, 0x20, 0x70,  // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,  // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,  // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,  // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,  // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,  // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,  // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,  // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,  // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,  // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,  // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,  // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,  // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,  // E
    0xF0, 0x80, 0xF0, 0x80, 0x80   // F
};

// '4kb' of memory
unsigned char memory[4096] = {0};

// display of 32x64
bool display[SCREEN_WIDTH*SCREEN_HEIGHT] = {false};

// Use this for the call stack, early ones apparently had space for two
// instructions, I went a bit overboard and have space for 32 16-bit addresses
unsigned short stack[32] = {0};

// The delay and sound timer registers:
// These get decremented 60 times per second until hitting 0, don't
// have a place where their values are first set or when the timers start
unsigned char delay_timer = 0;
unsigned char sound_timer = 0;

// Program counter, 'PC', pointer to the current instruction in memory:
// Since memory was set to unsigned chars, need to match types, I think
unsigned char* PC;

// Index register, 'I', pointer to locations in memory
unsigned char* I;

// Create the 16 different 8-bit general-purpose variable registers that 
// are number 0-F in hex ie 'V0' to 'VF'
unsigned char V0 = 0;
unsigned char V1 = 0;
unsigned char V2 = 0;
unsigned char V3 = 0;
unsigned char V4 = 0;
unsigned char V5 = 0;
unsigned char V6 = 0;
unsigned char V7 = 0;
unsigned char V8 = 0;
unsigned char V9 = 0;
unsigned char VA = 0;
unsigned char VB = 0;
unsigned char VC = 0;
unsigned char VD = 0;
unsigned char VE = 0;
// VF is kind of a special register, is also used as a flag register where
// many instructions set it to either 1 or 0 based on what the instruction 
// needs, ie can be used as a carry flag.
unsigned char VF = 0;

// implement the keypad somehow:
// Seems like it might be better to do this in the SDL implementation for
// actually drawing a window and interacting with it

int main () {
    // Run the instruction loop in here
    // The loop is fetch, decode, execute
    // instruction is fetched from memory at the current PC
    // Instructions are two bytes, so read two bytes from memory and 
    // increment 

    // The fetched instruction is then decoded 
    // the decoded instruction is then executed

}

// helper functions to use for different instruction codes.
void clear_screen (bool display[SCREEN_HEIGHT*SCREEN_WIDTH]) {

    int i, j;
    for (i = 0; i < SCREEN_HEIGHT * SCREEN_WIDTH; i++) {
        display[i] = false;
    }
    printf("clearing");

}

// Stack helpers, should probably have push and pop at the very least
void push_to_stack (unsigned short* the_stack, short value_pushed, unsigned char* index) {

    unsigned long insert_index = (unsigned long) index + 1;
    the_stack[insert_index * sizeof(value_pushed)] = value_pushed;
    index++;
    return;
    
}


short pop_from_stack (unsigned short* the_stack, unsigned char* index) {

    unsigned long curr_index = (unsigned long) index;
    short to_return = the_stack[curr_index * sizeof(short)];
    index--;
    return to_return;
    
}

void run_instruction () {
    // Probably changes based on the endianness of the machine this is run on
    // I'm writing this on an Apple silicon machine that is little-endian, 
    // meanwhile chip-8 is big-endian so for my machine I need to swap instruction
    // Just do some of the switch cases in here I think

    unsigned short op = memory[*PC] << 8 | memory[*PC + 1];

    switch (op & 0xF000) {
        case 0x0000:
            switch (op & 0x00FF) {
                case 0X00E0:
                    printf("[OK] 0x%X: 00E0\n", op);
                    clear_screen(display);
            }
    }

}
