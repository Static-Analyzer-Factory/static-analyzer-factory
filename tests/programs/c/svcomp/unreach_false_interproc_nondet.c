// unreach-call, expected verdict: FALSE. `main` reads a nondet and guards on it
// (`x == 42`) before calling a buggy callee whose `reach_error()` is reached only
// when the guard holds. The steering nondet lives in `main` but the error is in
// the callee, so this is missed by BOTH must-reach (the guard is a branch) and
// the intraprocedural candidate enumeration (rooted at the callee, blind to
// main's guard). R4 (plan 196) roots the search at `main`, composes the
// main -> buggy path, pins the steering nondet to 42, and native replay reaches
// the error -> `false(unreach-call)`.
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

void buggy(void) {
    reach_error();
}

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x == 42) {
        buggy();
    }
    return 0;
}
