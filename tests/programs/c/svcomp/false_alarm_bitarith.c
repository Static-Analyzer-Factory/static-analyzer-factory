// unreach-call, expected verdict: TRUE (a byte-masked value is always <= 255,
// so `(x & 0xFF) == 256` can never hold and reach_error is unreachable).
//
// The Z3 path engine over-approximates: it models `x & 0xFF` as a fresh,
// unconstrained variable (it does not model the bitwise-AND), so it PROPOSES
// this as a FALSE candidate. Slice-1c concrete replay pins the nondet input to
// the model (which constrains the masked value, not x, so x defaults to 0),
// runs the real program where `0 & 0xFF == 0 != 256`, never reaches
// reach_error, and `saf verify` soundly reports `unknown`. Shape of the
// integerpromotion-2 blind false alarm.
extern void reach_error(void);
extern int __VERIFIER_nondet_int(void);

int main(void) {
    int x = __VERIFIER_nondet_int();
    if ((x & 0xFF) == 256) {
        reach_error();
    }
    return 0;
}
