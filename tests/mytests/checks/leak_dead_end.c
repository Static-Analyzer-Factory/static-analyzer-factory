/**
 * leak_dead_end.c — Memory leak where pointer's value flow dead-ends.
 *
 * The allocation in bar() is never freed, returned, or stored.
 * The pointer's value flow dead-ends inside bar(), never reaching
 * any function exit node in the SVFG. This tests whether the
 * memory-leak checker can detect leaks when the BFS finds no exit.
 *
 * Expected: memory-leak finding reported for the malloc in bar().
 *
 * Compile:
 *   clang-18 -S -emit-llvm -O0 -g leak_dead_end.c -o leak_dead_end.ll
 */

#include <stdlib.h>

void bar(void) {
    void *p = malloc(10);
    /* p is not freed, not returned, not stored anywhere.
     * Its value flow dead-ends here — a clear memory leak. */
}

int main(void) {
    bar();
    return 0;
}
