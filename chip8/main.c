#include <stdio.h>
#include <stdbool.h>
#include <unistd.h>
#include <errno.h>
#include <sys/stat.h>

#include <SDL.h>
#include "include/chip8.h"

int DEBUG = 1;

// Define screen dimensions
#define SCREEN_WIDTH    64
#define SCREEN_HEIGHT   32
#define SDL_SCALING     8

bool should_quit = false;

SDL_Window* screen;
SDL_Renderer* renderer;

SDL_Scancode keymappings[16] = {
    SDL_SCANCODE_1, SDL_SCANCODE_2, SDL_SCANCODE_3, SDL_SCANCODE_4,
    SDL_SCANCODE_Q, SDL_SCANCODE_W, SDL_SCANCODE_E, SDL_SCANCODE_R,
    SDL_SCANCODE_A, SDL_SCANCODE_S, SDL_SCANCODE_D, SDL_SCANCODE_F,
    SDL_SCANCODE_Z, SDL_SCANCODE_X, SDL_SCANCODE_C, SDL_SCANCODE_V,
};

// Font:
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

// Memory implementation: 
// Total space is 4kb or 4096bytes;
//
// Then 0x000 - 0x1FF is space for the interpreter in most chip8 roms;
// 0x200 - 0xFFF is the program and data space
//
// Keep in mind that opcodes are big-endian while macs use little-endian 
//
// The following structures are being implemented as described in the specs
// section of Tobias Langhoff's CHIP-8 Guide.

// Memory
unsigned char memory[4096] = {0};

// Registers, CHIP-8 used 16 general purpose 8-bit registers, referred to 
// as VX where X is a hexadecimal digit, so V0-VF but ours will be stored in 
// an array and can be indexed after, use unsigned chars since they're 8-bits
unsigned char V[16] = {0};

// Special 16-bit register 'I' that is the index register which points at
// locations in memory, use short since that is 16-bits
unsigned short I = 0;

// The program counter 'PC' which points at the current instruction that is in memory
// reminder that in memory that program space starts at 0x200
unsigned short PC = 0x200;

// The stack, an array of 16-bit addresses, most original interpreters apparently
// had very limited space, with some limiting it to 2 addresses even, here I'll
// take a very excessive implementation of eight 16-bit addresses.
unsigned short stack[16] = {0};

// the keypad:
unsigned char keypad[16] = {0};

// display of height 64, and width of 32, stored in a 1D array
unsigned char display[SCREEN_WIDTH * SCREEN_HEIGHT] = {0};

// delay timer
unsigned char delay_timer = 0;

// sound timer
unsigned char sound_timer = 0;

// additional flag defined to make updating display simpler
// display flag
unsigned char draw_flag = 0;

//////////////////////////////////
// CHIP-8 Functionality:        //
//////////////////////////////////

void init_cpu(void) {

    // load the fontset into memory
    memcpy(memory, fontset, sizeof(fontset));

}

int load_rom(char* filename) {

    FILE* fp = fopen(filename, "rb");

    if (fp == NULL) return errno;

    struct stat file_stat;
    stat(filename, &file_stat);
    size_t file_size = file_stat.st_size;

    size_t bytes_read = fread(memory + 0x200, 1, sizeof(memory) - 0x200, fp);

    fclose(fp);

    if (bytes_read != file_size) {
        // ensure that the entire rom is loaded in, if not early return to error out
        return -1;
    }

    return 0;

}

//////////////////////////////////
// SDL HANDLING CODE GOES HERE: //
//////////////////////////////////

void init_sdl_display(void) {

    SDL_Init(SDL_INIT_VIDEO);
    screen = SDL_CreateWindow("CHIP-8", SDL_WINDOWPOS_CENTERED,
                             SDL_WINDOWPOS_CENTERED, SCREEN_WIDTH * SDL_SCALING, 
                             SCREEN_HEIGHT * SDL_SCALING, 0);
    renderer = SDL_CreateRenderer(screen, -1, SDL_RENDERER_ACCELERATED);

}

void draw_on_screen(unsigned char* display) {
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255); 

    SDL_RenderClear(renderer);

    SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);

    for (int y = 0; y < SCREEN_HEIGHT; y++) {
        for (int x = 0; x < SCREEN_WIDTH; x++) {
            if (display[x + (y * SCREEN_WIDTH)]) {
                SDL_Rect rect;

                rect.x = x * SDL_SCALING;
                rect.y = y * SDL_SCALING;
                rect.w = SDL_SCALING;
                rect.h = SDL_SCALING;

                SDL_RenderFillRect(renderer, &rect);

            }
        }
    }

    SDL_RenderPresent(renderer);

}

void sdl_handler(unsigned char* keypad) {
    
    SDL_Event event;

    if (SDL_PollEvent(&event)) {
        const Uint8* state = SDL_GetKeyboardState(NULL);

        switch (event.type) {
            case SDL_QUIT:
                should_quit = 1;
                break;
            default: 
                if (state[SDL_SCANCODE_ESCAPE]) {
                    should_quit = 1;
                }
                for (int keycode = 0; keycode < 16; keycode++) {
                    keypad[keycode] = state[keymappings[keycode]];
                }
                break;
        }
    }
}

void stop_display(void) {
    SDL_DestroyWindow(screen);
    SDL_Quit();
}

///////////////////////////////
// CHIP-8 'EMULATOR'         //
///////////////////////////////

// This is the function that will modify and use the emulated structures of a chip-8
// system. This will be what fetches, decodes, and executes opcodes from the roms
// controlling the emulated system.
void emulate_cycle(void) {

    unsigned short op = memory[PC] << 8 | memory[PC + 1];
    printf("opcode: %u\n", op);

    // grab 'nibbles' from the instruction opcode, 
    // first nibble is what specifies the instruction type
    // X: second nibble is for grabbing of the 16 registers, VX from V0-VF;
    // Y: third nibble is also for grabbing a register VY, from V0-VF;
    // N: 4th nibble a 4-bit number
    // NN: second byte (3rd and 4th nibbles), an 8-bit immediate number
    // NNN: 2nd, 3rd, 4th nibbles, 12-bit immediate mem address.

    unsigned short X = (op & 0x0F00) >> 8;
    unsigned short Y = (op & 0x00F0) >> 4;

    printf("opcode second and third nibbles: %u, %u\n", X, Y);

    switch (op) {
        case 0x00E0: // Clear screen
            debug_print("[OK] 0x%X: 00E0\n", op);
            for (int i = 0; i < SCREEN_WIDTH * SCREEN_HEIGHT; i++) {
                display[i] = 0;
            }
            PC += 2;
            break;
        case (op & 0xF000):
            switch (op)

    }

}

///////////////////////////////
// EMULATION CYCLE HANDLER   //
///////////////////////////////

int main(int argc, char** argv) {
    if (argc != 3) {
        perror("Usage: emulator rom.ch8 CYCLE_LEN");
        return 1;
    }

    printf("[PENDING] Initializing CHIP-8 interpreter");
    init_cpu();
    printf("[OK] Done!");

    char* ptr;

    char* rom = argv[1];
    int CYCLE_LEN = strtol(argv[2], &ptr, 10); // Recommended is around ~1400
    printf("[PENDING] Loading rom %s... \n", rom);
    int err_check_load_rom = load_rom(rom);
    if (err_check_load_rom) {
        if (err_check_load_rom == -1) {
            error("[FAILED] fread() failure: the return value is not equal to the rom file size.");
        } else {
            perror("Error while loading rom");
        }
        return 1;
    }

    printf("[OK] Rom loaded successfully!");

    init_sdl_display();
    printf("[OK] Display initialized");

    while (!should_quit) {

        emulate_cycle();
        sdl_handler(keypad);

        if (draw_flag) {
            draw_on_screen(display);
        }

        // sleep to match typical chip-8 clock speed
        usleep(CYCLE_LEN);

    }

    stop_display();
    return 0;

}
