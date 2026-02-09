/**
 * Config file reader with file I/O typestate bugs.
 *
 * Bugs detected by typestate analysis:
 * 1. read_config_leak(): fopen without fclose — file leak
 * 2. read_config_double_close(): fclose called twice — double-close error
 * 3. read_config_use_after_close(): fread after fclose — use-after-close error
 * 4. read_config_correct(): properly opens, reads, and closes — no bug
 */
#include <stdio.h>
#include <string.h>

/* BUG 1: File leak — fopen without fclose */
void read_config_leak(void) {
    FILE *fp = fopen("config.ini", "r");
    fread(NULL, 1, 1, fp);
    /* Missing fclose(fp) — resource leaked */
}

/* BUG 2: Double-close — fclose called twice */
void read_config_double_close(void) {
    FILE *fp = fopen("config.ini", "r");
    fread(NULL, 1, 1, fp);
    fclose(fp);
    fclose(fp);  /* BUG: double-close */
}

/* BUG 3: Use-after-close — fread after fclose */
void read_config_use_after_close(void) {
    FILE *fp = fopen("config.ini", "r");
    fclose(fp);
    fread(NULL, 1, 1, fp);  /* BUG: use-after-close */
}

/* CORRECT: Proper open-read-close sequence */
void read_config_correct(void) {
    FILE *fp = fopen("config.ini", "r");
    fread(NULL, 1, 1, fp);
    fclose(fp);
}
