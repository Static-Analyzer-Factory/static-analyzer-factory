// Interval no-overflow sentinel regression: a store through another pointer to an address-taken local.
extern int __VERIFIER_nondet_int(void);
int *gp; int main(void) { int x = 5; gp = &x; *gp = 2147483647; int r = x + 1; return r; }
