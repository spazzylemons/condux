#ifndef CONDUX_PLATFORM_H
#define CONDUX_PLATFORM_H

#include <stdbool.h>
#include <stdint.h>

#ifdef CONDUX_WEB
#define WEB_EXPORT(name) __attribute__((export_name(name)))
#define WEB_IMPORT(name) __attribute__((import_name(name)))
#define WEB_PACKED __attribute__((__packed__))
#else
#define WEB_EXPORT(name)
#define WEB_IMPORT(name)
#define WEB_PACKED
#endif

#define BTN_UP    (1 << 0)
#define BTN_DOWN  (1 << 1)
#define BTN_LEFT  (1 << 2)
#define BTN_RIGHT (1 << 3)
#define BTN_OK    (1 << 4)
#define BTN_BACK  (1 << 5)
#define BTN_PAUSE (1 << 6)

/**
 * Controller structure.
 */
typedef struct {
    uint8_t buttons;
    float steering;
} Controls;

/**
 * Initializes the platform-specific code.
 * The preferred screen resolution is passed in.
 */
WEB_IMPORT("platform_init")
void platform_init(int preferred_width, int preferred_height);

/**
 * Finalizes the platform-specific code.
 */
void platform_deinit(void);

/**
 * Draw a line on the screen.
 */
WEB_IMPORT("platform_line")
void platform_line(float x0, float y0, float x1, float y1);

/**
 * Return true unless the program has been asked to close by the underlying
 * system.
 */
bool platform_should_run(void);

/**
 * Begin the current frame. Returns the time in seconds since the last call to
 * platform_start_frame.
 */
float platform_start_frame(void);

/**
 * Finish drawing the current frame and wait for vblank or similar.
 */
void platform_end_frame(void);

/**
 * Returns the width of the screen in pixels.
 */
WEB_IMPORT("platform_width")
int platform_width(void);

/**
 * Returns the height of the screen in pixels.
 */
WEB_IMPORT("platform_height")
int platform_height(void);

/**
 * Poll the controller.
 */
WEB_IMPORT("platform_poll")
void platform_poll(Controls *controls);

#endif
