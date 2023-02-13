#ifndef CONDUX_RENDER_H
#define CONDUX_RENDER_H

#include "types.h"

#include <stdbool.h>

void set_camera(const Vec eye, const Vec at, const Vec up);

void render_line(const Vec a, const Vec b);

void render_load_spline(const Spline *spline);

void render_spline(void);

void render_deinit(void);

bool mesh_load(Mesh *mesh, Asset *asset);

void mesh_render(const Mesh *mesh, const Vec translation, const Mtx rotation);

#endif
