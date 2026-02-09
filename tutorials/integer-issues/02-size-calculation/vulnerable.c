/**
 * Image processing library with allocation size overflow vulnerabilities.
 *
 * Demonstrates how integer overflow in size calculations leads to
 * undersized allocations and subsequent buffer overflows.
 *
 * CWE-190: Integer Overflow or Wraparound
 * CWE-122: Heap-based Buffer Overflow (consequence)
 */
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

/**
 * BUG 1: Simple multiplication overflow in allocation.
 *
 * For large width and height values, width * height can overflow
 * a 32-bit integer, resulting in a small allocation.
 */
void *allocate_pixel_buffer(int width, int height) {
    /* BUG: This multiplication can overflow.
     * For width=65536, height=65536:
     *   65536 * 65536 = 2^32 = 0 (wraps around)
     * malloc(0) returns NULL or a minimal allocation. */
    size_t size = width * height;  /* Overflow happens here */
    return malloc(size);
}

/**
 * BUG 2: Multi-factor multiplication overflow.
 *
 * Common in image processing: width * height * channels * bytes_per_channel
 * Each additional factor increases overflow risk.
 */
void *allocate_rgba_buffer(int width, int height, int channels, int bpc) {
    /* BUG: Multiple multiplications, multiple overflow opportunities.
     * For 8K image (7680x4320) with 4 channels at 2 bytes each:
     *   7680 * 4320 = 33,177,600 (fits in 32-bit)
     *   33,177,600 * 4 = 132,710,400 (fits)
     *   132,710,400 * 2 = 265,420,800 (fits)
     * But for larger images, this chain can overflow. */
    int total = width * height * channels * bpc;
    if (total <= 0) {
        return NULL;  /* Tries to catch overflow, but too late */
    }
    return malloc((size_t)total);
}

/**
 * BUG 3: Array element count overflow.
 *
 * When allocating arrays of structures, count * sizeof(struct) can overflow.
 */
struct Pixel {
    uint8_t r, g, b, a;
};

struct Pixel *allocate_pixel_array(int count) {
    /* BUG: count * sizeof(struct Pixel) can overflow.
     * sizeof(Pixel) = 4 bytes
     * For count = 0x40000001 (about 1 billion):
     *   0x40000001 * 4 = 0x100000004, which truncates to 4 in 32-bit.
     * Allocates only 4 bytes but caller expects count * 4 bytes. */
    return (struct Pixel *)malloc(count * sizeof(struct Pixel));
}

/**
 * SAFE: Proper overflow checking before allocation.
 *
 * Uses 64-bit arithmetic and explicit overflow checks.
 */
void *allocate_pixel_buffer_safe(int width, int height) {
    /* Safe: Check for negative inputs */
    if (width <= 0 || height <= 0) {
        return NULL;
    }

    /* Safe: Use 64-bit arithmetic for the multiplication */
    uint64_t size = (uint64_t)width * (uint64_t)height;

    /* Safe: Check result fits in size_t and is reasonable */
    if (size > SIZE_MAX || size > (1ULL << 30)) {  /* 1GB limit */
        return NULL;
    }

    return malloc((size_t)size);
}

/**
 * BUG 4: Overflow in reallocation size calculation.
 *
 * Growing a buffer by a factor can overflow.
 */
void *grow_buffer(void *old_buf, size_t old_size, int growth_factor) {
    /* BUG: old_size * growth_factor can overflow.
     * For old_size = 0x80000000 (2GB), growth_factor = 2:
     *   0x80000000 * 2 = 0x100000000, truncates to 0 in 32-bit size_t.
     * realloc() with size 0 may return NULL or free the buffer. */
    size_t new_size = old_size * growth_factor;
    return realloc(old_buf, new_size);
}

/**
 * BUG 5: Stride calculation overflow.
 *
 * Common in image libraries: row stride = width * bytes_per_pixel + padding.
 */
void *allocate_image_with_stride(int width, int height, int bpp, int alignment) {
    /* BUG: stride calculation can overflow.
     * stride = ((width * bpp + alignment - 1) / alignment) * alignment */
    int stride = ((width * bpp + alignment - 1) / alignment) * alignment;

    /* BUG: stride * height can also overflow */
    int total = stride * height;

    if (total <= 0) {
        return NULL;
    }
    return malloc((size_t)total);
}

int main(void) {
    /* Test with small values - these work correctly */
    void *buf1 = allocate_pixel_buffer(100, 100);
    if (buf1) free(buf1);

    struct Pixel *buf2 = allocate_pixel_array(1000);
    if (buf2) free(buf2);

    void *buf3 = allocate_rgba_buffer(1920, 1080, 4, 1);
    if (buf3) free(buf3);

    void *buf4 = allocate_pixel_buffer_safe(1920, 1080);
    if (buf4) free(buf4);

    return 0;
}
