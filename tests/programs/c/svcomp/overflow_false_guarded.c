// no-overflow FALSE, mini-fuzz-guarded: `x + 1` overflows only when the nondet input
// is INT_MAX. The zeroed probe (0) does not trigger it, but OVERFLOW_CONSTS includes
// 2147483647, so the multi-constant mini-fuzz reproduces it -> false(no-overflow).
// Sound: INT_MAX is a valid concrete input the verifier may choose.
extern int __VERIFIER_nondet_int(void);
int main(void) {
    int x = __VERIFIER_nondet_int();
    return x + 1;
}
