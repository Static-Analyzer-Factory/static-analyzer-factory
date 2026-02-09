/**
 * Simplified HTTP header parser with memory safety bugs.
 *
 * Bugs:
 * 1. Memory leak: parse_header() allocates but caller doesn't free on error path
 * 2. Use-after-free: freed header is accessed after cleanup
 * 3. Double-free: header freed twice (once in cleanup, once at end)
 */
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
    char *name;
    char *value;
} Header;

Header *parse_header(const char *line) {
    Header *h = (Header *)malloc(sizeof(Header));
    if (!h) return NULL;

    h->name = (char *)malloc(64);
    h->value = (char *)malloc(256);

    if (!h->name || !h->value) {
        // BUG 1: Partial cleanup — if value alloc fails, name leaks
        free(h);
        return NULL;
    }

    // Simplified parse: just copy the line as name
    strncpy(h->name, line, 63);
    h->name[63] = '\0';
    strncpy(h->value, line, 255);
    h->value[255] = '\0';

    return h;
}

void free_header(Header *h) {
    if (h) {
        free(h->name);
        free(h->value);
        free(h);
    }
}

int process_request(const char *raw_request) {
    Header *content_type = parse_header(raw_request);
    if (!content_type) {
        return -1;
    }

    // Process the header...
    int result = (int)strlen(content_type->value);

    // BUG 2 & 3: Free and then use + double-free
    free_header(content_type);

    // BUG 2: Use after free — accessing freed memory
    printf("Processed header: %s\n", content_type->name);

    // BUG 3: Double free
    free_header(content_type);

    return result;
}

int main() {
    const char *request = "Content-Type: application/json";
    return process_request(request);
}
