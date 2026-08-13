// pthread_create reached via a function pointer (indirect call). Expect: unknown
// (gate abstains on reachable CallIndirect while a spawn symbol is linked).
#include <pthread.h>
int g;
void *w(void *a) { g++; return 0; }
typedef int (*create_fn)(pthread_t*, const pthread_attr_t*, void*(*)(void*), void*);
int main(void) {
  volatile create_fn f = pthread_create;
  pthread_t t;
  f(&t, 0, w, 0);
  return g;
}
