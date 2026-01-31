// SV-COMP special functions - declarations only
// These are used by SV-COMP benchmarks to interact with verifiers.
// We provide declarations so LLVM can compile the code; actual semantics
// are handled by the analysis (e.g., __VERIFIER_error marks unreachable code).

#ifndef SV_COMP_STUBS_H
#define SV_COMP_STUBS_H

#ifdef __cplusplus
extern "C" {
#endif

// Error function - if reachable, the property is violated
extern void __VERIFIER_error(void) __attribute__((noreturn));

// Assume function - constrain the state space
extern void __VERIFIER_assume(int condition);

// Nondeterministic value generators
extern int __VERIFIER_nondet_int(void);
extern unsigned int __VERIFIER_nondet_uint(void);
extern long __VERIFIER_nondet_long(void);
extern unsigned long __VERIFIER_nondet_ulong(void);
extern long long __VERIFIER_nondet_longlong(void);
extern unsigned long long __VERIFIER_nondet_ulonglong(void);
extern short __VERIFIER_nondet_short(void);
extern unsigned short __VERIFIER_nondet_ushort(void);
extern char __VERIFIER_nondet_char(void);
extern unsigned char __VERIFIER_nondet_uchar(void);
extern _Bool __VERIFIER_nondet_bool(void);
extern void* __VERIFIER_nondet_pointer(void);
extern float __VERIFIER_nondet_float(void);
extern double __VERIFIER_nondet_double(void);
extern size_t __VERIFIER_nondet_size_t(void);

// Atomic section markers (for concurrency benchmarks)
extern void __VERIFIER_atomic_begin(void);
extern void __VERIFIER_atomic_end(void);

// Assert function (some benchmarks use this instead of __VERIFIER_error)
#ifndef __VERIFIER_assert
#define __VERIFIER_assert(cond) do { if (!(cond)) __VERIFIER_error(); } while(0)
#endif

#ifdef __cplusplus
}
#endif

#endif // SV_COMP_STUBS_H
