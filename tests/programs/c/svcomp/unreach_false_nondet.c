// unreach-call, expected verdict: FALSE. A nondet value is unconstrained, so
// the guarded reach_error() call is reachable (e.g. x == 6 satisfies x > 5).
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x > 5) {
        reach_error();
    }
    return 0;
}
