#ifndef CONDUX_ASSETS_H
#define CONDUX_ASSETS_H

#include "types.h"

#include <stdbool.h>

bool asset_load(Asset *asset, const char *name);
bool asset_read_byte(Asset *asset, uint8_t *b);
bool asset_read_fixed(Asset *asset, float *f);
bool asset_read_vec(Asset *asset, Vec v);

#endif
