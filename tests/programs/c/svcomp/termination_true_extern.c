// R7 (plan 201): a loop-free `main` that calls only KNOWN-TERMINATING externals
// (__VERIFIER_nondet_int from the bundled stub, and printf) => `true`.
extern int printf(const char *fmt, ...);

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x > 0) {
        printf("positive\n");
    }
    return 0;
}
