#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>

void print_i64(int64_t x) {
    printf("%ld\n", x);
}

void print_u64(uint64_t x) {
    printf("%lu\n", x);
}

void print_ptr(void* ptr) {
    printf("%p\n", ptr);
}

int32_t fibonacci(int32_t limit) {
    int32_t a = 0;
    int32_t b = 1;

    while (b < limit) {
        int32_t temp = a + b;
        a = b;
        b = temp;
    }

    return a;
}

uint32_t gcd(uint32_t a, uint32_t b) {
    while (b >= 1) {
        uint32_t temp = a % b;
        a = b;
        b = temp;
    }

    return a;
}

int32_t main() {
    bool a = true;
    bool b = false;
    bool c = true;

    bool d = (a && b) || c;

    if ((a && b) || c) {
        print_i64((int64_t) fibonacci(1000));
    }
    else {
        print_u64((uint64_t) gcd(18, 45));
    }

    return 0;
}