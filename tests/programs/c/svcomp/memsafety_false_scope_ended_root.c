/* The only pointer to the block lives in an array whose scope has ended
   before exit(): the block is unreachable at exit, a valid-memtrack
   violation. The prover must never say true. */
typedef __SIZE_TYPE__ size_t;
extern void *malloc(size_t);
extern void exit(int);
int __VERIFIER_nondet_int();
int main() {
  {
    int *keep[2];
    keep[0] = malloc(sizeof(int));
    keep[1] = 0;
    if (!keep[0]) return 0;
    *keep[0] = __VERIFIER_nondet_int();
  }
  exit(0);
}
