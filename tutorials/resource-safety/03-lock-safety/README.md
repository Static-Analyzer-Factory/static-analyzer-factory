# Tutorial 3: Mutex Lock/Unlock Tracking (CWE-667)

## What You'll Learn

- How typestate analysis applies to synchronization primitives
- Detecting held locks at function exit
- Detecting double-lock and unlock-without-lock bugs
- Creating custom typestate specifications

## Prerequisites

> Complete [Tutorial 2: Typestate File I/O](../02-typestate-file-io/README.md) first.

## The Vulnerability

Improper locking (CWE-667) occurs when mutex operations are performed
incorrectly:

### 1. Held Lock at Exit
```cpp
void acquire_no_release(void) {
    pthread_mutex_t mtx;
    pthread_mutex_init(&mtx, nullptr);
    pthread_mutex_lock(&mtx);
    /* Missing pthread_mutex_unlock - lock held at exit */
}
```

### 2. Double Lock
```cpp
void double_lock(void) {
    pthread_mutex_t mtx;
    pthread_mutex_init(&mtx, nullptr);
    pthread_mutex_lock(&mtx);
    pthread_mutex_lock(&mtx);  /* BUG: deadlock! */
}
```

### 3. Unlock Without Lock
```cpp
void unlock_without_lock(void) {
    pthread_mutex_t mtx;
    pthread_mutex_init(&mtx, nullptr);
    pthread_mutex_unlock(&mtx);  /* BUG: unlock without prior lock */
}
```

## Why Lock Bugs Matter

Lock bugs cause:
- **Deadlocks**: Thread waits forever for a lock it already holds
- **Data races**: Forgetting to lock before accessing shared data
- **Undefined behavior**: Unlocking an unlocked mutex
- **Resource exhaustion**: Held locks prevent other threads from progressing

## The Mutex State Machine

```
                    pthread_mutex_init()
      [Uninit] --------------------------> [Unlocked]
                                              |   ^
               pthread_mutex_lock()           |   |  pthread_mutex_unlock()
                                              v   |
                                           [Locked]
                                              |
                   pthread_mutex_lock()       |
                   (while locked)             |
                                              v
                                           [Error]

Accepting states: Uninit, Unlocked
Error states: Error
```

## Run the Detector

```bash
python3 detect.py
```

Expected output:

```
Step 1: Compiling C++ source to LLVM IR...
  Compiled: vulnerable.cpp -> vulnerable.ll

Step 2: Loading via LLVM frontend...
  Project loaded: <saf.Project object>

Step 3: Running mutex_lock typestate analysis...
  Result: <saf.TypestateResult object>

Step 4: Inspecting findings...

  Held-lock findings (non-accepting at exit): 1
    [leak] state=Locked, resource=0x1234...
      spec: mutex_lock

  Error-state findings: 0

Step 5: IDE solver diagnostics...
  Jump function updates: 12
  Value propagations: 35

Step 6: Custom typestate spec example...
  Custom spec result: <saf.TypestateResult object>

============================================================
SUMMARY: Found 1 mutex typestate violations
  Held locks at exit: 1
  -> Lock leak DETECTED
============================================================
```

## Understanding the Code

```python
import saf

proj = saf.Project.open(str(llvm_ir))

# Run typestate analysis with the built-in mutex_lock spec
result = proj.typestate("mutex_lock")

# Get held-lock findings (lock not released before exit)
leaks = result.leak_findings()

# Get error findings (double-lock, unlock-without-lock)
errors = result.error_findings()
```

## Creating a Custom Typestate Spec

You can define your own typestate specifications for domain-specific
resources:

```python
import saf

# Define a custom mutex spec with explicit error transitions
custom_spec = saf.TypestateSpec(
    name="custom_mutex",
    states=["uninit", "unlocked", "locked", "error"],
    initial_state="unlocked",
    error_states=["error"],
    accepting_states=["uninit", "unlocked"],
    transitions=[
        # (current_state, function_name, next_state)
        ("unlocked", "pthread_mutex_lock", "locked"),
        ("locked", "pthread_mutex_unlock", "unlocked"),
        ("locked", "pthread_mutex_lock", "error"),      # Double lock
        ("unlocked", "pthread_mutex_unlock", "error"),  # Unlock w/o lock
    ],
    constructors=["pthread_mutex_init"],
)

# Run analysis with the custom spec
custom_result = proj.typestate_custom(custom_spec)
```

### TypestateSpec Parameters

| Parameter | Description |
|-----------|-------------|
| `name` | Identifier for the spec |
| `states` | List of all possible states |
| `initial_state` | State before any operation |
| `error_states` | States that indicate a bug |
| `accepting_states` | Valid states at function exit |
| `transitions` | `(from_state, function, to_state)` tuples |
| `constructors` | Functions that create new resources |

## Correct Code

```cpp
void acquire_release_correct(void) {
    pthread_mutex_t mtx;
    pthread_mutex_init(&mtx, nullptr);
    pthread_mutex_lock(&mtx);
    pthread_mutex_unlock(&mtx);  /* Proper unlock */
}
```

This function:
1. Initializes: creates new mutex in `Unlocked` state
2. Locks: `Unlocked` -> `Locked`
3. Unlocks: `Locked` -> `Unlocked` (accepting)
4. Exits in accepting state: no leak

## Real-World Patterns

### RAII Lock Guards (C++)

Modern C++ uses RAII to avoid manual lock management:

```cpp
void safe_with_guard(std::mutex& mtx) {
    std::lock_guard<std::mutex> guard(mtx);
    // Critical section
}  // guard destructor releases lock
```

SAF's typestate analysis works on the compiled LLVM IR, so it sees the
inlined constructor/destructor calls and can still track the lock state.

### Multiple Locks

For deadlock-free ordering, you might define a custom spec that tracks
which locks are held:

```python
# Simplified example - real deadlock detection needs more states
multi_lock_spec = saf.TypestateSpec(
    name="ordered_locks",
    states=["none", "has_A", "has_B", "has_both", "error"],
    initial_state="none",
    error_states=["error"],
    accepting_states=["none"],
    transitions=[
        ("none", "lock_A", "has_A"),
        ("has_A", "lock_B", "has_both"),
        ("has_both", "unlock_B", "has_A"),
        ("has_A", "unlock_A", "none"),
        # Out-of-order acquisition
        ("none", "lock_B", "has_B"),
        ("has_B", "lock_A", "error"),  # Wrong order!
    ],
    constructors=[],
)
```

## Exercises

1. **Add double-lock detection**: The `vulnerable.cpp` doesn't have a
   double-lock case. Add one and verify SAF detects it with the custom spec.

2. **Add unlock-without-lock**: Create a function that unlocks without
   locking first. Does SAF report an error?

3. **Test conditional paths**: Create a function where locking happens in
   one branch. Does SAF track the state correctly across branches?

4. **Compare with memory-alloc**: How does the `mutex_lock` spec differ
   from the `memory_alloc` spec?

## Next Steps

Continue to [Tutorial 4: Custom Resources](../04-custom-resources/README.md)
to learn how to define typestate specifications for your own domain-specific
resources like database connections or network sockets.
