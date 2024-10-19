#/usr/bin/env bash

CFLAGS="-g -O0"
BUILD_DIR="build"

clang $CFLAGS -c ./main.c -o $BUILD_DIR/main.o

clang -o $BUILD_DIR/main ./$BUILD_DIR/main.o
