// Test fixture for local variable name extraction from debug info.
// Compiled with: clang -S -emit-llvm -g -O0
//
// Exercises:
// - Multiple local variables with different types
// - Function parameters (already handled by AirParam.name)
// - Multiple functions
#include <stdlib.h>

int add(int a, int b) {
    int result = a + b;
    return result;
}

int main(void) {
    int x = 10;
    int y = 20;
    int *ptr = (int *)malloc(sizeof(int));
    int sum = add(x, y);
    *ptr = sum;
    free(ptr);
    return 0;
}
