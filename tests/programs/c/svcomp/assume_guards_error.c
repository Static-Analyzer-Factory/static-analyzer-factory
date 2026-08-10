// unreach-call, expected verdict: TRUE (the assume makes the error infeasible).
// `saf verify` must NOT emit `false`; it prints `unknown`.
//
// __VERIFIER_assume(x == 0) constrains x, so the guard `x != 0` can never hold
// on a feasible path: `x == 0 && x != 0` is UNSAT. If the assume were ignored
// (a soundness bug for a bug-finder), this would be a false alarm.
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);
extern void __VERIFIER_assume(int);

int main(void) {
    int x = __VERIFIER_nondet_int();
    __VERIFIER_assume(x == 0);
    if (x != 0) {
        reach_error();
    }
    return 0;
}
