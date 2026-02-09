/**
 * Simple double-free vulnerability demonstration.
 *
 * CWE-415: Double Free
 *
 * A double-free occurs when free() is called twice on the same pointer.
 * This corrupts the heap allocator's internal data structures, leading
 * to crashes, data corruption, or exploitable conditions.
 */
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *data;
    int size;
} Buffer;

Buffer *buffer_create(int size) {
    Buffer *buf = (Buffer *)malloc(sizeof(Buffer));
    if (!buf) return NULL;

    buf->data = (char *)malloc(size);
    if (!buf->data) {
        free(buf);
        return NULL;
    }

    buf->size = size;
    memset(buf->data, 0, size);
    return buf;
}

void buffer_free(Buffer *buf) {
    if (buf) {
        free(buf->data);
        free(buf);
    }
}

int process_data(void) {
    Buffer *buf = buffer_create(256);
    if (!buf) return -1;

    /* Do some work with the buffer */
    strcpy(buf->data, "Hello, World!");

    /* First free - correct */
    buffer_free(buf);

    /* BUG: Double-free - buf was already freed above */
    buffer_free(buf);

    return 0;
}

int main(void) {
    return process_data();
}
