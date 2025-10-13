#include <stdio.h>
#include <math.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <stdlib.h>

void mandelbrot(int base, double zoom, double center_r, double center_i, int max_iter) {
    struct winsize w;
    ioctl(0, TIOCGWINSZ, &w);
    int cols = w.ws_col > 0 ? w.ws_col : 80;
    int rows = w.ws_row > 0 ? w.ws_row : 24;

    double width = 3.0 / zoom;
    double height = 2.0 / zoom;
    double min_r = center_r - width / 2.0;
    double max_r = center_r + width / 2.0;
    double min_i = center_i - height / 2.0;
    double max_i = center_i + height / 2.0;
    double step_r = width / cols;
    double step_i = height / rows;

    size_t bufsize = (size_t)rows * ((size_t)cols + 1) + 1;
    char *buf = malloc(bufsize);
    if (!buf) return;
    size_t pos = 0;

    for (int y = 0; y < rows; y++) {
        double ci = max_i - (y + 0.5) * step_i;
        for (int x = 0; x < cols; x++) {
            double cr = min_r + (x + 0.5) * step_r;
            double zr = 0.0;
            double zi = 0.0;
            int iter;

            // Escape loop
            for (iter = 0; iter < max_iter; iter++) {
                double zrsq = zr * zr;
                double zisq = zi * zi;
                if (zrsq + zisq > 4.0) break; // Escapes!
                double temp = zrsq - zisq + cr;
                zi = 2.0 * zr * zi + ci;
                zr = temp;
            }

            // Render: inside → blank, outside → shaded ASCII
            if (iter == max_iter) {
                buf[pos++] = ' ';           // Inside set: empty (or use '.' for dot fill)
            } else {
                buf[pos++] = base + (iter % 95); // Outside: shaded
            }
        }
        buf[pos++] = '\n';
    }
    buf[pos] = '\0';
    printf("\033[H%s", buf);  // ANSI escape to move cursor + print buffer
    fflush(stdout);
    free(buf);
}

int main() {
    // =============================================
    // 🔍 REGION: Elephant Valley (High-Precision)
    // =============================================
    double center_r = 0.2825852632545845;
    double center_i = 0.011953425728572489057;

    int frame = 0;
    while (1) {
        double zoom = pow(1.02, frame);
        int max_iter = 26 + frame / 3;

        mandelbrot(32, zoom, center_r, center_i, max_iter);

        usleep(50000); // ~20 FPS
        frame++;
    }
    return 0;
}