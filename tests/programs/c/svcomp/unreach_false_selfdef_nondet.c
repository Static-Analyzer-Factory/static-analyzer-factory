// unreach-call, expected verdict: FALSE. Like unreach_false_nondet.c, but the
// task DEFINES reach_error itself via __assert_fail — the canonical
// sv-benchmarks pattern (a preprocessed .i almost always carries this body).
//
// Slice-1c concrete replay must not collide with that definition: the driver's
// reach_error is a WEAK symbol (the task's strong one wins the link) and
// __assert_fail is overridden to detect the violation. The Z3 model x = 6
// satisfies x > 5, so the pinned run reaches reach_error -> __assert_fail ->
// sentinel -> false(unreach-call). Regression for the link collision the blind
// eval exposed.
extern void __assert_fail(const char *, const char *, unsigned int, const char *)
    __attribute__((noreturn));
extern int __VERIFIER_nondet_int(void);

void reach_error(void) { __assert_fail("0", "selfdef.c", 15, "reach_error"); }

int main(void) {
    int x = __VERIFIER_nondet_int();
    if (x > 5) {
        reach_error();
    }
    return 0;
}
