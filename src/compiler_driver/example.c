#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

void my_func(int32_t x) {
    printf("%d\n", x);
}

void do_call(void(*pointer)(int32_t), int32_t value) {
    pointer(value);
}

int32_t main() {
    do_call(my_func, 8);
    return 0;
}