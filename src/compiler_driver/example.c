#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

struct MyStruct {
    int32_t a;
    int64_t b;
    int8_t c;
};

struct MyStruct my_global_struct;

int32_t main() {
    struct MyStruct my_uninit_struct;
    my_uninit_struct.c = 5;
    struct MyStruct my_init_struct = { 1, 2, 3 };
    my_global_struct.b = 7;
    int x = 3, y = 5;
    bool b;
    b = x == 3 && y == 5;
    b = x > 2 || y > 8;
    b = x < 4 && (y < 2 || y > 3);
    return 0;
}