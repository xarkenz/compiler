#include <inttypes.h>
#include <stdbool.h>
#include <stdio.h>

const char my_string_array[] = "assigning string to array";
const char* const my_string_ptr = &"assigning string to pointer"[0];
const uint32_t my_other_array[] = {'h', 'e', 'l', 'l', 'o'};
const uint32_t* const my_other_ptr = &((const uint32_t[5]) {'w', 'o', 'r', 'l', 'd'})[0];

int32_t main() {
    return 0;
}