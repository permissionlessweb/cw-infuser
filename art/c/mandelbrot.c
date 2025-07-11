#include <stdio.h>

void mandelbrot(int n) {
    float r, i, R, I, b;
    for (i = -1; i < 1; i += .06, puts(""))
        for (r = -2; I = i, (R = r) < 1; r += .03, putchar(n + 31))
            for (n = 0; b = I * I, 26 > n++ && R * R + b < 4; I = 2 * R * I + i, R = R * R - b + r);
}

int main() {
    mandelbrot(32); // Use 32 to get printable ASCII characters (32 + 31 = 63, which is '?')
    return 0;
}