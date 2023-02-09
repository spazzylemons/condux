#include "../platform.h"

#include <math.h>
#include <SDL2/SDL.h>

static int screen_width, screen_height;
static SDL_Window *window;
static SDL_Renderer *renderer;
static Uint64 last_time_ms = 0;
static bool should_run = true;

void platform_init(int preferred_width, int preferred_height) {
    if (SDL_Init(SDL_INIT_VIDEO)) {
        // failed to initialize platform
        abort();
    }

    window = SDL_CreateWindow(
        "window",
        SDL_WINDOWPOS_CENTERED,
        SDL_WINDOWPOS_CENTERED,
        preferred_width,
        preferred_height,
        0
    );
    if (!window) {
        // failed to create window
        SDL_Quit();
        abort();
    }

    renderer = SDL_CreateRenderer(
        window,
        -1,
        SDL_RENDERER_ACCELERATED | SDL_RENDERER_PRESENTVSYNC
    );
    if (!renderer) {
        // failed to create renderer
        SDL_DestroyWindow(window);
        SDL_Quit();
        abort();
    }
}

void platform_deinit(void) {
    SDL_DestroyWindow(window);
    SDL_Quit();
}

void platform_line(float x0, float y0, float x1, float y1) {
    SDL_RenderDrawLine(
        renderer,
        roundf(x0),
        roundf(y0),
        roundf(x1),
        roundf(y1)
    );
}

bool platform_should_run(void) {
    return should_run;
}

float platform_start_frame(void) {
    // clear black
    SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
    SDL_RenderClear(renderer);
    // set to white for lines
    SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);
    // get frame time
    Uint64 now = SDL_GetTicks64();
    float result = (now - last_time_ms) / 1000.0f;
    last_time_ms = now;
    return result;
}

void platform_end_frame(void) {
    // present the window
    SDL_RenderPresent(renderer);
    // accept events
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        // check if the window should close
        if (event.type == SDL_WINDOWEVENT) {
            if (event.window.event == SDL_WINDOWEVENT_CLOSE) {
                // window close
                should_run = false;
            }
        }
    }
    // update dimensions
    SDL_GetRendererOutputSize(renderer, &screen_width, &screen_height);
}

int platform_width(void) {
    return screen_width;
}

int platform_height(void) {
    return screen_height;
}
