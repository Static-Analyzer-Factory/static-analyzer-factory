/* Slice 2 de-risk — const-nondet driver for the mini-fuzz (measurement only).
 * Every scalar __VERIFIER_nondet_* returns the SAME constant, chosen at RUNTIME
 * from $SAF_NONDET_CONST (atol) — so one ASan-instrumented binary can be run under
 * many constants without recompiling. Pointer/float nondet stay 0/NULL;
 * __VERIFIER_assume is honoured (a constant that violates an assume exits cleanly).
 * This lower-bounds the scalar-nondet-steerable reservoir: if a buggy task ASan-traps
 * under SOME constant but not 0, a Z3-steered value would (at least) also reach it. */
#include <stddef.h>
#include <stdlib.h>
extern void _exit(int) __attribute__((noreturn));

static long __saf_v(void) {
    const char *e = getenv("SAF_NONDET_CONST");
    return e ? atol(e) : 0;
}

int                __VERIFIER_nondet_int(void)       { return (int)__saf_v(); }
unsigned int       __VERIFIER_nondet_uint(void)      { return (unsigned int)__saf_v(); }
long               __VERIFIER_nondet_long(void)      { return (long)__saf_v(); }
unsigned long      __VERIFIER_nondet_ulong(void)     { return (unsigned long)__saf_v(); }
long long          __VERIFIER_nondet_longlong(void)  { return (long long)__saf_v(); }
unsigned long long __VERIFIER_nondet_ulonglong(void) { return (unsigned long long)__saf_v(); }
short              __VERIFIER_nondet_short(void)      { return (short)__saf_v(); }
unsigned short     __VERIFIER_nondet_ushort(void)     { return (unsigned short)__saf_v(); }
char               __VERIFIER_nondet_char(void)       { return (char)__saf_v(); }
unsigned char      __VERIFIER_nondet_uchar(void)      { return (unsigned char)__saf_v(); }
_Bool              __VERIFIER_nondet_bool(void)       { return (_Bool)(__saf_v() & 1); }
size_t             __VERIFIER_nondet_size_t(void)     { return (size_t)__saf_v(); }
/* common extra spellings across sv-benchmarks (reduce link-fail exclusions) */
unsigned           __VERIFIER_nondet_unsigned(void)   { return (unsigned)__saf_v(); }
unsigned char      __VERIFIER_nondet_U8(void)         { return (unsigned char)__saf_v(); }
unsigned short     __VERIFIER_nondet_U16(void)        { return (unsigned short)__saf_v(); }
unsigned int       __VERIFIER_nondet_U32(void)        { return (unsigned int)__saf_v(); }
char               __VERIFIER_nondet_S8(void)         { return (char)__saf_v(); }
short              __VERIFIER_nondet_S16(void)        { return (short)__saf_v(); }
int                __VERIFIER_nondet_S32(void)        { return (int)__saf_v(); }
long               __VERIFIER_nondet_loff_t(void)     { return (long)__saf_v(); }

void  *__VERIFIER_nondet_pointer(void) { return (void *)0; }
float  __VERIFIER_nondet_float(void)   { return 0.0f; }
double __VERIFIER_nondet_double(void)  { return 0.0; }
void   __VERIFIER_atomic_begin(void)   {}
void   __VERIFIER_atomic_end(void)     {}
void   __VERIFIER_assume(int c)        { if (!c) _exit(0); }
