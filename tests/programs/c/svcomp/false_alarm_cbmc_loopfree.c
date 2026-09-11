// unreach-call, expected verdict: TRUE under the C standard (the only path to
// reach_error goes through a signed-integer overflow, which is undefined
// behaviour, so no defined execution reaches it). `saf verify` must NOT emit
// `false`; it prints `unknown`.
//
// The point of the fixture is the LOOP-FREE CBMC-oracle path (lever
// `cbmc-solve`): `cbmc_precheck` admits acyclic programs, and CBMC runs with
// `--no-standard-checks`, so it models `x + y` as wrapping two's-complement and
// PROPOSES the overflowing vector as a counterexample. The native replay is the
// sole arbiter (R6): it compiles with `-fsanitize-trap=signed-integer-overflow`,
// so the pinned inputs trap at the addition and never reach the sentinel, and
// the verdict stays `unknown`. If the replay gate were ever short-circuited for
// the CBMC lever, this fixture would turn into a false alarm.
extern void __assert_fail(const char *, const char *, unsigned int, const char *)
    __attribute__((__noreturn__));
extern int __VERIFIER_nondet_int(void);

void reach_error(void) {
    __assert_fail("0", "false_alarm_cbmc_loopfree.c", 3, "reach_error");
}

int main(void) {
    int x = __VERIFIER_nondet_int();
    int y = __VERIFIER_nondet_int();
    if (x > 2000000000 && y > 2000000000) {
        if (x + y < 0) {
            reach_error();
        }
    }
    return 0;
}
