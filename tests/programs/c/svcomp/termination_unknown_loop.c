// R7 (plan 201): a reachable loop whose bound is nondeterministic — proving it
// terminates needs a ranking function (out of scope), so R7 abstains => `unknown`
// (never `true`, never `false`).
int main(void) {
    int n = __VERIFIER_nondet_int();
    int s = 0;
    int i = 0;
    while (i < n) {
        s += i;
        i++;
    }
    return s;
}
