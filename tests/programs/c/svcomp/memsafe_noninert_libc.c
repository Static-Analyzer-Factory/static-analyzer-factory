/* plans/214 Movement 2 -- pins OBLIGATION 2 (non-inert libc).
 *
 * Every access this program makes through its OWN instructions is a safe,
 * anchored, whole-object global access, so obligations 1 and 3 are satisfied and
 * the prover would PROVE on the instruction stream alone. The violation is
 * invisible there: `puts` walks `msg` until it finds a NUL, and `msg` has none,
 * so the read runs off the end inside libc.
 *
 * That is the whole point of obligation 2 -- an external that dereferences a
 * caller-supplied pointer is not inert, and admitting it would mean proving a
 * program whose only unsafe access is in code the analysis never saw. Deleting
 * the allowlist (or adding `puts` to it) turns this into a wrong TRUE. */
extern int puts(const char *);

char msg[4] = {'a', 'b', 'c', 'd'}; /* deliberately NOT NUL-terminated */

int main(void) {
  puts(msg);
  return 0;
}
