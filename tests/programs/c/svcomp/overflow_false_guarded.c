// no-overflow FALSE, mini-fuzz-guarded: `x + x` overflows when the nondet input is large
// (>= 2^30). The zeroed probe (0) does not trigger it, but OVERFLOW_CONSTS includes 2^30
// (1073741824), so the multi-constant mini-fuzz reproduces it -> false(no-overflow).
// (`x + 1` would overflow only at INT_MAX, which is indistinguishable in the UBSan report
// from a loop counter/accumulator reaching INT_MAX -- a case SV-COMP labels no-overflow
// TRUE -- so the fuzz uses 2^30, not INT_MAX; see OVERFLOW_CONSTS.)
// Sound: 2^30 is a valid concrete input the verifier may choose.
extern int __VERIFIER_nondet_int(void);
int main(void) {
    int x = __VERIFIER_nondet_int();
    return x + x;
}
