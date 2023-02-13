#ifndef CONDUX_TIMING_H
#define CONDUX_TIMING_H

#include <stdint.h>

/**
 * Load the initial platform timer.
 */
void timing_init(void);

/**
 * Run this after each frame. This returns the number of ticks to run.
 * It also calculates an interpolation value.
 */
uint16_t timing_num_ticks(float *interpolation);

#endif
