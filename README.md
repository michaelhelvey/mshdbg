# mshdbg

Just playing around with Rust GUI stuff. In theory writing a visual debugger but if I can come away
with some fun GUI patterns that's all I really care about.

LLDB (+ more someday?) GUI frontend for my personal use. I wanted a native debugger UI that wasn't
VSCode, and I wanted to write a GUI in Rust, so here we are.

Inspiration: [Remedybg](https://remedybg.itch.io/remedybg)

## Getting Started

Requirements:

(you can run without these things you just have to modify the build scripts etc)

- Just
- Direnv
- Homebrew installed LLDB

Then run

- `direnv allow`
- `just`

## TODO

- [x] Display a file tree containing the files in the directory in the first command line argument,
      or current working directory if not provided.

- [ ] Clicking on a file in the file tree loads its contents into a file viewer in the main window.

- [ ] Clicking on multiple files opens them in tabs loaded horizontally across the top of the file
      viewer.
