## Quill
A lightweight code editor, built because I wanted to learn how to build one.

## Status
Still being built, trying to get text rendering and highlights to work before 
adding some neovim like features, such as Modes and completions.

## Current Roadmap
1. (will write later)

## Future Features 
1. We can add a viewport where only part of the buffer is ever rendered which can
help with performance by giving a (col_off, row_off) offset cutoff which is what's being
rendered, and the other text can be rendered on demand when it comes within the
viewport.
