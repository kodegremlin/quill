# Quill
A lightweight code editor, built because I wanted to learn how to build one.

## Status
Still being built, trying to get text rendering and highlights to work before 
adding some neovim like features, such as Modes and completions.

## Roadmap
0. Provide proper documentation.
1. ~~Syntax Highlighting.~~ ***Done***
2. ~~Add logging ( use the `log` crate ).~~
3. Use the definition field, that helps highlight function names, to highlight `enum`, `struct`, etc. identifiers. 

## Features in Current Scope (Not in any particular order).
1. ~~Implement a efficient span based syntax highlighting.~~ ***Done***
    - After a basic span based highlighter, integrate it with tree-sitter for better highlighting.
2. Basic vim modes and operations they control.
3. Auto completion:
    - First, just provide completion based on history, without necessarily a popup but rather a text replacement type thing.
    - After it works, we can introduce popups (or maybe do it together only, we'll see).
    - Second, after the history-buffer based completion we can see if the integration with tree-sitter can help in providing some smart syntax based completion even if relying on history. 

## Future Features (In no particular order).
1. If the language is recognised, provide smart code traversal based on semantic meaning (maybe tree-sitter helps with this).
2. See if core vim grammar can also be introduced if it doesn't introduce too much complexity.
3. Look into undo trees like vim, instead of destructive linear history for mangaging parallel diffs.
