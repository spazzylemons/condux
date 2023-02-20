#ifndef CONDUX_TYPES_H
#define CONDUX_TYPES_H

#include "macros.h"

#include <stddef.h>
#include <stdint.h>

typedef float Vec[3];
typedef float Quat[4];
typedef float Mtx[3][3];

typedef struct {
    const char *name;
    size_t size;
    const char *data;
} AssetEntry;

typedef struct {
    const AssetEntry *entry;
    size_t index;
} Asset;

#endif
