/* plans/214 Movement 2 -- pins OBLIGATION 3 on the STACK.
 *
 * The mirror of `memsafe_bounds_oob_width.c`, one scope down. `c` is a one-byte
 * LOCAL and the store writes four. Because the store's type does not match the
 * alloca's, mem2reg leaves the slot in place, so the prover sees a direct
 * stack-slot access -- which it is allowed to anchor -- and the ONLY thing
 * standing between it and a wrong TRUE is comparing the 4-byte width against the
 * slot's exact 1-byte size.
 *
 * This is why the alloca size had to come from `ALLOCA_EXACT_SIZE_KEY` rather
 * than `Operation::Alloca { size_bytes }`: the latter reports 8 bytes for every
 * float and pointer, and an over-stated size makes this check pass. */
int main(void) {
  char c;
  *(int *)&c = 1;
  return c;
}
