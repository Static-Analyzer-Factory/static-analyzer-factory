/* plans/214 Movement 2 -- pins that anchoring is INTRAPROCEDURAL.
 *
 * `&target` is a perfectly good anchor at the call site, but inside `sink` the
 * address is a function PARAMETER: its `ValueId` is not any global's, so the
 * store is unanchored and the prover abstains. The access is in fact safe, which
 * is the point -- this is recall the design gives up deliberately rather than
 * introducing an interprocedural anchor summary, whose soundness would have to
 * be argued separately.
 *
 * It is also real attrition: a program that passes globals to helpers will not
 * prove, which is why the yielding clusters are the ones that operate on globals
 * directly. */
int target;

static int *sink(int *p) {
  *p = 1;
  return p;
}

int main(void) {
  sink(&target);
  return target;
}
