// The loop body runs at most once: y = x | (x + 1) > x makes x negative.
// x + 1 overflows for x == INT_MAX; a termination task has no undefined
// behaviour, so that execution does not exist (sv-benchmarks CONVENTIONS.md).
extern int __VERIFIER_nondet_int(void);
int main(void) {
    int x = __VERIFIER_nondet_int();
    int y = __VERIFIER_nondet_int();
    while (x >= y && y > 0) {
        y = x | (x + 1);
        x = x - y;
    }
    return 0;
}
