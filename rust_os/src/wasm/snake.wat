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
    
    ;; Initialize snake position to center
    (global.set $snake_head_x 
      (i32.div_u (global.get $grid_width) (i32.const 2)))
    (global.set $snake_head_y 
      (i32.div_u (global.get $grid_height) (i32.const 2)))
  )

  ;; Update game state (call this every frame)
  (func (export "game_update")
    (local $new_x i32)
    (local $new_y i32)
    
    ;; Check if game over
    (if (global.get $game_over)
      (then (return)))
    
    ;; Save previous position
    (global.set $prev_snake_x (global.get $snake_head_x))
    (global.set $prev_snake_y (global.get $snake_head_y))
    
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
    
    ;; Update head position
    (global.set $snake_head_x (local.get $new_x))
    (global.set $snake_head_y (local.get $new_y))
    
    ;; Check food collision
    (if (i32.and
          (i32.eq (global.get $snake_head_x) (global.get $food_x))
          (i32.eq (global.get $snake_head_y) (global.get $food_y)))
      (then
        ;; Grow snake
        (global.set $snake_length 
          (i32.add (global.get $snake_length) (i32.const 1)))
        ;; Spawn new food (simple position)
        (global.set $food_x 
          (i32.rem_u 
            (i32.add (global.get $food_x) (i32.const 7))
            (global.get $grid_width)))
        (global.set $food_y 
          (i32.rem_u 
            (i32.add (global.get $food_y) (i32.const 11))
            (global.get $grid_height)))))
  )

  ;; Render the game (only redraw changed cells)
  (func (export "game_render")
    ;; Clear previous snake position (draw black cell)
    (call $draw_cell
      (global.get $prev_snake_x)
      (global.get $prev_snake_y)
      (global.get $CELL_SIZE)
      (global.get $BLACK_R)
      (global.get $BLACK_G)
      (global.get $BLACK_B))
    
    ;; Draw snake head at new position
    (call $draw_cell
      (global.get $snake_head_x)
      (global.get $snake_head_y)
      (global.get $CELL_SIZE)
      (global.get $GREEN_R)
      (global.get $GREEN_G)
      (global.get $GREEN_B))
    
    ;; Draw food
    (call $draw_cell
      (global.get $food_x)
      (global.get $food_y)
      (global.get $CELL_SIZE)
      (global.get $RED_R)
      (global.get $RED_G)
      (global.get $RED_B))
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