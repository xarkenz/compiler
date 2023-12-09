#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

const char* const my_string = "hello world";

int32_t main() {
    const char x = my_string[5];
    return 0;
}