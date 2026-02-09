/**
 * Config file processor with resource management bugs.
 *
 * Bugs:
 * 1. File descriptor leak: fopen without fclose on error path
 * 2. Memory leak: malloc without free in read_config_value()
 *
 * Also demonstrates custom checker for a user-defined resource.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * Read a config value from a file.
 * Returns a heap-allocated string, or NULL on failure.
 */
char *read_config_value(const char *filepath, const char *key) {
    FILE *f = fopen(filepath, "r");
    if (!f) {
        return NULL;
    }

    char *buffer = (char *)malloc(1024);
    if (!buffer) {
        // BUG 1: File handle leaked — fclose(f) not called before return
        return NULL;
    }

    // Simplified: just read first line
    if (fgets(buffer, 1024, f) == NULL) {
        free(buffer);
        fclose(f);
        return NULL;
    }

    fclose(f);
    return buffer;  // Caller must free this
}

/**
 * Load and process application configuration.
 */
int load_config(const char *config_path) {
    // Read database host config
    char *db_host = read_config_value(config_path, "db_host");
    if (!db_host) {
        return -1;
    }

    // Read database port config
    char *db_port = read_config_value(config_path, "db_port");
    if (!db_port) {
        // BUG 2: Memory leak — db_host is not freed before returning
        return -1;
    }

    int port = atoi(db_port);

    // Use the values...
    printf("Connecting to %s:%d\n", db_host, port);

    free(db_host);
    free(db_port);
    return 0;
}

int main() {
    return load_config("/etc/myapp/config.ini");
}
