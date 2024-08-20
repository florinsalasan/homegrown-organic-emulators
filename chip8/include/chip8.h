#ifndef CHIP8_H_
#define CHIP8_H_

void init_cpu(void);
int load_rom(char* filename);
bool emulate_cycle(void);

void init_sdl_display();
void draw(unsigned char* display);
void sdl_handler(unsigned char* keypad);
void stop_display();
void print_arrays(unsigned char* given_array, int array_size);

#define error(...) fprintf(stderr, __VA_ARGS__)

#endif
