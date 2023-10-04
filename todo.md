To do
=====
- colors in Text
  - escape sequences imply cursor position changes
- nicer presentation of command outputs
  - invert colors of command line
    - may be multiple lines
  - return code and last command in status line
- history with ESC/j/k
  - move command output with j/k
    - record positions in scroll buffer for every command
- tab completion
- completion of files not in the local directory
- better parsing of command line
  - double quotes
    - for git commit message
- aliases
  - j
  - git diff --color=always
  - alias command?
    - settings for loading and saving aliases?
- variable expansion
- load and save and edit text
- don't scan the directory if no * or ? contained
- make C-W work in command mode and switch to insert mode
  - quick path to editing the command line

font editor
-----------
check correct usage of `char_idx/display_idx` in `font::copy_char, set_pixel` etc.
