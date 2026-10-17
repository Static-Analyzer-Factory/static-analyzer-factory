// Interval no-overflow sentinel regression: a store through another pointer to an address-taken local.
extern int __VERIFIER_nondet_int(void);
union u { int i; unsigned char b[4]; }; int main(void) { union u v; v.i = 0x7ffffe00; v.b[1] = 0xff; int r = v.i + 0x100; return r; }
