#include "linalg.h"
#include "platform.h"
#include "render.h"

#include <math.h>
#include <stdio.h>

#define CUTOFF 0.01f

void render_line(Vec a, Vec b) {
    if (a[2] < CUTOFF && b[2] < CUTOFF) {
        // lies entirely behind camera, don't draw it
        return;
    }
    // sort endpoints
    if (a[2] > b[2]) {
        float *temp = a;
        a = b;
        b = temp;
    }
    Vec c;
    if (a[2] < CUTOFF && b[2] > CUTOFF) {
        // if line crosses, we need to cut the line
        float n = (b[2] - CUTOFF) / (b[2] - a[2]);
        Vec d;
        vec_copy(c, a);
        vec_copy(d, b);
        vec_scale(c, n);
        vec_scale(d, 1.0f - n);
        vec_add(c, d);
        a = c;
    }
    // adjust for screen res
    float width = platform_width();
    float height = platform_height();
    float scale = width < height ? width : height;
    // draw it
    float x0 = scale * (a[0] / a[2]) + (width / 2);
    float y0 = scale * (a[1] / a[2]) + (height / 2);
    float x1 = scale * (b[0] / b[2]) + (width / 2);
    float y1 = scale * (b[1] / b[2]) + (height / 2);
    platform_line(x0, y0, x1, y1);
}
