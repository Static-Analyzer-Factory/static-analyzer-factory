// Interval no-overflow sentinel regression: a store through another pointer to an address-taken local.
int main(void) { int x = 0x7ffffe00; unsigned char *p = (unsigned char *)&x; p[1] = 0xff; int y = x + 0x100; return y; }
