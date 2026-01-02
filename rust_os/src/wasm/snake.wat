;; Snake Game in WebAssembly
;; This calls the Rust framebuffer host functions

(module
  ;; Import host functions from Rust
  (import "env" "put_pixel" (func $put_pixel (param i32 i32 i32 i32 i32)))
  (import "env" "draw_cell" (func $draw_cell (param i32 i32 i32 i32 i32 i32)))
  (import "env" "clear_color" (func $clear_color (param i32 i32 i32)))
  (import "env" "get_grid_width" (func $get_grid_width (result i32)))
  (import "env" "get_grid_height" (func $get_grid_height (result i32)))
  (import "env" "init_cell_size" (func $init_cell_size (param i32)))

  ;; Memory for game state
  (memory (export "memory") 1)

  ;; Game state globals
  (global $snake_length (mut i32) (i32.const 3))
  (global $snake_head_x (mut i32) (i32.const 10))
  (global $snake_head_y (mut i32) (i32.const 10))
  (global $prev_snake_x (mut i32) (i32.const 10))
  (global $prev_snake_y (mut i32) (i32.const 10))
  (global $direction (mut i32) (i32.const 0)) ;; 0=right, 1=down, 2=left, 3=up
  (global $food_x (mut i32) (i32.const 5))
  (global $food_y (mut i32) (i32.const 5))
  (global $grid_width (mut i32) (i32.const 0))
  (global $grid_height (mut i32) (i32.const 0))
  (global $game_over (mut i32) (i32.const 0))
  (global $tail_index (mut i32) (i32.const 0))  ;; Circular buffer index

  ;; Constants
  (global $CELL_SIZE i32 (i32.const 10))
  (global $BLACK_R i32 (i32.const 0))
  (global $BLACK_G i32 (i32.const 0))
  (global $BLACK_B i32 (i32.const 0))
  (global $GREEN_R i32 (i32.const 0))
  (global $GREEN_G i32 (i32.const 160))
  (global $GREEN_B i32 (i32.const 0))
  (global $RED_R i32 (i32.const 255))
  (global $RED_G i32 (i32.const 0))
  (global $RED_B i32 (i32.const 0))
  (global $DARK_GREEN_R i32 (i32.const 0))
  (global $DARK_GREEN_G i32 (i32.const 140))
  (global $DARK_GREEN_B i32 (i32.const 0))
  (global $WHITE_R i32 (i32.const 255))
  (global $WHITE_G i32 (i32.const 255))
  (global $WHITE_B i32 (i32.const 255))

  (global $game_over_drawn (mut i32) (i32.const 0)) ;; 0 = not drawn yet, 1 = already drawn
  (global $MAX_SEGMENTS i32 (i32.const 1024))

(func $get_segment_x (param $index i32) (result i32)
  (i32.load (i32.mul (local.get $index) (i32.const 8))))

;; Get segment Y coordinate  
(func $get_segment_y (param $index i32) (result i32)
  (i32.load (i32.add (i32.mul (local.get $index) (i32.const 8)) (i32.const 4))))

;; Set segment coordinates
(func $set_segment (param $index i32) (param $x i32) (param $y i32)
  (i32.store (i32.mul (local.get $index) (i32.const 8)) (local.get $x))
  (i32.store (i32.add (i32.mul (local.get $index) (i32.const 8)) (i32.const 4)) (local.get $y)))
(func $draw_headline
  (local $x0 i32)
  (local $y0 i32)

  ;; top-left position (in grid cells)
  (local.set $y0 (i32.const 2))
  (local.set $x0 (i32.const 3))

  ;; =======================
  ;; S  (5x5)
  ;; 11111
  ;; 10000
  ;; 11110
  ;; 00001
  ;; 11111
  ;; =======================
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  ;; advance x by 6 (5 cols + 1 space)
  (local.set $x0 (i32.add (local.get $x0) (i32.const 6)))

  ;; =======================
  ;; N (5x5)  <-- this is what fixes your H problem
  ;; 10001
  ;; 11001
  ;; 10101
  ;; 10011
  ;; 10001
  ;; =======================
  ;; left column
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  ;; diagonal
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  ;; right column
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  ;; advance
  (local.set $x0 (i32.add (local.get $x0) (i32.const 6)))

  ;; =======================
  ;; A (5x5)
  ;; 01110
  ;; 10001
  ;; 11111
  ;; 10001
  ;; 10001
  ;; =======================
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  (local.set $x0 (i32.add (local.get $x0) (i32.const 6)))

  ;; =======================
  ;; K (5x5)
  ;; 10001
  ;; 10010
  ;; 11100
  ;; 10010
  ;; 10001
  ;; =======================
  ;; left column
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  ;; upper arm + lower arm + mid
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  (local.set $x0 (i32.add (local.get $x0) (i32.const 6)))

  ;; =======================
  ;; E (5x5)
  ;; 11111
  ;; 10000
  ;; 11110
  ;; 10000
  ;; 11111
  ;; =======================
  ;; top row
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 0)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  ;; left column
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 1)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  ;; middle row
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 2)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 3)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))

  ;; bottom row
  (call $draw_cell (i32.add (local.get $x0) (i32.const 0)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 1)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 2)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 3)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
  (call $draw_cell (i32.add (local.get $x0) (i32.const 4)) (i32.add (local.get $y0) (i32.const 4)) (global.get $CELL_SIZE) (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B))
)


;; ============================================================
;; 5x5 DOT FONT
;; row-major index: (0,0)=0, (1,0)=1, ... (4,0)=4, (0,1)=5, ... (4,4)=24
;; each glyph = 25 bytes (0/1)
;; glyph ids:
;; 0..9 digits
;; 10 'S'
;; 11 'C'
;; 12 'O'
;; 13 'R'
;; 14 'E'
;; 15 ':' (colon)
;; ============================================================

(global $GLYPH_STRIDE i32 (i32.const 25))
(global $GLYPHS_BASE  i32 (i32.const 8192))

(data (i32.const 8192)
  ;; ---------------- digits 0..9 ----------------
  ;; 0
  "\01\01\01\01\01"
  "\01\00\00\00\01"
  "\01\00\00\00\01"
  "\01\00\00\00\01"
  "\01\01\01\01\01"

  ;; 1
  "\00\00\01\00\00"
  "\00\01\01\00\00"
  "\00\00\01\00\00"
  "\00\00\01\00\00"
  "\01\01\01\01\01"

  ;; 2
  "\01\01\01\01\01"
  "\00\00\00\00\01"
  "\01\01\01\01\01"
  "\01\00\00\00\00"
  "\01\01\01\01\01"

  ;; 3
  "\01\01\01\01\01"
  "\00\00\00\00\01"
  "\00\01\01\01\01"
  "\00\00\00\00\01"
  "\01\01\01\01\01"

  ;; 4
  "\01\00\00\00\01"
  "\01\00\00\00\01"
  "\01\01\01\01\01"
  "\00\00\00\00\01"
  "\00\00\00\00\01"

  ;; 5
  "\01\01\01\01\01"
  "\01\00\00\00\00"
  "\01\01\01\01\01"
  "\00\00\00\00\01"
  "\01\01\01\01\01"

  ;; 6
  "\01\01\01\01\01"
  "\01\00\00\00\00"
  "\01\01\01\01\01"
  "\01\00\00\00\01"
  "\01\01\01\01\01"

  ;; 7
  "\01\01\01\01\01"
  "\00\00\00\00\01"
  "\00\00\00\01\00"
  "\00\00\01\00\00"
  "\00\01\00\00\00"

  ;; 8
  "\01\01\01\01\01"
  "\01\00\00\00\01"
  "\01\01\01\01\01"
  "\01\00\00\00\01"
  "\01\01\01\01\01"

  ;; 9
  "\01\01\01\01\01"
  "\01\00\00\00\01"
  "\01\01\01\01\01"
  "\00\00\00\00\01"
  "\01\01\01\01\01"

  ;; ---------------- letters for "SCORE:" ----------------
  ;; 10 = 'S'
  "\01\01\01\01\01"
  "\01\00\00\00\00"
  "\01\01\01\01\00"
  "\00\00\00\00\01"
  "\01\01\01\01\01"

  ;; 11 = 'C'
  "\01\01\01\01\01"
  "\01\00\00\00\00"
  "\01\00\00\00\00"
  "\01\00\00\00\00"
  "\01\01\01\01\01"

  ;; 12 = 'O'
  "\01\01\01\01\01"
  "\01\00\00\00\01"
  "\01\00\00\00\01"
  "\01\00\00\00\01"
  "\01\01\01\01\01"

  ;; 13 = 'R'
  "\01\01\01\01\00"
  "\01\00\00\00\01"
  "\01\01\01\01\00"
  "\01\00\00\01\00"
  "\01\00\00\00\01"

  ;; 14 = 'E'
  "\01\01\01\01\01"
  "\01\00\00\00\00"
  "\01\01\01\01\00"
  "\01\00\00\00\00"
  "\01\01\01\01\01"

  ;; 15 = ':'
  "\00\00\00\00\00"
  "\00\00\01\00\00"
  "\00\00\00\00\00"
  "\00\00\01\00\00"
  "\00\00\00\00\00"
)

(func $glyph_bit (param $g i32) (param $x i32) (param $y i32) (result i32)
  (local $addr i32)
  (local.set $addr
    (i32.add
      (global.get $GLYPHS_BASE)
      (i32.add
        (i32.mul (local.get $g) (global.get $GLYPH_STRIDE))
        (i32.add (i32.mul (local.get $y) (i32.const 5)) (local.get $x)))))
  (i32.load8_u (local.get $addr))
)

(func $draw_glyph5x5 (param $g i32) (param $x0 i32) (param $y0 i32)
  (local $x i32)
  (local $y i32)

  (local.set $y (i32.const 0))
  (block $ybreak
    (loop $yloop
      (br_if $ybreak (i32.ge_u (local.get $y) (i32.const 5)))

      (local.set $x (i32.const 0))
      (block $xbreak
        (loop $xloop
          (br_if $xbreak (i32.ge_u (local.get $x) (i32.const 5)))

          (if (call $glyph_bit (local.get $g) (local.get $x) (local.get $y))
            (then
              (call $draw_cell
                (i32.add (local.get $x0) (local.get $x))
                (i32.add (local.get $y0) (local.get $y))
                (global.get $CELL_SIZE)
                (global.get $WHITE_R) (global.get $WHITE_G) (global.get $WHITE_B)
              )
            )
          )

          (local.set $x (i32.add (local.get $x) (i32.const 1)))
          (br $xloop)
        )
      )

      (local.set $y (i32.add (local.get $y) (i32.const 1)))
      (br $yloop)
    )
  )
)

;; Clear a rectangle in *grid cells* (not pixels)
(func $clear_rect_cells (param $x0 i32) (param $y0 i32) (param $w i32) (param $h i32)
  (local $x i32)
  (local $y i32)

  (local.set $y (i32.const 0))
  (block $ybreak
    (loop $yloop
      (br_if $ybreak (i32.ge_u (local.get $y) (local.get $h)))

      (local.set $x (i32.const 0))
      (block $xbreak
        (loop $xloop
          (br_if $xbreak (i32.ge_u (local.get $x) (local.get $w)))

          (call $draw_cell
            (i32.add (local.get $x0) (local.get $x))
            (i32.add (local.get $y0) (local.get $y))
            (global.get $CELL_SIZE)
            (global.get $BLACK_R) (global.get $BLACK_G) (global.get $BLACK_B)
          )

          (local.set $x (i32.add (local.get $x) (i32.const 1)))
          (br $xloop)
        )
      )

      (local.set $y (i32.add (local.get $y) (i32.const 1)))
      (br $yloop)
    )
  )
)

;; Draw a number using digit glyphs (0..9) from the same table
;; (same as before, but now uses $draw_glyph5x5 with glyph id == digit)
(func $draw_number5x5 (param $n i32) (param $x0 i32) (param $y0 i32)
  (local $tmp i32)
  (local $d0 i32) (local $d1 i32) (local $d2 i32) (local $d3 i32)
  (local $started i32)
  (local $x i32)

  (local.set $tmp (local.get $n))
  (if (i32.gt_s (local.get $tmp) (i32.const 9999)) (then (local.set $tmp (i32.const 9999))))
  (if (i32.lt_s (local.get $tmp) (i32.const 0))    (then (local.set $tmp (i32.const 0))))

  (local.set $d0 (i32.rem_u (local.get $tmp) (i32.const 10)))
  (local.set $tmp (i32.div_u (local.get $tmp) (i32.const 10)))
  (local.set $d1 (i32.rem_u (local.get $tmp) (i32.const 10)))
  (local.set $tmp (i32.div_u (local.get $tmp) (i32.const 10)))
  (local.set $d2 (i32.rem_u (local.get $tmp) (i32.const 10)))
  (local.set $tmp (i32.div_u (local.get $tmp) (i32.const 10)))
  (local.set $d3 (i32.rem_u (local.get $tmp) (i32.const 10)))

  (local.set $started (i32.const 0))
  (local.set $x (local.get $x0))

  (if (i32.or (local.get $started) (i32.ne (local.get $d3) (i32.const 0)))
    (then
      (call $draw_glyph5x5 (local.get $d3) (local.get $x) (local.get $y0))
      (local.set $x (i32.add (local.get $x) (i32.const 6)))
      (local.set $started (i32.const 1))
    )
  )

  (if (i32.or (local.get $started) (i32.ne (local.get $d2) (i32.const 0)))
    (then
      (call $draw_glyph5x5 (local.get $d2) (local.get $x) (local.get $y0))
      (local.set $x (i32.add (local.get $x) (i32.const 6)))
      (local.set $started (i32.const 1))
    )
  )

  (if (i32.or (local.get $started) (i32.ne (local.get $d1) (i32.const 0)))
    (then
      (call $draw_glyph5x5 (local.get $d1) (local.get $x) (local.get $y0))
      (local.set $x (i32.add (local.get $x) (i32.const 6)))
      (local.set $started (i32.const 1))
    )
  )

  (call $draw_glyph5x5 (local.get $d0) (local.get $x) (local.get $y0))
)

;; Draw "SCORE:" + score value (here: snake_length)
(func $draw_score (param $x0 i32) (param $y0 i32)
  (local $x i32)
  (local $score i32)

  ;; score = snake_length (change to (snake_length - 3) if you want)
  (local.set $score (global.get $snake_length))

  ;; clear enough area: "SCORE:" is 6 glyphs * 6 cols = 36 cells,
  ;; plus up to 4 digits * 6 = 24 => total ~60 wide, 5 high
  (call $clear_rect_cells (local.get $x0) (local.get $y0) (i32.const 60) (i32.const 5))

  (local.set $x (local.get $x0))

  ;; S C O R E :
  (call $draw_glyph5x5 (i32.const 10) (local.get $x) (local.get $y0)) (local.set $x (i32.add (local.get $x) (i32.const 6)))
  (call $draw_glyph5x5 (i32.const 11) (local.get $x) (local.get $y0)) (local.set $x (i32.add (local.get $x) (i32.const 6)))
  (call $draw_glyph5x5 (i32.const 12) (local.get $x) (local.get $y0)) (local.set $x (i32.add (local.get $x) (i32.const 6)))
  (call $draw_glyph5x5 (i32.const 13) (local.get $x) (local.get $y0)) (local.set $x (i32.add (local.get $x) (i32.const 6)))
  (call $draw_glyph5x5 (i32.const 14) (local.get $x) (local.get $y0)) (local.set $x (i32.add (local.get $x) (i32.const 6)))
  (call $draw_glyph5x5 (i32.const 15) (local.get $x) (local.get $y0)) (local.set $x (i32.add (local.get $x) (i32.const 6)))

  ;; number right after "SCORE:"
  (call $draw_number5x5 (local.get $score) (local.get $x) (local.get $y0))
)


  ;; Initialize the game
  (func (export "game_init")
    ;; Set cell size
    (call $init_cell_size (global.get $CELL_SIZE))

    
    ;; Get grid dimensions
    (global.set $grid_width (call $get_grid_width))
    (global.set $grid_height (call $get_grid_height))
    
    ;; Clear screen to black
    (call $clear_color 
      (global.get $BLACK_R)
      (global.get $BLACK_G)
      (global.get $BLACK_B))
	(call $draw_headline)
    
    ;; Initialize snake position to center
    (global.set $snake_head_x 
      (i32.div_u (global.get $grid_width) (i32.const 2)))
    (global.set $snake_head_y 
      (i32.div_u (global.get $grid_height) (i32.const 2)))
	(call $set_segment (i32.const 0) (global.get $snake_head_x) (global.get $snake_head_y))
	(call $set_segment (i32.const 1) (global.get $snake_head_x) (global.get $snake_head_y))
	(call $set_segment (i32.const 2) (global.get $snake_head_x) (global.get $snake_head_y))
  )

  ;; Update game state (call this every frame)
	(func (export "game_update")
	(local $new_x i32)
	(local $new_y i32)
	(local $ate_food i32)
	(local $old_tail_x i32)
	(local $old_tail_y i32)
	(local $j i32)
	(local $idx i32)
	(local $sx i32)
	(local $sy i32)
	
	;; Check if game over
	(if (global.get $game_over)

		(then
			(return)))
	
	;; Calculate new head position based on direction
	(local.set $new_x (global.get $snake_head_x))
	(local.set $new_y (global.get $snake_head_y))
	
	;; Direction: 0=right, 1=down, 2=left, 3=up
	(block $direction_check
		(block $case_3
		(block $case_2
			(block $case_1
			(block $case_0
				(br_table $case_0 $case_1 $case_2 $case_3
				(global.get $direction))
			)
			;; Case 0: Right
			(local.set $new_x (i32.add (local.get $new_x) (i32.const 1)))
			(br $direction_check)
			)
			;; Case 1: Down
			(local.set $new_y (i32.add (local.get $new_y) (i32.const 1)))
			(br $direction_check)
		)
		;; Case 2: Left
		(local.set $new_x (i32.sub (local.get $new_x) (i32.const 1)))
		(br $direction_check)
		)
		;; Case 3: Up
		(local.set $new_y (i32.sub (local.get $new_y) (i32.const 1)))
	)
	;; Check wall collision
	(if (i32.or
			(i32.or
			(i32.lt_s (local.get $new_x) (i32.const 0))
			(i32.ge_u (local.get $new_x) (global.get $grid_width)))
			(i32.or
			(i32.lt_s (local.get $new_y) (i32.const 0))
			(i32.ge_u (local.get $new_y) (global.get $grid_height))))
		(then
		(global.set $game_over (i32.const 1))
		(return)))
		(local.set $j (i32.const 0))
(block $self_ok
  (loop $self_loop
    (br_if $self_ok (i32.ge_u (local.get $j) (global.get $snake_length)))

    ;; idx = (tail_index + j) % MAX_SEGMENTS
    (local.set $idx
      (i32.rem_u
        (i32.add (global.get $tail_index) (local.get $j))
        (global.get $MAX_SEGMENTS)))

    (local.set $sx (call $get_segment_x (local.get $idx)))
    (local.set $sy (call $get_segment_y (local.get $idx)))

    (if (i32.and
          (i32.eq (local.get $sx) (local.get $new_x))
          (i32.eq (local.get $sy) (local.get $new_y)))
      (then
        (global.set $game_over (i32.const 1))
        (return)
      )
    )

    (local.set $j (i32.add (local.get $j) (i32.const 1)))
    (br $self_loop)
  )
)
		

	
	;; Save OLD tail position BEFORE we potentially move it
	(local.set $old_tail_x (call $get_segment_x (global.get $tail_index)))
	(local.set $old_tail_y (call $get_segment_y (global.get $tail_index)))
	
	;; Update head position
	(global.set $snake_head_x (local.get $new_x))
	(global.set $snake_head_y (local.get $new_y))
	
	;; Did we eat?
	(local.set $ate_food
		(i32.and
		(i32.eq (global.get $snake_head_x) (global.get $food_x))
		(i32.eq (global.get $snake_head_y) (global.get $food_y))))

	;; If we did NOT eat: drop tail by moving tail_index forward
	(if (i32.eqz (local.get $ate_food))
		(then
		(global.set $tail_index
			(i32.rem_u
			(i32.add (global.get $tail_index) (i32.const 1))
			(global.get $MAX_SEGMENTS)))
		))

	;; If we ate: grow length (ONLY here)
	(if (local.get $ate_food)
		(then
		(global.set $snake_length
			(i32.add (global.get $snake_length) (i32.const 1)))

		;; respawn food (ONLY here)
		(global.set $food_x
			(i32.rem_u (i32.add (global.get $food_x) (i32.const 7)) (global.get $grid_width)))
		(global.set $food_y
			(i32.rem_u (i32.add (global.get $food_y) (i32.const 11)) (global.get $grid_height)))
		))

	;; Write new head at end of snake in circular buffer
	(call $set_segment
		(i32.rem_u
		(i32.add
			(global.get $tail_index)
			(i32.sub (global.get $snake_length) (i32.const 1)))
		(global.get $MAX_SEGMENTS))
		(global.get $snake_head_x)
		(global.get $snake_head_y))
	
	;; Save old tail for rendering
	(global.set $prev_snake_x (local.get $old_tail_x))
	(global.set $prev_snake_y (local.get $old_tail_y))
)

;; Render the game (only redraw changed cells)
(func (export "game_render")
  (local $i i32)
  (local $seg_x i32)
  (local $seg_y i32)
  (local $tail_x i32)
  (local $tail_y i32)
  (local $should_clear_tail i32)
  
  ;; Check if tail position changed (if it changed, we didn't eat food)
  (local.set $tail_x (call $get_segment_x (global.get $tail_index)))
  (local.set $tail_y (call $get_segment_y (global.get $tail_index)))
  
  ;; Only clear tail if it's not part of the current snake
  ;; We need to clear the OLD tail position that's no longer part of snake
  ;; This happens when tail_index moved forward (didn't eat food)
  (call $draw_cell
    (global.get $prev_snake_x)
    (global.get $prev_snake_y)
    (global.get $CELL_SIZE)
    (global.get $BLACK_R)
    (global.get $BLACK_G)
    (global.get $BLACK_B))
  
  ;; Draw entire snake body
  (local.set $i (i32.const 0))
  (block $break
    (loop $continue
      ;; Exit if we've drawn all segments
      (br_if $break (i32.ge_u (local.get $i) (global.get $snake_length)))
      
      ;; Get segment position from circular buffer
      (local.set $seg_x (call $get_segment_x 
        (i32.rem_u 
          (i32.add (global.get $tail_index) (local.get $i))
          (global.get $MAX_SEGMENTS))))
      (local.set $seg_y (call $get_segment_y 
        (i32.rem_u 
          (i32.add (global.get $tail_index) (local.get $i))
          (global.get $MAX_SEGMENTS))))
      
      ;; Draw segment
      (call $draw_cell
        (local.get $seg_x)
        (local.get $seg_y)
        (global.get $CELL_SIZE)
        (global.get $GREEN_R)
        (global.get $GREEN_G)
        (global.get $GREEN_B))
      
      ;; Increment counter
      (local.set $i (i32.add (local.get $i) (i32.const 1)))
      (br $continue)
    )
  )
  
  ;; Draw food
  (call $draw_cell
    (global.get $food_x)
    (global.get $food_y)
    (global.get $CELL_SIZE)
    (global.get $RED_R)
    (global.get $RED_G)
    (global.get $RED_B))
(if (global.get $game_over)
  (then
    (if (i32.eqz (global.get $game_over_drawn))
      (then
        ;; mark as drawn
        (global.set $game_over_drawn (i32.const 1))

        ;; draw "SCORE:" + length at some position
        ;; pick coords that look good under your headline
        (call $draw_score (i32.const 3) (i32.const 8))
      )
    )
  )
)
)

;; Reset the game to initial state
(func $game_reset (export "game_reset")
  ;; Reset game state
  (global.set $game_over_drawn (i32.const 0))

  (global.set $game_over (i32.const 0))
  (global.set $snake_length (i32.const 3))
  (global.set $direction (i32.const 0))
  (global.set $tail_index (i32.const 0))
  
  ;; Reset snake position to center
  (global.set $snake_head_x 
    (i32.div_u (global.get $grid_width) (i32.const 2)))
  (global.set $snake_head_y 
    (i32.div_u (global.get $grid_height) (i32.const 2)))
  
  ;; Reset food position
  (global.set $food_x (i32.const 5))
  (global.set $food_y (i32.const 5))
  
  ;; Initialize snake segments
  (call $set_segment (i32.const 0) (global.get $snake_head_x) (global.get $snake_head_y))
  (call $set_segment (i32.const 1) (global.get $snake_head_x) (global.get $snake_head_y))
  (call $set_segment (i32.const 2) (global.get $snake_head_x) (global.get $snake_head_y))
  
  ;; Clear screen and redraw
  (call $clear_color 
    (global.get $BLACK_R)
    (global.get $BLACK_G)
    (global.get $BLACK_B))
  (call $draw_headline)
)

  ;; Internal function to set direction (0=right, 1=down, 2=left, 3=up)
  (func $set_direction_internal (param $dir i32)
    (local $opposite i32)
    
    ;; Calculate opposite direction
    (local.set $opposite
      (i32.xor (global.get $direction) (i32.const 2)))
    
    ;; Only change if not opposite
    (if (i32.ne (local.get $dir) (local.get $opposite))
      (then
        (global.set $direction (local.get $dir)))))

  ;; Set direction (exported for external use)
  (func (export "set_direction") (param $dir i32)
    (call $set_direction_internal (local.get $dir)))

  ;; Handle keyboard input directly
  (func (export "handle_key") (param $key i32)
	(if (global.get $game_over)
		(then 
			(call $game_reset)
			(return)))

    ;; Arrow keys (scan codes)
    (if (i32.eq (local.get $key) (i32.const 103)) ;; Right arrow
      (then (call $set_direction_internal (i32.const 0))))
    (if (i32.eq (local.get $key) (i32.const 102)) ;; Down arrow  
      (then (call $set_direction_internal (i32.const 1))))
    (if (i32.eq (local.get $key) (i32.const 101)) ;; Left arrow
      (then (call $set_direction_internal (i32.const 2))))
    (if (i32.eq (local.get $key) (i32.const 88)) ;; Up arrow
      (then (call $set_direction_internal (i32.const 3))))
    
    ;; WASD keys (ASCII codes)
    (if (i32.or (i32.eq (local.get $key) (i32.const 100)) ;; 'd'
                (i32.eq (local.get $key) (i32.const 68)))  ;; 'D'
      (then (call $set_direction_internal (i32.const 0))))
    (if (i32.or (i32.eq (local.get $key) (i32.const 115)) ;; 's'
                (i32.eq (local.get $key) (i32.const 83)))  ;; 'S'
      (then (call $set_direction_internal (i32.const 1))))
    (if (i32.or (i32.eq (local.get $key) (i32.const 97))  ;; 'a'
                (i32.eq (local.get $key) (i32.const 65)))  ;; 'A'
      (then (call $set_direction_internal (i32.const 2))))
    (if (i32.or (i32.eq (local.get $key) (i32.const 119)) ;; 'w'
                (i32.eq (local.get $key) (i32.const 87)))  ;; 'W'
      (then (call $set_direction_internal (i32.const 3))))
  )

  ;; Check if game is over
  (func (export "is_game_over") (result i32)
    (global.get $game_over))
)
