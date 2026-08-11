// unreach-call, expected verdict: FALSE. `main` reads a nondet and passes it as
// an ARGUMENT to a callee whose reach_error() is guarded on the PARAMETER
// (`n == 42`). This is "Shape 2": Slice 1 extracts the callee's guard `n == 42`
// but `n` is a formal parameter, not a nondet CALL, so the model constrains `n`
// (not main's `x`) and the nondet stays 0 -> replay leaves x=0 -> f(0) -> no
// error -> UNKNOWN. Confirming this needs arg->param binding (plan 196 Slice 2):
// assert `x == n` so the joint model pins main's nondet to 42.
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

void f(int n) {
    if (n == 42) {
        reach_error();
    }
}

int main(void) {
    int x = __VERIFIER_nondet_int();
    f(x);
    return 0;
}
