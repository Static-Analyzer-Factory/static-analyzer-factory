/**
 * Complex buffer overflow pattern requiring context-sensitive PTA and Z3.
 *
 * This program demonstrates indirect pointer flows through wrapper functions
 * where context-insensitive analysis conflates different allocation sites,
 * and path conditions determine whether an overflow actually occurs.
 *
 * CWE-120: Buffer Copy without Checking Size of Input
 * CWE-121: Stack-based Buffer Overflow
 */
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
    char *data;
    size_t size;
    size_t capacity;
} DynamicBuffer;

/* Wrapper function that creates buffers of different sizes */
DynamicBuffer *create_buffer(size_t capacity) {
    DynamicBuffer *buf = (DynamicBuffer *)malloc(sizeof(DynamicBuffer));
    if (!buf) return NULL;

    buf->data = (char *)malloc(capacity);
    if (!buf->data) {
        free(buf);
        return NULL;
    }

    buf->size = 0;
    buf->capacity = capacity;
    return buf;
}

void free_buffer(DynamicBuffer *buf) {
    if (buf) {
        free(buf->data);
        free(buf);
    }
}

/* Write data to buffer - has bounds check but with subtle bug */
int buffer_write(DynamicBuffer *buf, const char *data, size_t len) {
    if (!buf || !data) return -1;

    /* BUG: Off-by-one - should be buf->size + len <= buf->capacity */
    if (buf->size + len < buf->capacity) {
        memcpy(buf->data + buf->size, data, len);
        buf->size += len;
        return 0;
    }
    return -1;  /* Would overflow */
}

/* Process user input with two different buffer sizes */
int process_input(const char *small_input, const char *large_input) {
    /* Small buffer for metadata */
    DynamicBuffer *meta = create_buffer(32);
    if (!meta) return -1;

    /* Large buffer for content */
    DynamicBuffer *content = create_buffer(1024);
    if (!content) {
        free_buffer(meta);
        return -1;
    }

    /*
     * Context-insensitive PTA would conflate meta->data and content->data
     * because both come from create_buffer(). Only context-sensitive analysis
     * can distinguish that meta has capacity 32 and content has capacity 1024.
     */

    /* Write to small buffer - potential overflow if input is > 31 bytes */
    if (buffer_write(meta, small_input, strlen(small_input)) < 0) {
        printf("Metadata too large\n");
        /* BUG: On error, we don't free and return - leak both buffers */
    }

    /* Write to large buffer - safe for most inputs */
    buffer_write(content, large_input, strlen(large_input));

    free_buffer(meta);
    free_buffer(content);
    return 0;
}

/* Path-dependent overflow: only happens if flag is true AND size > threshold */
int conditional_overflow(int flag, size_t user_size) {
    char stack_buf[64];
    char *heap_buf = (char *)malloc(128);

    if (!heap_buf) return -1;

    if (flag) {
        /* Only in this branch: potential stack overflow if user_size > 64 */
        if (user_size > 0 && user_size < 256) {
            /* BUG: No upper bound check against stack_buf size */
            memset(stack_buf, 'A', user_size);
        }
    } else {
        /* This branch is safe - uses heap buffer */
        if (user_size > 0 && user_size < 128) {
            memset(heap_buf, 'B', user_size);
        }
    }

    free(heap_buf);
    return 0;
}

int main(int argc, char *argv[]) {
    /* Simulate user input */
    const char *meta_input = "This is metadata that might be too long for the buffer";
    const char *content_input = "Content goes here";

    process_input(meta_input, content_input);

    /* Path-dependent: flag=1, size=100 will overflow stack_buf */
    conditional_overflow(1, 100);

    /* Path-dependent: flag=0, size=100 is safe (uses heap) */
    conditional_overflow(0, 100);

    return 0;
}
