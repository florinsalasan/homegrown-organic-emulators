#ifndef CHIP8_SDL_H_
#define CHIP8_SDL_H_

void init_display();
void draw(unsigned char* display);
void sdl_keyboard(unsigned char* keypad);
void stop_display();

#endif
