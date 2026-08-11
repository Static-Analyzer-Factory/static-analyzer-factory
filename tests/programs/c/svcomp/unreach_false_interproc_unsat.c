// unreach-call, expected verdict: UNKNOWN (soundness negative for R4). The
// interprocedural path main -> buggy exists, but the guard on the steering nondet
// is unsatisfiable (`x > 5 && x < 3`), so the callee's reach_error() is genuinely
// unreachable. R4's joint Z3 model must find the path infeasible (no candidate),
// and even if a candidate were proposed, native replay would not reproduce the
// error -> `unknown`. We NEVER emit `false` here (that would be a -16 false alarm).
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

void buggy(void) {
    reach_error();
}

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x > 5 && x < 3) {
        buggy();
    }
    return 0;
}
