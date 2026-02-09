/**
 * Configuration file parser with null-guarded pointer usage.
 *
 * Demonstrates path-sensitive analysis:
 * - Some potential null dereferences are guarded by branch conditions,
 *   making them false positives under path-sensitive analysis.
 * - One genuine use-after-free is NOT guarded and should survive
 *   path-sensitive filtering.
 *
 * Expected:
 *   Path-insensitive: may report findings for both guarded and unguarded paths.
 *   Path-sensitive:   should filter out infeasible paths (null dereferences
 *                     guarded by null checks), preserving genuine bugs.
 */
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
    char *key;
    char *value;
} ConfigEntry;

typedef struct {
    ConfigEntry *entries;
    int count;
    int capacity;
} Config;

Config *config_create(int capacity) {
    Config *cfg = (Config *)malloc(sizeof(Config));
    if (!cfg) return NULL;

    cfg->entries = (ConfigEntry *)malloc(sizeof(ConfigEntry) * capacity);
    if (!cfg->entries) {
        /* Correctly frees cfg when entries allocation fails */
        free(cfg);
        return NULL;
    }

    cfg->count = 0;
    cfg->capacity = capacity;
    return cfg;
}

int config_add(Config *cfg, const char *key, const char *value) {
    /* Null guard: only dereference cfg if non-null.
     * A path-insensitive checker might flag the dereference below
     * as a potential null-deref, but the guard makes it infeasible. */
    if (!cfg) return -1;

    if (cfg->count >= cfg->capacity) return -2;

    cfg->entries[cfg->count].key = strdup(key);
    cfg->entries[cfg->count].value = strdup(value);
    cfg->count++;
    return 0;
}

void config_free(Config *cfg) {
    if (!cfg) return;
    for (int i = 0; i < cfg->count; i++) {
        free(cfg->entries[i].key);
        free(cfg->entries[i].value);
    }
    free(cfg->entries);
    free(cfg);
}

const char *config_get(Config *cfg, const char *key) {
    /* Null guard: returns NULL early if cfg is NULL. */
    if (!cfg) return NULL;

    for (int i = 0; i < cfg->count; i++) {
        if (strcmp(cfg->entries[i].key, key) == 0) {
            return cfg->entries[i].value;
        }
    }
    return NULL;
}

int process_config(const char *filename) {
    Config *cfg = config_create(16);

    /* Add entries */
    config_add(cfg, "host", "localhost");
    config_add(cfg, "port", "8080");
    config_add(cfg, "timeout", "30");

    /* Lookup a value */
    const char *host = config_get(cfg, "host");

    /* Free the config */
    config_free(cfg);

    /* BUG: Use-after-free -- accessing host which points into
     * freed cfg->entries[0].value memory. This is a genuine bug
     * that should survive path-sensitive filtering. */
    if (host) {
        printf("Connecting to: %s\n", host);
    }

    return 0;
}

int main() {
    return process_config("config.ini");
}
