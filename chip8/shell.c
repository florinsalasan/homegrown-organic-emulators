#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

#define SCREEN_HEIGHT 32
#define SCREEN_WIDTH 64

void clear_screen(bool[SCREEN_HEIGHT][SCREEN_WIDTH]);

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
bool display[SCREEN_WIDTH][SCREEN_HEIGHT] = {false};

// Use this for the call stack, early ones apparently had space for two
// instructions, I went a bit overboard and have space for 32 16-bit addresses
unsigned short stack[32] = {0};

// The delay and sound timer registers:
unsigned char delay_timer = 0;
unsigned char sound_timer = 0;

// implement the keypad somehow:

int main () {
    // Run the instruction loop in here

}

void clear_screen (bool display[SCREEN_HEIGHT][SCREEN_WIDTH]) {

    int i, j;
    for (i = 0; i < SCREEN_HEIGHT; i++) {
        for (j = 0; j < SCREEN_WIDTH; j++) {
            display[j][i] = false;
        }
    }
    printf("clearing");

}
