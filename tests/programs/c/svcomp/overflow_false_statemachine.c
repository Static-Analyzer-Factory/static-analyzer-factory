// plans/213 -- COMPLETENESS GATE regression. `no-overflow` FALSE.
//
// A three-state machine, reduced from `openssl-simplified/s3_srvr_1a.cil` (a CIL
// `while(1)` dispatch chain). `n` is `__VERIFIER_nondet_int()`, so `n + 1` can
// overflow and the ONLY correct sentinel answers are `ABSTAIN` (the operand is
// TOP) or a FALSE verdict -- never PROVE.
//
// SAF used to PROVE it. The interval fixpoint under-approximates the state
// variable's loop-head phi (it converges to the first two states and never admits
// 8512), so the dispatch arm that guards the arithmetic is refuted, the block is
// never visited, and `state_at_inst` returns None for the `add`. The sentinel's
// `None => continue` rule then skipped its ONLY obligation and returned `Proven`
// having checked NOTHING.
//
// The magnitude matters: the identical program with states 0/1/2 abstains
// correctly, so this is a widening-threshold artifact. The gate makes the
// sentinel fail closed on it regardless of the underlying imprecision.
extern int __VERIFIER_nondet_int(void);

int main(void) {
  int s = 8466;
  int n = __VERIFIER_nondet_int();
  while (1) {
    if (s == 8466) {
      s = 8496;
    } else if (s == 8496) {
      s = 8512;
    } else if (s == 8512) {
      int y = n + 1;
      return y;
    } else {
      return 0;
    }
  }
}
