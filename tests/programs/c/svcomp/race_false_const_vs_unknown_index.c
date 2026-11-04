// Reduced from SV-COMP 2027 goblint-regression/05-lval_ls_14-idxunknown_access.c
// (expected no-data-race FALSE). The thread writes data[4], main writes data[i] for a
// nondet i in [0, 10): the two accesses race when i == 4. SAF's points-to analysis
// gives them different locations of `data` (constant index 4 vs an unknown index), and
// `may_alias` called those disjoint, so the structural race prover answered `true`.
extern int __VERIFIER_nondet_int();
extern void abort(void);
void assume_abort_if_not(int cond) {
  if (!cond) {
    abort();
  }
}

#include <pthread.h>

int data[10];

void *t_fun(void *arg) {
  data[4]++;
  return NULL;
}

int main() {
  int i = __VERIFIER_nondet_int();
  assume_abort_if_not(0 <= i && i < 10);
  pthread_t id;
  pthread_create(&id, NULL, t_fun, NULL);
  data[i]++;
  return 0;
}
