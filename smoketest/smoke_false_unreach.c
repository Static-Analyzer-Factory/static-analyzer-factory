// SAF smoke fixture -- property: unreach-call, expected verdict: FALSE.
//
// `reach_error()` sits behind a nondeterministic guard, so `false` is only
// reachable by running the real pipeline: clang -> LLVM IR -> ingest -> solve
// the guard (`x == 6` satisfies `x > 5`) -> confirm -> emit a witness. That
// makes it a genuine dependency check: with `SAF_CLANG=/definitely/not/clang`
// SAF reports "compilation failed: failed to spawn ... -> unknown" and the
// smoke test fails. The witness pins `\result == 6` for the nondet call, so a
// pass also proves the violation-witness writer works.
//
// Keep this fixture trivially FALSE. Anything harder risks degrading to
// `unknown` on a slow or loaded machine and making the smoke test flaky.
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x > 5) {
        reach_error();
    }
    return 0;
}
