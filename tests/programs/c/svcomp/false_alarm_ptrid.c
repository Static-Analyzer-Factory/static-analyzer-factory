// unreach-call, expected verdict: TRUE (two distinct objects never share an
// address, so `p == q` is impossible and reach_error is unreachable).
//
// The Z3 path engine models the pointer operands of `p == q` as fresh, freely
// equal integers (it does not model pointer identity / object separation), so
// it PROPOSES this as a FALSE candidate. Slice-1c concrete replay runs the real
// program where `&a != &b`, the guard is false, reach_error is never executed,
// and `saf verify` soundly reports `unknown`. Shape of the test01 pointer-
// identity blind false alarm.
extern void reach_error(void);

int main(void) {
    int a = 0;
    int b = 0;
    int *p = &a;
    int *q = &b;
    if (p == q) {
        reach_error();
    }
    return 0;
}
