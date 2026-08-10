// unreach-call, expected verdict: TRUE (reach_error present but unreachable).
// `saf verify` must NOT emit `false`; since we never emit `true`, it prints `unknown`.
//
// The guard `x != x` is a self-contradiction, so the reach_error() call is
// unreachable on every path. The call survives in the IR (x is a nondet value,
// not a compile-time constant), exercising the Z3 infeasibility path.
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x != x) {
        reach_error();
    }
    return 0;
}
