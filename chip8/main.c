#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>
#include <unistd.h>
#include <errno.h>
#include <sys/stat.h>
#include <stdlib.h>
#include <time.h>

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
    SDL_SCANCODE_X, SDL_SCANCODE_1, SDL_SCANCODE_2, SDL_SCANCODE_3,
    SDL_SCANCODE_Q, SDL_SCANCODE_W, SDL_SCANCODE_E, SDL_SCANCODE_A,
    SDL_SCANCODE_S, SDL_SCANCODE_D, SDL_SCANCODE_Z, SDL_SCANCODE_C,
    SDL_SCANCODE_4, SDL_SCANCODE_R, SDL_SCANCODE_F, SDL_SCANCODE_V,
};

// Font:
uint8_t fontset[80] = {
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
uint8_t memory[4096] = {0};

// Registers, CHIP-8 used 16 general purpose 8-bit registers, referred to 
// as VX where X is a hexadecimal digit, so V0-VF but ours will be stored in 
// an array and can be indexed after, use unsigned chars since they're 8-bits
uint8_t V[16] = {0};

// Special 16-bit register 'I' that is the index register which points at
// locations in memory, use short since that is 16-bits
uint16_t I = 0;

// The program counter 'PC' which points at the current instruction that is in memory
// reminder that in memory that program space starts at 0x200
uint16_t PC = 0x200;

// The stack, an array of 16-bit addresses, most original interpreters apparently
// had very limited space, with some limiting it to 2 addresses even, here I'll
// take a very excessive implementation of eight 16-bit addresses.
uint16_t stack[16] = {0};

// points to the top of the stack so to speak.
uint8_t stack_idx = 0;

// the keypad:
uint8_t keypad[16] = {0};

// display of height 64, and width of 32, stored in a 1D array
uint8_t display[SCREEN_WIDTH * SCREEN_HEIGHT] = {0};

// delay timer
uint8_t delay_timer = 0;

// sound timer
uint8_t sound_timer = 0;

// additional flag defined to make updating display simpler
// display flag
uint8_t draw_flag = 0;
uint8_t sound_flag = 0;

bool key_found = false;
uint8_t key_pressed = 255;

//////////////////////////////////
// CHIP-8 Functionality:        //
//////////////////////////////////

void init_cpu(void) {

    // Set a seed for the random functions in the rest of the interpreter.
    srand((unsigned int)time(NULL));

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

    // TODO: Fix going offscreen behaviour

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
bool emulate_cycle(void) {

    uint16_t op = memory[PC] << 8 | memory[PC + 1];
    printf("opcode: %X\n", op);
    int opcode_type = (op & 0xF000) >> 12;

    int op_nibbles = op & 0x0FFF;

    // grab 'nibbles' from the instruction opcode, 
    // first nibble is what specifies the instruction type
    // X: second nibble is for grabbing of the 16 registers, VX from V0-VF;
    // Y: third nibble is also for grabbing a register VY, from V0-VF;
    // N: 4th nibble a 4-bit number
    // NN: second byte (3rd and 4th nibbles), an 8-bit immediate number
    // NNN: 2nd, 3rd, 4th nibbles, 12-bit immediate mem address.

    uint8_t X = (op & 0x0F00) >> 8;
    uint8_t Y = (op & 0x00F0) >> 4;

    printf("opcode_type: %X, op_nibbles: X: %X, Y: %X\n", opcode_type, X, Y);

    switch (opcode_type) {
        case 0x0: // First digit is a zero: 
            switch(op_nibbles) {
                case 0x0E0: // combined the opcode is 0x00E0 which clears the screen
                    printf("[OK] 0x%X: 00E0\n", op);
                    for (int i = 0; i < SCREEN_WIDTH * SCREEN_HEIGHT; i++) {
                        display[i] = 0;
                    }
                    PC += 2;
                    break;
                case 0x0EE: // Return from subroutine setting PC to address at top of 
                    // stack, then subtracting one from the stack pointer.
                    printf("[OK] 0x%X: 00EE\n", op); // Prints op just as EE rather than 00EE
                    // why does this not print as expected??
                    // Get top of stack
                    PC = stack[stack_idx];
                    stack_idx--;
                    PC += 2;
                    break;
                // Remaining cases for 0x0NNN are made to jump to a machine code routine
                // at NNN, according to one of the guides I'm following, this instruction
                // does not get implemented in modern interpreters.
                default:
                    printf("[ERROR] these instructions shouldn't be getting called, %X\n", op);
                    break;
            }
            break;
        case 0x1:
            // For this case of 0x1NNN, it is a jump to location NNN, ie setting the PC
            // to NNN.
            printf("[OK] 0x%X: 1NNN\n", op);
            PC = op_nibbles;
            break;
        case 0x2:
            // 0x2NNN - Call subroutine at NNN, interpreter increments the stack pointer,
            // and puts the current PC on the top of the stack, the PC is then set to NNN
            printf("[OK] 0x%X: 2NNN\n", op);
            stack_idx++;
            stack[stack_idx] = PC;
            PC = op_nibbles;
            printf("PC: 0x%X; stack_idx: %d; value @ stack_idx: 0x%X\n", PC, stack_idx, stack[stack_idx]);
            break;
        case 0x3:
            // 0x3XNN, skips the next instruction if VX = NN;
            printf("[OK] 0x%X: 3NNN\n", op);
            if (V[X] == (op_nibbles & 0x0FF)) {
                PC += 2;
            }
            PC += 2;
            break;
        case 0x4:
            // 0x4XNN, skips the instruction if VX != NN;
            printf("[OK] 0x%X: 4NNN\n", op);
            if (V[X] !=  (op_nibbles & 0x0FF)) {
                PC += 2;
            }
            PC += 2;
            break;
        case 0x5:
            // 0x5XY0, skip the next instruction if VX = VY
            printf("[OK] 0x%X: 5XY0\n", op);
            if (V[X] == V[Y]) {
                PC += 2;
            }
            PC += 2;
            break;
        case 0x6:
            // 0x6XNN, sets V[X] to NN
            printf("[OK] 0x%X: 6NNN\n", op);
            V[X] = (op_nibbles & 0x0FF);
            PC += 2;
            break;
        case 0x7:
            printf("[OK] 0x%X: 7NNN\n", op);
            // 0x7XNN, sets VX to value at VX + NN;
            V[X] += (op_nibbles & 0x0FF);
            PC += 2;
            break;
        case 0x8:
            // 0x8XYZ, last nibble has different operators so break this 
            // section down some more with a sub switch statement.
            switch (op_nibbles & 0x00F) {
                case 0x0:
                    // 0x8XY0: Set VX to VY 
                    printf("[OK] 0x%X: 8XY0\n", op);
                    V[X] = V[Y];
                    PC += 2;
                    break;
                case 0x1:
                    // 0x8XY1: set VX to VX OR VY; do bitwise OR on the registers
                    printf("[OK] 0x%X: 8XY1\n", op);
                    V[X] = V[X] | V[Y];
                    V[15] = 0;
                    PC += 2;
                    break;
                case 0x2:
                    // 0x8XY2: set VX to VX AND VY; do bitwise AND;
                    printf("[OK] 0x%X: 8XY2\n", op);
                    V[X] = V[X] & V[Y];
                    V[15] = 0;
                    PC += 2;
                    break;
                case 0x3:
                    // 0x8XY3: set VX to VX XOR VY; do bitwise XOR;
                    printf("[OK] 0x%X: 8XY3\n", op);
                    V[X] = V[X] ^ V[Y];
                    V[15] = 0;
                    PC += 2;
                    break;
                case 0x4: { // Include the braces to be able to define reg_sum
                    // 0x8XY4: set VX to VX + VY; use VF as carry if result is more than 255;
                    // VF set to 1 in that case, otherwise 0 and only lowest 8 bits are 
                    // kept and stored in VX;
                    printf("[OK] 0x%X: 8XY4\n", op);
                    uint16_t sum = V[X] + V[Y];
                    if (sum > 255) {
                        V[0xF] = 1;
                    } else {
                        V[0xF] = 0;
                    }
                    // Do bitwise and to keep the lower 8 bits
                    V[X] = sum & 0xFF;

                    if (X == 0xF) {
                        if (sum > 255) {
                            V[0xF] = 1;
                        } else {
                            V[0xF] = 0;
                        }
                    }

                    PC += 2;
                    break;
                }
                case 0x5: 
                    // 0x8XY5: Set VX to VX - VY, VF set to NOT Borrow;
                    // if VX > VY then VF = 1, 0 otherwise;
                    printf("[OK] 0x%X: 8XY5\n", op);
                    uint8_t diff = V[X] - V[Y];

                    if (V[X] >= V[Y]) {
                        V[0xF] = 1;
                    } else {
                        V[0xF] = 0;
                    }

                    if (X != 0xF) {
                        V[X] = diff;
                    }

                    PC += 2;
                    break;
                case 0x6:
                    // Place the value of V[Y] into V[X], shift the value in V[X] 1
                    // bit to the right, store the shifted bit into V[F]
                    printf("[OK] 0x%X: 8XY6\n", op);
                    V[X] = V[Y];
                    int shifted_bit = V[X] & 0b00000001;
                    V[X] /= 2;
                    if (shifted_bit) { // Bit mask to see if last bit is 1;
                        V[0xF] = 1;
                    } else {
                        V[0xF] = 0;
                    }
                    PC += 2;
                    break;
                case 0x7:
                    // 0x8XY7: set VX to VY - VX, if VY > VX then VF = 1; 0 otherwise
                    printf("[OK] 0x%X: 8XY7\n", op);
                    uint8_t diff2 = V[Y] - V[X];

                    if (V[Y] >= V[X]) {
                        V[0xF] = 1;
                    } else {
                        V[0xF] = 0;
                    }

                    if (X != 0xF) {
                        V[X] = diff2;
                    }

                    PC += 2;
                    break;
                case 0xE:
                    // 0x8XYE: If the most significant bit of VX is 1, then VF is set to 1
                    // otherwise it's set to 0, then V[X] is multiplied by 2;
                    printf("[OK] 0x%X: 8XYE\n", op);
                    V[X] = V[Y];
                    int shifted_bit2 = V[X] & 0b10000000;
                    V[X] *= 2;
                    if (shifted_bit2) {
                        V[0xF] = 1;
                    } else {
                        V[0xF] = 0;
                    }
                    PC += 2;
                    break;
                default:
                    printf("[ERROR] Some other instruction in the 0x8XYZ that is not implmented was called: %X\n", op);
                    break;
            }
            break;
        case 0x9:
            // 0x9XY0: Skip the next instruction if VX != VY;
            printf("[OK] 0x%X: 9XY0\n", op);
            if (((op_nibbles & 0x00F) == 0) && V[X] != V[Y]) {
                PC += 2;
            }
            PC += 2;
            break;
        case 0xA:
            // 0xANNN: Set special register I to NNN;
            printf("[OK] 0x%X: ANNN\n", op);
            I = op_nibbles;
            PC += 2;
            break;
        case 0xB:
            // 0xBNNN: set PC to NNN + V0;
            printf("[OK] 0x%X: BNNN\n", op);
            PC = V[0] + op_nibbles;
            break;
        case 0xC:
            // 0xCXNN: Set V[X] to a random byte and NN, ie generate a rand int
            // from 0 to 255 and then do bitwise AND with NN and store it in V[X]
            printf("[OK] 0x%X: CXNN\n", op);
            srand(time(NULL));
            int r = rand() % 255;
            V[X] = r & (op_nibbles & 0x0FF);
            PC += 2;
            break;
        case 0xD: {
            // 0xDXYN: display a sprite starting at memory location I at (VX, VY),
            // use VF for collision bool, Sprites that are read in are XORed onto the display
            // if any pixels are erased because of this, VF is set to 1, otherwise to 0.
            // if the sprite is cut off, it should wrap around the screen to the opposite side
            
            // Start by isolating the last nibble since that will be the size of the sprite
            // being displayed.
            printf("[OK] 0x%X: DXYN\n", op);
            uint8_t n_bytes = op_nibbles & 0x00F;
            // Get X and Y coords from VX and VY;
            uint8_t x_coord = V[X] % SCREEN_WIDTH; // modulo to 'wrap' around in case sprite is too big
            uint8_t y_coord = V[Y] % SCREEN_HEIGHT; // same reasoning for modulo here.
            uint8_t combined_display_idx = x_coord + (y_coord * SCREEN_WIDTH); // display is a 1D array
            unsigned short curr_sprite;
            // reset V[0xF] to 0 before beginning
            V[0xF] = 0;
            // Draw the sprite
            // outer loop is looping over the height of the sprite, n_bytes represents the height,
            // inner loop is over the width, each row of the sprite is 8 bits since it is represented
            // by one byte, at least that's my understanding of it.
            for (int nth_byte = 0; nth_byte < n_bytes; nth_byte++) {
                curr_sprite = memory[I + nth_byte];
                // loop over the bits from the byte grabbed earlier;
                if (nth_byte + y_coord >= SCREEN_HEIGHT) {
                    continue;
                }
                for (int nth_bit = 0; nth_bit < 8; nth_bit++) {
                    if ((curr_sprite & (0x80 >> nth_bit)) != 0) {
                        if (nth_bit + x_coord >= SCREEN_WIDTH) {
                            continue;
                        }
                        // Check if there is a pixel that is already on that will be switched off
                        if (display[(x_coord + nth_bit + ((y_coord + nth_byte) * SCREEN_WIDTH))] == 1) {
                            // Set the collision flag to 1 if theres a pixel already on that will
                            // be shut off.
                            V[0xF] = 1;
                        }
                        // Set display with XOR, what if it needs to clip? There should almost certainly
                        // require a modulo operation
                        display[x_coord + nth_bit + ((y_coord + nth_byte) * SCREEN_WIDTH)] ^= 1;
                    }
                }
            }
            // printf("right before printing display from DXYN");
            // print_arrays(display, (sizeof(display)/sizeof(display[0])));
            // printf("right after printing display from DXYN");
            // draw_on_screen(display);
            draw_flag = 1;
            PC += 2;
            return true;
        }   break;
        case 0xE:
            // two different instructions, 0xEX9E and 0xEXA1;
            switch (op_nibbles & 0x0FF) {
                case 0x9E:
                    // 0xEX9E: skip instruction if the key with the value of VX is pressed
                    printf("[OK] 0x%X: EX9E\n", op);
                    if (keypad[V[X]]) {
                        PC += 2;
                    }
                    PC += 2;
                    break;
                case 0xA1:
                    // 0xEXA1: skip instruction if key with value of VX is NOT pressed
                    printf("[OK] 0x%X: EXA1\n", op);
                    if (!keypad[V[X]]) {
                        PC += 2;
                    }
                    PC += 2;
                    break;
                default:
                    // shouldn't get here
                    printf("[ERROR] Unknown op of 0xEXXX: 0x%X\n", op);
            }
            break;
        case 0xF:
            // Couple of instructions in this opcode type;
            switch(op_nibbles & 0x0FF) {
                case 0x07:
                    // 0xFX07: set VX to the value of the delay timer;
                    printf("[OK] 0x%X: FX07\n", op);
                    V[X] = delay_timer;
                    PC += 2;
                    break;
                case 0x0A:
                    // 0xFX0A: Wait for a keypress, then store the value in VX
                    // All execution should stop until a key is pressed. Done by not incrementing
                    // PC until a keypress is found.
                    printf("[OK] 0x%X: FX0A\n", op);

                    if (key_found && (key_pressed != 255)) {
                        if (!keypad[key_pressed]) {
                            key_found = false;
                            key_pressed = 255;
                            PC += 2;
                        }
                    }

                    for (int i = 0; i < 16; i++) {
                        if (keypad[i]) {
                            V[X] = i;
                            key_found = true;
                            key_pressed = i;
                            break;
                        }
                    }

                    break;
                case 0x15:
                    // 0xFX15: opposite of 0xFX07 where this time delay timer is set to value of VX;
                    printf("[OK] 0x%X: FX15\n", op);
                    delay_timer = V[X];
                    PC += 2;
                    break;
                case 0x18: 
                    // 0xFX18: set sound timer to VX
                    printf("[OK] 0x%X: FX18\n", op);
                    sound_timer = V[X];
                    PC += 2;
                    break;
                case 0x1E:
                    // 0xFX1E: set I = I + VX;
                    printf("[OK] 0x%X: FX1E\n", op);
                    I += V[X];
                    PC += 2;
                    break;
                case 0x29:
                    printf("[OK] 0x%X: FX29\n", op);
                    // 0xFX29: set I = location of sprite for digit VX
                    // TODO: Don't think this is right.
                    I = V[X] * 5;
                    PC += 2;
                    break;
                case 0x33: {
                    // 0xFX33: Store BCD representation of VX in mem locations I, I + 1, I + 2;
                    // take the decimal value of VX, placing hundreds digit at I, tens at I + 1, ones at I + 2;
                    printf("[OK] 0x%X: FX33\n", op);
                    unsigned char vx_value = V[X];
                    unsigned char hundreds = vx_value / 100;
                    unsigned char tens = (vx_value % 100) / 10;
                    unsigned char ones = vx_value % 10;

                    printf("VX value: %d\n X value: %X\n", vx_value, X);
                    memory[I] = hundreds;
                    memory[I + 1] = tens;
                    memory[I + 2] = ones;
                    printf("hundreds value: %d\n", hundreds);
                    printf("value at mem[I] (hundreds): %d\n", memory[I]);
                    printf("tens value: %d\n", tens);
                    printf("value at mem[I + 1] (tens): %d\n", memory[I + 1]);
                    printf("ones value: %d\n", ones);
                    printf("value at mem[I + 2] (ones): %d\n", memory[I + 2]);


                    PC += 2;
                    
                } break; 
                case 0x55:
                    // 0xFX55: Store registers V0-VX in memory start at location I;
                    // Copy the values from the registers into memory starting at I
                    printf("[OK] 0x%X: FX55\n", op);
                    printf("value of X is: %d\n", X);
                    for (int i = 0; i <= X; i++) {
                        memory[I + i] = V[i];
                        printf("value of i is: %d\n", i);
                        printf("value of V[i] is: %d\n value of memory[I + i] is: %d\n", V[i], memory[I + i]);
                    }
                    I += 15;
                    PC += 2;
                    break;
                case 0x65:
                    // 0xFX65: Read registers V0-VX from memory starting at location I;
                    // Read values from memory into the registers.
                    printf("[OK] 0x%X: FX65\n", op);
                    for (int i = 0; i <= X; i++) {
                        V[i] = memory[I + i];
                    }
                    I += 15;
                    PC += 2;
                    break;
            }
            break;
        default:
            error("[ERROR] Unknown opcode encountered: 0x%X\n", op);
    }
    return false;
}

///////////////////////////////
// EMULATION CYCLE HANDLER   //
///////////////////////////////

int main(int argc, char** argv) {
    if (argc != 2) {
        perror("Usage: emulator rom.ch8");
        return 1;
    }

    printf("[PENDING] Initializing CHIP-8 interpreter\n");
    init_cpu();
    printf("[OK] Done!");

    char* ptr;

    char* rom = argv[1];
    // int CYCLE_LEN = strtol(argv[1], &ptr, 10); // Recommended is around ~1400
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

    printf("[OK] Rom loaded successfully!\n");

    init_sdl_display();
    printf("[OK] Display initialized\n");

    const uint32_t frame_time = 16;
    const uint32_t timer_freq = 60;
    uint32_t last_draw_time = 0;
    uint32_t last_timer_update = 0;
    uint32_t current_time = SDL_GetTicks();

    while (!should_quit) {
        if (current_time - last_timer_update >= 1000 / timer_freq) {
            bool draw_occurred = false;

            for (int i = 0; i < 16; i++) {
                draw_occurred = emulate_cycle();
                if (draw_occurred) break;
            }

            if (draw_flag) {
                draw_on_screen(display);
                draw_flag = 0;
                
                last_draw_time = current_time;
            }

            // Decrement the timers if needed:
            if (delay_timer > 0) {
                delay_timer -= 1;
                printf("delay_timer: %i\n", delay_timer);
            }
            if (sound_timer > 0){
                sound_timer -= 1;
                printf("sound_timer: %i\n", sound_timer);
            }

            uint32_t time_since_last_draw = current_time - last_draw_time;
            if (time_since_last_draw < frame_time) {
                SDL_Delay(frame_time - time_since_last_draw);
            }

            sdl_handler(keypad);
        }

    }

    stop_display();
    return 0;

}


void print_arrays(unsigned char* given_array, int array_size){
    for (int i = 0; i < array_size; ++i){
        printf("%dth idx value: %d\n", i, given_array[i]);
    }
}
