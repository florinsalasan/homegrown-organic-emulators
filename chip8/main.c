#include <stdio.h>
#include <stdbool.h>

#include <SDL.h>

// Define screen dimensions
#define SCREEN_WIDTH    64
#define SCREEN_HEIGHT   32
#define SDL_SCALING     8

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
}

int should_quit = 0;

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
    SDL_SetTenderDrawColor(renderer, 0, 0, 0, 255); 

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

///////////////////////////////
// EMULATION CYCLE HANDLER   //
///////////////////////////////

int main(int argc, char** argv) {
    if (argc != 2) {
        error("Usage: emulator rom.ch8");
        return 1;
    }
}
