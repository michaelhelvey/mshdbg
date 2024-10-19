#include <stdio.h>

void foo(void) {
  int a = 1;
  int b = 2;
  int c = a + b;

  printf("(foo): %d\n", c);
}

int main(void) {
  int a = 69;
  printf("(main): %d\n", a);
  foo();

  return 0;
}
