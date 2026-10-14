// Wrong-TRUE guard (frontend/engine review 2026-10-14): reach_error is reachable.
#include <fenv.h>
extern void reach_error(void);
int main(void) {
  volatile double x = 1.0;
  fesetround(FE_UPWARD);
  double z = x / 3.0;
  if (z * 3.0 != 1.0) reach_error();
  return 0;
}
