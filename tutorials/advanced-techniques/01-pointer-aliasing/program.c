// Pointer aliasing basics
//
// This program demonstrates fundamental pointer analysis concepts:
// - Address-of creates a points-to relation
// - Copy propagation creates alias relations
// - PTA can determine which pointers may or may not alias
//
// Through PTA:
//   points_to(p) = {x}, points_to(q) = {x}, points_to(r) = {y}, points_to(s) = {x}
//   may_alias(p, q) = true  (both point to x)
//   may_alias(p, s) = true  (s copies p, so also points to x)
//   no_alias(p, r)  = true  (p -> x, r -> y, disjoint)
//   no_alias(q, r)  = true  (q -> x, r -> y, disjoint)

#include <stdio.h>

int main(void) {
    int x = 10;
    int y = 20;
    int z = 30;

    int *p = &x;      // p points to x
    int *q = &x;      // q also points to x -- aliases p
    int *r = &y;      // r points to y -- does NOT alias p
    int *s = p;        // s copies p -- aliases both p and q

    printf("x=%d y=%d z=%d\n", *p, *r, z);

    // Modify through aliased pointer
    *s = 42;
    printf("x after *s=42: %d\n", x);  // x is now 42

    // Modify through non-aliased pointer
    *r = 99;
    printf("y after *r=99: %d\n", y);   // y is now 99
    printf("x unchanged: %d\n", x);     // x still 42

    return 0;
}
