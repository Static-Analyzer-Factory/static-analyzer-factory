/**
 * Tutorial 11: Z3 Analysis Comparison
 *
 * File processing pipeline that exercises all Z3-enhanced analysis types.
 * Each section contains one false positive (guarded by branch conditions)
 * and one genuine bug.
 *
 * Section 1 - Memory Safety (taint/checker):
 *   FP: correlated malloc/free on same flag (path-insensitive reports leak)
 *   Bug: missing free on error path
 *
 * Section 2 - Typestate (file I/O):
 *   FP: file opened and closed in correlated branches
 *   Bug: file opened but never closed on early return
 *
 * Section 3 - Numeric (buffer access):
 *   FP: array access guarded by size check
 *   Bug: unchecked multiplication may overflow
 *
 * Section 4 - Taint (command injection):
 *   FP: input sanitized before use on validated path
 *   Bug: unsanitized input passed to system() on error path
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern char *getenv(const char *name);
extern int system(const char *command);

/* ── Section 1: Memory Safety ───────────────────────────────────────── */

void process_memory(int use_buffer) {
    char *buf = NULL;

    if (use_buffer) {
        buf = (char *)malloc(1024);
        if (!buf) return;
    }

    /* ... processing ... */

    if (use_buffer) {
        free(buf); /* Same guard: no leak (FP for path-insensitive) */
    }
}

void process_memory_buggy(int mode) {
    char *buf = (char *)malloc(512);
    if (!buf) return;

    if (mode == 1) {
        /* error path: forgot to free buf */
        return; /* GENUINE BUG: memory leak */
    }

    free(buf);
}

/* ── Section 2: Typestate (File I/O) ────────────────────────────────── */

void process_file(const char *path, int validate) {
    FILE *fp = NULL;

    if (validate) {
        fp = fopen(path, "r"); /* open */
        if (!fp) return;
    }

    /* ... processing ... */

    if (validate) {
        fclose(fp); /* Same guard: proper close (FP for path-insensitive) */
    }
}

void process_file_buggy(const char *path) {
    FILE *fp = fopen(path, "r");
    if (!fp) return;

    char line[256];
    if (fgets(line, sizeof(line), fp) == NULL) {
        return; /* GENUINE BUG: file descriptor leak on error */
    }

    fclose(fp);
}

/* ── Section 3: Numeric (Buffer Access) ─────────────────────────────── */

void process_array(int *data, int size) {
    if (size > 0 && size <= 100) {
        /* Guarded: access is safe (FP for path-insensitive overflow check) */
        for (int i = 0; i < size; i++) {
            data[i] = i * 2;
        }
    }
}

int process_multiply(int a, int b) {
    /* GENUINE BUG: unchecked multiplication may overflow */
    int result = a * b;
    return result;
}

/* ── Section 4: Taint (Command Injection) ───────────────────────────── */

void process_command(int trusted) {
    char *input = getenv("USER_INPUT"); /* taint source */
    char cmd[256];

    if (trusted) {
        /* Sanitize: replace dangerous characters */
        char safe[256];
        int j = 0;
        for (int i = 0; input[i] && j < 255; i++) {
            if (input[i] != ';' && input[i] != '|') {
                safe[j++] = input[i];
            }
        }
        safe[j] = '\0';
        snprintf(cmd, sizeof(cmd), "echo %s", safe);
        system(cmd); /* FP: sanitized input on trusted path */
    }
}

void process_command_buggy(void) {
    char *input = getenv("USER_INPUT"); /* taint source */
    char cmd[256];
    snprintf(cmd, sizeof(cmd), "process %s", input);
    system(cmd); /* GENUINE BUG: unsanitized input to system() */
}

/* ── Main ───────────────────────────────────────────────────────────── */

int main(void) {
    /* Section 1 */
    process_memory(1);
    process_memory_buggy(1);

    /* Section 2 */
    process_file("test.txt", 1);
    process_file_buggy("test.txt");

    /* Section 3 */
    int data[100];
    process_array(data, 50);
    process_multiply(1000000, 1000000);

    /* Section 4 */
    process_command(1);
    process_command_buggy();

    return 0;
}
