/**
 * leak_basic.c — Simple memory leak: malloc in main, never freed.
 *
 * The pointer p is allocated and used but never freed before main returns.
 * The value flow reaches main's exit without a sanitizer (free).
 *
 * Expected: memory-leak finding reported for the malloc in main().
 *
 * Compile:
 *   clang-18 -S -emit-llvm -O0 -g leak_basic.c -o leak_basic.ll
 */

#include <stdlib.h>

int main(void) {
    int *p = (int *)malloc(sizeof(int));
    *p = 42;
    /* p is never freed — leak at program exit */
    return *p;
}
