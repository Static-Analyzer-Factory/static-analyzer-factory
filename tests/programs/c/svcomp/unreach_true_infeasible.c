// unreach-call, expected verdict: TRUE (rank-3 sound-TRUE happy path).
//
// `x > 0 && x < 0` is a contradiction over a nondet value, so the interval
// abstract interpreter refines the inner `then` edge to bottom and proves the
// `reach_error()` block UNREACHABLE; the in-process CPAchecker confirmation gate
// agrees, so `saf verify` prints `true`. The comparisons are over a nondet value
// (not a compile-time constant), so the guards survive as real `icmp`s that the
// interval refinement can prune (unlike a constant-folded `if (0)`).
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x > 0) {
        if (x < 0) {
            reach_error();
        }
    }
    return 0;
}
