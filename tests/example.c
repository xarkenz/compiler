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

bool func_a() {
    return true;
}

bool func_b() {
    return false;
}

int32_t main() {
    do_call(my_func, 8);
    bool b1 = func_a() && func_b() && func_a();
    bool b2 = func_a() || func_b();
    if (func_a() && func_b()) {
        my_func(1);
    }
    if (func_a() || func_b()) {
        my_func(2);
    }
    return 0;
}
