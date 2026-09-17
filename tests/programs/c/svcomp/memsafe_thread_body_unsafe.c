/* plans/214 Movement 2 -- pins that THREAD BODIES are really in the universe.
 *
 * `main` itself is spotless. The only unsafe access is inside `w1`, which is
 * never reachable from `main` in the call graph -- `pthread_create` is opaque, so
 * a thread body arrives only via MTA's PTA-driven thread-entry discovery being
 * unioned in as `ThreadRoots::Also`.
 *
 * If that discovery silently returned nothing, the universe would be main's tree
 * alone, `w1` would never be scanned, and the prover would answer TRUE having
 * examined none of the code that actually runs. This fixture is the difference
 * between a proof and a vacuous one. */
#include <pthread.h>

char small;
pthread_t t1;

void *w1(void *a) {
  *(int *)&small = 1; /* 4-byte store into a 1-byte object */
  return 0;
}

int main(void) {
  pthread_create(&t1, 0, w1, 0);
  pthread_join(t1, 0);
  return 0;
}
