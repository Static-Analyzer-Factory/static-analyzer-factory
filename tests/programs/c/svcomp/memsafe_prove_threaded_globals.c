/* plans/214 Movement 2 — the POSITIVE case for the Anchored-Object prover.
 *
 * Two threads, a mutex, and four globals. Every dereference is a whole-object
 * access at the base of a global: no `getelementptr`, no cast, no allocation,
 * no libc that touches memory. This is the shape the concurrent clusters
 * (`pthread-wmm`, `weaver`, `pthread`, `goblint-regression`) actually have, and
 * it must PROVE -- including across the thread bodies, which reach the universe
 * only through MTA's PTA-driven thread-entry discovery. */
#include <pthread.h>

int x, y;
pthread_mutex_t m;
pthread_t t1, t2;

void *w1(void *a) {
  pthread_mutex_lock(&m);
  x = 1;
  y = x;
  pthread_mutex_unlock(&m);
  return 0;
}

void *w2(void *a) {
  pthread_mutex_lock(&m);
  x = 2;
  y = x;
  pthread_mutex_unlock(&m);
  return 0;
}

int main(void) {
  pthread_mutex_init(&m, 0);
  pthread_create(&t1, 0, w1, 0);
  pthread_create(&t2, 0, w2, 0);
  pthread_join(t1, 0);
  pthread_join(t2, 0);
  return y;
}
