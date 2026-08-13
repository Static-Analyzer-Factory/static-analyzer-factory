// R7 (plan 201): a reachable indirect call through a nondet-chosen function
// pointer. The direct call graph is incomplete there — the target could hide
// recursion — so R7 abstains conservatively (CLAUDE.md redline #8) => `unknown`.
int f0(void) { return 0; }
int f1(void) { return 1; }

int main(void) {
    int (*fp)(void) = __VERIFIER_nondet_int() ? f0 : f1;
    return fp();
}
