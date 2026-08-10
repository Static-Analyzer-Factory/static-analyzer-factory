// unreach-call, expected verdict: TRUE. `f` is only ever called with n >= 0
// (main calls f(5), which recurses down to f(0)), so the `n < 0` guard never
// holds and reach_error is unreachable.
//
// The Z3 path engine analyzes `f` intraprocedurally and sees its parameter `n`
// as unconstrained (it does not compose the caller's argument into the callee),
// so `n < 0` looks satisfiable and it PROPOSES a FALSE candidate. Slice-1c
// concrete replay runs the real, composed call chain f(5)->f(4)->...->f(0),
// where n is never negative, reach_error is never executed, and `saf verify`
// soundly reports `unknown`. Shape of the afterrec-2 uncomposed-recursion-
// argument blind false alarm.
extern void reach_error(void);

void f(int n) {
    if (n < 0) {
        reach_error();
    }
    if (n > 0) {
        f(n - 1);
    }
}

int main(void) {
    f(5);
    return 0;
}
