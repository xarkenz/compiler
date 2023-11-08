#include <stdio.h>

int main() {
    int a = 0;
    int b = 1;

    while (1) {
        if (b >= 1000) {
            break;
        }

        printf("%lld\n", (long long) b);
        int temp = a + b;
        a = b;
        b = temp;
    }

    return 0;
}