#include <stdio.h>

int main() {
    int x = 5;
    int y;

    if (x > 0) {
        y = 2;
    }
    else if (x < 0) {
        y = 0;
    }
    else {
        y = 1;
    }

    while (y < 10) {
        printf("%d\n", y);
        y = y + 1;
    }

    return 0;
}