// GENUINE race: main spawns 2 threads writing a shared global unsynchronized.
// Expect: unknown (reachable spawn -> abstain). A `true` here would be -32.
#include <pthread.h>
int g;
void *w(void *a) { g = g + 1; return 0; }
int main(void) {
  pthread_t t1, t2;
  pthread_create(&t1, 0, w, 0);
  pthread_create(&t2, 0, w, 0);
  pthread_join(t1, 0); pthread_join(t2, 0);
  return g;
}
