#include "assets.h"
#include "bundle.h"

#include <string.h>

bool asset_load(Asset *asset, const char *name) {
    for (const AssetEntry *entry = generated_assets; entry->name != NULL; entry++) {
        if (!strcmp(name, entry->name)) {
            asset->entry = entry;
            asset->index = 0;
            return true;
        }
    }
    return false;
}

bool asset_read_byte(Asset *asset, uint8_t *b) {
    if (asset->index >= asset->entry->size) {
        return false;
    }
    *b = asset->entry->data[asset->index++];
    return true;
}

bool asset_read_fixed(Asset *asset, float *f) {
    uint8_t lo, hi;
    if (!asset_read_byte(asset, &lo)) return false;
    if (!asset_read_byte(asset, &hi)) return false;
    *f = (int16_t) ((uint16_t) lo + ((uint16_t) hi << 8)) / 256.0f;
    return true;
}

bool asset_read_vec(Asset *asset, Vec v) {
    if (!asset_read_fixed(asset, &v[0])) return false;
    if (!asset_read_fixed(asset, &v[1])) return false;
    return asset_read_fixed(asset, &v[2]);
}
