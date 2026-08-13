// OpenMP race, NO pthread. GOMP_parallel/__kmpc_fork_call are NOT in SPAWN_FUNCTIONS.
// Completeness probe: does the current gate wrongly emit true? (Depends on whether
// SAF's compile enables -fopenmp; documents the -32 completeness surface.)
extern int __VERIFIER_nondet_int(void);
int g;
int main(void) {
  int n = __VERIFIER_nondet_int();
  #pragma omp parallel for
  for (int i = 0; i < 8; i++) g += n;
  return g;
}
