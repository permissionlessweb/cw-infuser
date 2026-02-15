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

    size_t bufsize = (size_t)rows * ((size_t)cols * 20) + 1; // Rough estimate for color codes
    char *buf = malloc(bufsize);
    if (!buf) return;
    size_t pos = 0;

    for (int y = 0; y < rows; y++) {
        double ci = max_i - (y + 0.5) * step_i;
        int last_color = -1;

        for (int x = 0; x < cols; x++) {
            double cr = min_r + (x + 0.5) * step_r;
            double zr = 0.0, zi = 0.0;
            int iter;

            for (iter = 0; iter < max_iter; iter++) {
                double zrsq = zr * zr;
                double zisq = zi * zi;
                if (zrsq + zisq > 4.0) break;
                double temp = zrsq - zisq + cr;
                zi = 2.0 * zr * zi + ci;
                zr = temp;
            }

            int color;
            if (iter == max_iter) {
                color = 0; // Black for inside
            } else {
                int t = iter % 24;
                // Map iteration to a smooth 256-color palette (blue -> purple -> red)
                color = 16 + (t * 10) % 240;
            }

            // Only emit color code if changed
            if (color != last_color) {
                pos += sprintf(buf + pos, "\033[38;5;%dm", color);
                last_color = color;
            }

            buf[pos++] = (iter == max_iter) ? ' ' : (base + (iter % 95));
        }

        // Reset color at end of line
        pos += sprintf(buf + pos, "\033[0m\n");
    }

    buf[pos] = '\0';
    printf("\033[H%s", buf);  // Move cursor to top-left and draw
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