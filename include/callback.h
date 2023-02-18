#ifndef CONDUX_CALLBACK_H
#define CONDUX_CALLBACK_H

#include "platform.h"

WEB_EXPORT("game_init")
void game_init(void);

WEB_EXPORT("game_loop")
void game_loop(void);

void game_deinit(void);

#endif
