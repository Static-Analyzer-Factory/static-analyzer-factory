// Fixture source for the correctness-witness end-to-end milestone (plan 207).
// The 1a pure-interval program: safety of `i == 1000000` after the loop follows
// from the loop-head interval invariant `0 <= i && i <= 1000000`. The loop guard
// `i < 1000000` is a comparison (NOT a function call), so the driver does not
// abstain, and CPAchecker confirmed this exact invariant in spike 1a.
//
// Compiled with the `saf verify` recipe:
//   clang-18 -g -S -emit-llvm -O0 -Xclang -disable-O0-optnone -Wno-everything -m32 \
//     spike_counter.c -o - | opt-18 -S -passes=mem2reg > .../spike_counter.ll
extern void abort(void);
extern void __assert_fail(const char *, const char *, unsigned int, const char *)
    __attribute__((__nothrow__, __leaf__)) __attribute__((__noreturn__));
void reach_error() { __assert_fail("0", "spike_counter.c", 3, "reach_error"); }
void __VERIFIER_assert(int cond) {
    if (!(cond)) {
    ERROR:
        {
            reach_error();
            abort();
        }
    }
    return;
}
int main(void) {
    unsigned int i = 0;
    while (i < 1000000) {
        i++;
    }
    __VERIFIER_assert(i == 1000000);
    return 0;
}
