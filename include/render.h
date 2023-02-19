#ifndef CONDUX_RENDER_H
#define CONDUX_RENDER_H

#include "types.h"

#include <stdarg.h>
#include <stdbool.h>

void render_init(void);

void render_line(const Vec a, const Vec b);

void render_load_spline(const Spline *spline);

void render_spline(void);

bool mesh_load(Mesh *mesh, Asset *asset);

void mesh_render(const Mesh *mesh, const Vec translation, const Mtx rotation);

#endif
