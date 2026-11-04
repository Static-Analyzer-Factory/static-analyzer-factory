// GUARD for race_false_const_vs_unknown_index.c: the thread writes `a`, main writes
// `b`. Two distinct objects never overlap, so this program is race-free and the
// structural prover must still prove it after the same-object overlap rule.
#include <pthread.h>

int a[10];
int b[10];

void *t_fun(void *arg) {
  a[4]++;
  return NULL;
}

int main() {
  pthread_t id;
  pthread_create(&id, NULL, t_fun, NULL);
  b[4]++;
  return 0;
}
