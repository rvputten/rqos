To do
=====
- aliases
  - j
  - git diff --color=always
  - alias command?
    - settings for loading and saving aliases?
- load and save and edit text
- select text with mouse
- copy and paste
- password entry ("su -")
  - possibly requires libc or a crate
- split app.rs into multiple files
  - app.rs
  - app/input.rs
  - app/window.rs
- redirect output with '>'
- 'clear' (Ansi code?)

1. gfx-rs:
   - Complexity: High, due to its bindless design.
   - Comprehensiveness: High, as it aims to be the default API for Rust graphics.
   - Elegance: Medium, as it provides a lot of control but can be verbose.
   - Lightweight: Medium, as it provides a lot of functionality but isn't the most
     minimal option.

2. wgpu-rs:
   - Complexity: Medium, as it provides a lot of functionality but is designed to
     be easy to use.
   - Comprehensiveness: High, as it's designed for general purpose graphics and
     computation.
   - Elegance: High, as it's designed to be idiomatic Rust.
   - Lightweight: Medium, as it provides a lot of functionality but isn't the most
     minimal option.

3. vulkano:
   - Complexity: High, as it closely follows the Vulkan specification.
   - Comprehensiveness: High, as it provides a lot of control over the graphics
     pipeline.
   - Elegance: Medium, as it provides a lot of control but can be verbose.
   - Lightweight: Low, as it provides a lot of functionality and control.

4. ash:
   - Complexity: High, as it's a low-level wrapper around Vulkan.
   - Comprehensiveness: High, as it provides a lot of control over the graphics
     pipeline.
   - Elegance: Low, as it's a low-level API and can be verbose.
   - Lightweight: Low, as it provides a lot of functionality and control.

5. rendy:
   - Complexity: Medium, as it's a high-level framework built on top of gfx-hal.
   - Comprehensiveness: Medium, as it's still under heavy development.
   - Elegance: High, as it's designed to be easy to use.
   - Lightweight: Medium, as it provides a lot of functionality but isn't the most
     minimal option.

6. luminance:
   - Complexity: Low, as it's a type-safe, type-level and stateless framework.
   - Comprehensiveness: Medium, as it provides a simple way to perform graphics
     computations.
   - Elegance: High, as it's designed to be idiomatic Rust.
   - Lightweight: High, as it's stateless and designed to be minimal.

gfx-rs
wgpu-rs
vulkano
ash
rendy
luminance
