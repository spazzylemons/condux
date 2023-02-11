#include "platform.h"

#include <math.h>
#include <SDL2/SDL.h>

#include <GL/gl.h>

static int screen_width, screen_height;
static SDL_Window *window;
static SDL_GLContext gl_context;
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
    screen_width = preferred_width;
    screen_height = preferred_height;
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
        SDL_WINDOW_OPENGL | SDL_WINDOW_RESIZABLE
    );
    if (!window) {
        // failed to create window
        SDL_Quit();
        abort();
    }

    gl_context = SDL_GL_CreateContext(window);
    if (!gl_context) {
        SDL_DestroyWindow(window);
        SDL_Quit();
        abort();
    }

    glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
    glColor3f(1.0f, 1.0f, 1.0f);
}

void platform_deinit(void) {
    if (controller != NULL) {
        SDL_GameControllerClose(controller);
    }
    SDL_DestroyWindow(window);
    SDL_Quit();
}

static void point(float x, float y) {
    // convert to [-1, 1]
    x /= screen_width / 2.0f;
    x -= 1.0f;
    y /= screen_height / 2.0f;
    y = 1.0f - y; // flip y axis
    glVertex2f(x, y);
}

void platform_line(float x0, float y0, float x1, float y1) {
    point(x0, y0);
    point(x1, y1);
}

bool platform_should_run(void) {
    return should_run;
}

float platform_start_frame(void) {
    glClear(GL_COLOR_BUFFER_BIT);
    // we start drawing GL primitives
    glBegin(GL_LINES);
    // get frame time
    Uint64 now = SDL_GetTicks64();
    float result = (now - last_time_ms) / 1000.0f;
    last_time_ms = now;
    return result;
}

void platform_end_frame(void) {
    // finish drawing lines
    glEnd();
    // present the window
    SDL_GL_SwapWindow(window);
    // accept events
    SDL_Event event;
    while (SDL_PollEvent(&event)) {
        if (event.type == SDL_WINDOWEVENT) {
            // check if the window should close
            if (event.window.event == SDL_WINDOWEVENT_CLOSE) {
                // window close
                should_run = false;
            } else if (event.window.event == SDL_WINDOWEVENT_RESIZED) {
                glViewport(0, 0, event.window.data1, event.window.data2);
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
    SDL_GL_GetDrawableSize(window, &screen_width, &screen_height);
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
