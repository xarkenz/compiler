#include <stdio.h>

long long print_i64(long long x) {
    printf("%lld\n", x);
    return x;
}

long long unsigned print_u64(long long unsigned x) {
    printf("%llu\n", x);
    return x;
}

void* print_ptr(void* ptr) {
    printf("%p\n", ptr);
    return ptr;
}

int main() {
    int a = 0;
    int b = 1;

    while (1) {
        if (b >= 1000) {
            break;
        }

        print_i64(b);
        int temp = a + b;
        a = b;
        b = temp;
    }

    return 0;
}