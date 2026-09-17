/* plans/214 Movement 2 -- a pointer that came out of MEMORY is never anchored.
 *
 * `q` holds a pointer read from a global. Whatever it points at, the prover
 * cannot know that object is live or large enough -- and under interference
 * another thread could have written `q` between the load and the store. This is
 * the rule that makes the domain immune to dangling pointers, aliasing and
 * interference.
 *
 * As with `memsafe_use_after_scope.c`, the reported reason is
 * `load-width-unknown`: loading `q` is a pointer-width load and `exact_size_of`
 * declines pointer widths, so the scan stops there and never reaches the
 * dereference. Assert only that it abstains. */
int target;
int *q = &target;

int main(void) {
  int *local = q; /* the pointer comes out of memory */
  *local = 1;
  return target;
}
