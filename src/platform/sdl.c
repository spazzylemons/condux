#include "../platform.h"

#include <math.h>
#include <SDL2/SDL.h>

static int screen_width, screen_height;
static SDL_Window *window;
static SDL_Renderer *renderer;
static Uint64 last_time_ms = 0;
static bool should_run = true;

static uint8_t keyboard_buttons = 0;

static SDL_Keycode keyboard_mapping[7] = {
    SDLK_UP,
    SDLK_DOWN,
    SDLK_LEFT,
    SDLK_RIGHT,
    SDLK_x,
    SDLK_z,
    SDLK_ESCAPE,
};

static SDL_GameController *controller = NULL;

void platform_init(int preferred_width, int preferred_height) {
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_GAMECONTROLLER)) {
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
    if (controller != NULL) {
        SDL_GameControllerClose(controller);
    }
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
        if (event.type == SDL_WINDOWEVENT) {
            // check if the window should close
            if (event.window.event == SDL_WINDOWEVENT_CLOSE) {
                // window close
                should_run = false;
            }
        } else if (event.type == SDL_KEYDOWN || event.type == SDL_KEYUP) {
            for (int i = 0; i < 7; i++) {
                if (event.key.keysym.sym == keyboard_mapping[i]) {
                    if (event.type == SDL_KEYDOWN) {
                        keyboard_buttons |= 1 << i;
                    } else {
                        keyboard_buttons &= ~(1 << i);
                    }
                    break;
                }
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

void platform_poll(Controls *controls) {
    // close controller if not attached
    if (controller != NULL && !SDL_GameControllerGetAttached(controller)) {
        SDL_GameControllerClose(controller);
    }
    // attempt to open a controller if not opened already
    if (controller == NULL) {
        int n = SDL_NumJoysticks();
        for (int i = 0; i < n; i++) {
            if (SDL_IsGameController(i)) {
                controller = SDL_GameControllerOpen(i);
                if (controller != NULL) {
                    break;
                }
            }
        }
    }
    controls->buttons = keyboard_buttons;
    if (controller != NULL) {
        if (SDL_GameControllerGetButton(controller, SDL_CONTROLLER_BUTTON_DPAD_UP)) controls->buttons |= BTN_UP;
        if (SDL_GameControllerGetButton(controller, SDL_CONTROLLER_BUTTON_DPAD_DOWN)) controls->buttons |= BTN_DOWN;
        if (SDL_GameControllerGetButton(controller, SDL_CONTROLLER_BUTTON_DPAD_LEFT)) controls->buttons |= BTN_LEFT;
        if (SDL_GameControllerGetButton(controller, SDL_CONTROLLER_BUTTON_DPAD_RIGHT)) controls->buttons |= BTN_RIGHT;
        if (SDL_GameControllerGetButton(controller, SDL_CONTROLLER_BUTTON_A)) controls->buttons |= BTN_OK;
        if (SDL_GameControllerGetButton(controller, SDL_CONTROLLER_BUTTON_B)) controls->buttons |= BTN_BACK;
        if (SDL_GameControllerGetButton(controller, SDL_CONTROLLER_BUTTON_START)) controls->buttons |= BTN_PAUSE;
        Sint16 axis = SDL_GameControllerGetAxis(controller, SDL_CONTROLLER_AXIS_LEFTX);
        if (axis == -32768) axis = -32767;
        controls->steering = axis / 32767.0;
    } else {
        // if no controller connected, use keyboard steering
        controls->steering = (keyboard_buttons & BTN_LEFT) ? -1.0f : (keyboard_buttons & BTN_RIGHT) ? 1.0f : 0.0f;
    }
}
