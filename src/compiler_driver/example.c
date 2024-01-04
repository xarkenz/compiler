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
    int x = 3, y = 5;
    bool b;
    b = x == 3 && y == 5;
    b = x > 2 || y > 8;
    b = x < 4 && (y < 2 || y > 3);
    int64_t test_b = 34;
    ((struct MyStruct) {12, test_b, 56}).a = 5;
    return 0;
}