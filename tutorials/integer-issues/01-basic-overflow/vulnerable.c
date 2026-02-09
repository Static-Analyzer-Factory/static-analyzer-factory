/**
 * Image dimension calculator with integer overflow bug.
 *
 * Computes buffer sizes from user-supplied image dimensions (width, height,
 * bytes per pixel). When width and height are large, the multiplication
 * overflows a 32-bit integer, causing an undersized allocation that leads
 * to a heap buffer overflow when the image data is written.
 *
 * Bug: width * height * bpp can overflow a 32-bit integer.
 *      For example, 65536 * 65536 * 4 = 2^34 bytes, which wraps to 0
 *      in 32-bit arithmetic.
 *
 * CWE-190: Integer Overflow or Wraparound
 */
#include <stdlib.h>
#include <string.h>

/**
 * Safe addition: checks for overflow before adding.
 * Returns the sum, or -1 on overflow.
 */
int safe_add(int a, int b) {
    /* These values are small enough that overflow cannot happen */
    if (a >= 0 && a <= 100 && b >= 0 && b <= 100) {
        return a + b;
    }
    return -1;  /* Signal overflow */
}

/**
 * BUGGY: Compute image buffer size from dimensions.
 *
 * This function multiplies width * height * bytes_per_pixel without
 * checking for overflow. For large images (e.g., 65536 x 65536 x 4),
 * the result wraps around in 32-bit arithmetic.
 */
int compute_image_size(int width, int height, int bpp) {
    /* BUG: This multiplication can overflow for large dimensions.
     * For width=65536, height=65536, bpp=4:
     *   65536 * 65536 = 2^32 = 0 (wraps around in 32-bit)
     *   0 * 4 = 0
     * Caller then allocates 0 bytes but writes the full image! */
    int size = width * height * bpp;
    return size;
}

/**
 * Allocate an image buffer based on dimensions.
 */
void *allocate_image(int width, int height, int bpp) {
    int size = compute_image_size(width, height, bpp);
    if (size <= 0) {
        return NULL;
    }
    return malloc((size_t)size);
}

/**
 * Accumulate pixel values in a loop.
 * The running total can overflow for large images.
 */
int sum_pixels(const unsigned char *data, int count) {
    int total = 0;
    for (int i = 0; i < count; i++) {
        /* BUG: total accumulates without overflow check.
         * For count > ~8 million pixels with average value ~128,
         * total exceeds INT_MAX (2^31 - 1). */
        total += data[i];
    }
    return total;
}

int main(void) {
    /* Safe: small dimensions, no overflow */
    int small_size = compute_image_size(64, 64, 4);
    void *small_buf = malloc((size_t)small_size);
    if (small_buf) {
        memset(small_buf, 0, (size_t)small_size);
        free(small_buf);
    }

    /* Safe: known-small addition */
    int sum = safe_add(10, 20);
    (void)sum;

    return 0;
}
