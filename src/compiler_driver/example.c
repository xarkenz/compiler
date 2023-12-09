#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>


int32_t main() {
    int x = 3, y = 5;
    bool b;
    b = x == 3 && y == 5;
    b = x > 2 || y > 8;
    b = x < 4 && (y < 2 || y > 3);
    return 0;
}