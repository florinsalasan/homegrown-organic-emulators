#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

const int SCREEN_HEIGHT = 32;
const int SCREEN_WIDTH = 64;

void clear_screen(bool[SCREEN_HEIGHT][SCREEN_WIDTH]);

int main () {

    bool display[64][32];

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
