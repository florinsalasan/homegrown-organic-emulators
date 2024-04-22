#include <stdbool.h>
#include "../include/sdl_layer.h"

#include <SDL2/SDL.h>

SDL_Window* screen;

SDL_Renderer* renderer;

SDL_Scancode keymappings[16] = {
    SDL_SCANCODE_1, SDL_SCANCODE_2, SDL_SCANCODE_3, SDL_SCANCODE_4,
    SDL_SCANCODE_Q, SDL_SCANCODE_W, SDL_SCANCODE_E, SDL_SCANCODE_R,
    SDL_SCANCODE_A, SDL_SCANCODE_S, SDL_SCANCODE_D, SDL_SCANCODE_F,
    SDL_SCANCODE_Z, SDL_SCANCODE_X, SDL_SCANCODE_C, SDL_SCANCODE_V,
};

bool should_quit = false;

// Display initialization
void init_display(void) {
    SDL_Init(SDL_INIT_VIDEO);

    screen = SDL_CreateWindow("CHIP-8", SDL_WINDOWPOS_CENTERED,
                              SDL_WINDOWPOS_CENTERED, 64 * 8, 32 * 8, 0);
    renderer = SDL_CreateRenderer(screen, -1, SDL_RENDERER_ACCELERATED);
}

// draw on sdl display
void draw(unsigned char* display) {
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);

    // clear the display
    SDL_RenderClear(renderer);

    SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);

    // Iterate through the display (64*32)

}
