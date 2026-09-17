/* plans/213 mechanism M4 -- a wrong TRUE that reached production.
 *
 * A four-state dispatch chain. The final `else` arm is REACHABLE with
 * s == INT_MAX, so `s * 2` overflows: this is ground-truth FALSE, and
 * `saf verify --property no-overflow` answered `true` on it.
 *
 * The cause was in SCCP, not in the interval domain. `mark_edge_executable`
 * scheduled a block only when the BLOCK became newly executable, never when a
 * new EDGE reached an already-executable block -- so the join phi carrying `s`
 * was evaluated once, before the deepest arm's edge existed, and FROZE at
 * [8466, 8512]. SCCP published that singleton, the interval solver adopted it
 * through `constant_map`, and the sentinel proved an `INT_MAX * 2` it never saw.
 *
 * Arity four is not incidental: it is the exact depth at which the mem2reg
 * join-phi chain puts an all-literal phi where SCCP freezes it. n<=3 has no such
 * phi and n>=5 widens to TOP first, which is why both were already correct and
 * why this must be pinned at FOUR states specifically.
 */
extern void abort(void);

int main() {
  int s = 8466;
  int r = 0;
  while (1) {
    if (s == 8466) {
      s = 8496;
    } else if (s == 8496) {
      s = 8512;
    } else if (s == 8512) {
      s = 2147483647;
    } else {
      r = s * 2; /* signed overflow when s == INT_MAX */
      return r;
    }
  }
}
