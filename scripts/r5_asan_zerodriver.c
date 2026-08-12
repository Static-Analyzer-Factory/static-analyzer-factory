/* Slice 0c — zero-nondet driver for the R5 ASan-gate measurement (measurement only).
 * Defines the SV-COMP __VERIFIER_* symbols the -include stub only declares, so the
 * ORIGINAL task can be compiled+run natively under -fsanitize=address. All nondet
 * inputs return 0/NULL (the "unsteered / default input" probe); __VERIFIER_assume
 * prunes assumed-false paths at runtime (so a benchmark that guards its inputs cannot
 * be driven into a spurious fault). Does NOT override malloc/free/memcpy (ASan
 * intercepts them). reach_error/__VERIFIER_error are weak no-ops (memsafety faults are
 * intrinsic; we only count AddressSanitizer reports). Mirrors synthesize_driver's
 * nondet half without the reach_error sentinel. */
#include <stddef.h>
#include <unistd.h>

int                __VERIFIER_nondet_int(void)        { return 0; }
unsigned int       __VERIFIER_nondet_uint(void)       { return 0u; }
long               __VERIFIER_nondet_long(void)       { return 0; }
unsigned long      __VERIFIER_nondet_ulong(void)      { return 0ul; }
long long          __VERIFIER_nondet_longlong(void)   { return 0; }
unsigned long long __VERIFIER_nondet_ulonglong(void)  { return 0ull; }
short              __VERIFIER_nondet_short(void)      { return 0; }
unsigned short     __VERIFIER_nondet_ushort(void)     { return 0; }
char               __VERIFIER_nondet_char(void)       { return 0; }
unsigned char      __VERIFIER_nondet_uchar(void)      { return 0; }
_Bool              __VERIFIER_nondet_bool(void)       { return 0; }
void              *__VERIFIER_nondet_pointer(void)    { return (void *)0; }
float              __VERIFIER_nondet_float(void)      { return 0.0f; }
double             __VERIFIER_nondet_double(void)     { return 0.0; }
size_t             __VERIFIER_nondet_size_t(void)     { return 0; }
/* common extra spellings seen across sv-benchmarks (reduce link-fail exclusions) */
unsigned           __VERIFIER_nondet_unsigned(void)   { return 0u; }
unsigned char      __VERIFIER_nondet_U8(void)         { return 0; }
unsigned short     __VERIFIER_nondet_U16(void)        { return 0; }
unsigned int       __VERIFIER_nondet_U32(void)        { return 0u; }
char               __VERIFIER_nondet_S8(void)         { return 0; }
short              __VERIFIER_nondet_S16(void)        { return 0; }
int                __VERIFIER_nondet_S32(void)        { return 0; }
long               __VERIFIER_nondet_loff_t(void)     { return 0; }

void __VERIFIER_assume(int c) { if (!c) _exit(0); }
void __VERIFIER_atomic_begin(void) {}
void __VERIFIER_atomic_end(void) {}

__attribute__((weak)) void reach_error(void) {}
__attribute__((weak)) void __VERIFIER_error(void) {}
