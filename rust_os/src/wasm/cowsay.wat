(module
  (import "env" "println" (func $println (param i32 i32)))
  (import "env" "clear_color" (func $clear_color (param i32 i32 i32)))
  (import "env" "reset_cursor" (func $reset_cursor))
  (memory (export "memory") 1)
  
  ;; Static strings
  (data (i32.const 0)  " \\   ^__^")
  (data (i32.const 10) "  \\  (oo)\\_______")
  (data (i32.const 28) "     (__)\\       )\\/\\")
  (data (i32.const 50) "         ||----w |")
  (data (i32.const 68) "         ||     ||")
  
  ;; Working buffer for building lines (starting at 1024)
  (global $input_ptr (mut i32) (i32.const 2048))
  (global $input_len (mut i32) (i32.const 0))
  (global $line_buffer (mut i32) (i32.const 1024))

  ;; Output storage (5 lines)
  (global $output_line1_ptr (mut i32) (i32.const 3072))
  (global $output_line1_len (mut i32) (i32.const 0))
  (global $output_line2_ptr (mut i32) (i32.const 3200))
  (global $output_line2_len (mut i32) (i32.const 0))
  (global $output_line3_ptr (mut i32) (i32.const 3328))
  (global $output_line3_len (mut i32) (i32.const 0))


  ;; Color Black
  (global $BLACK_R i32 (i32.const 0))
  (global $BLACK_G i32 (i32.const 0))
  (global $BLACK_B i32 (i32.const 0))
  
  ;; Default message
  (data (i32.const 200) "Hello! Type something...")
  
  (func (export "game_init")
    (global.set $input_len (i32.const 24))
    (memory.copy 
      (global.get $input_ptr)
      (i32.const 200)
      (i32.const 24))
    (call $build_output)
  )
  
  (func (export "game_update") nop)
  
  (func (export "game_render") nop)
  (func $build_output
    (local $i i32)
    (local $msg_len i32)
    (local $buf_pos i32)
    
    (local.set $msg_len (global.get $input_len))
    (local.set $buf_pos (global.get $line_buffer))
    

	(call $reset_cursor)
    ;;(call $clear_color 
	;;   (global.get $BLACK_R)
	;;   (global.get $BLACK_G)
	;;   (global.get $BLACK_B))
    ;; Build top border line: " " + "_" repeated
    (i32.store8 (local.get $buf_pos) (i32.const 32))  ;; space
    (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
    
    (local.set $i (i32.const 0))
    (block $done1
      (loop $loop1
        (br_if $done1 (i32.ge_u (local.get $i) (i32.add (local.get $msg_len) (i32.const 2))))
        (i32.store8 (local.get $buf_pos) (i32.const 95))  ;; underscore
        (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop1)
      )
    )
    
    ;; Print top border
    (call $println 
      (global.get $line_buffer)
      (i32.sub (local.get $buf_pos) (global.get $line_buffer)))
    
    ;; Build message line: "< " + message + " >"
    (local.set $buf_pos (global.get $line_buffer))
    (i32.store8 (local.get $buf_pos) (i32.const 60))  ;; 
    (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
    (i32.store8 (local.get $buf_pos) (i32.const 32))  ;; space
    (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
    
    ;; Copy message
    (memory.copy
      (local.get $buf_pos)
      (global.get $input_ptr)
      (local.get $msg_len))
    (local.set $buf_pos (i32.add (local.get $buf_pos) (local.get $msg_len)))
    
    (i32.store8 (local.get $buf_pos) (i32.const 32))  ;; space
    (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
    (i32.store8 (local.get $buf_pos) (i32.const 62))  ;; >
    (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
    
    ;; Print message line
    (call $println 
      (global.get $line_buffer)
      (i32.sub (local.get $buf_pos) (global.get $line_buffer)))
    
    ;; Build bottom border: " " + "-" repeated
    (local.set $buf_pos (global.get $line_buffer))
    (i32.store8 (local.get $buf_pos) (i32.const 32))  ;; space
    (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
    
    (local.set $i (i32.const 0))
    (block $done2
      (loop $loop2
        (br_if $done2 (i32.ge_u (local.get $i) (i32.add (local.get $msg_len) (i32.const 2))))
        (i32.store8 (local.get $buf_pos) (i32.const 45))  ;; dash
        (local.set $buf_pos (i32.add (local.get $buf_pos) (i32.const 1)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop2)
      )
    )
    
    ;; Print bottom border
    (call $println 
      (global.get $line_buffer)
      (i32.sub (local.get $buf_pos) (global.get $line_buffer)))
    
    ;; Print cow lines (they're already complete strings)
    (call $println (i32.const 0) (i32.const 9))
    (call $println (i32.const 10) (i32.const 17))
    (call $println (i32.const 28) (i32.const 21))
    (call $println (i32.const 50) (i32.const 18))
    (call $println (i32.const 68) (i32.const 18))
  )
  
  (func (export "handle_key") (param $key i32)
    (local $current_len i32)
    (local.set $current_len (global.get $input_len))
    
    ;; Backspace
    (if (i32.eq (local.get $key) (i32.const 8))
      (then
        (if (i32.gt_u (local.get $current_len) (i32.const 0))
          (then
            (global.set $input_len (i32.sub (local.get $current_len) (i32.const 1)))
			(call $build_output)
          )
        )
        (return)
      )
    )
    
    ;; Enter - clear
    (if (i32.eq (local.get $key) (i32.const 10))
      (then
        (global.set $input_len (i32.const 0))
		(call $clear_color 
		(global.get $BLACK_R)
		(global.get $BLACK_G)
		(global.get $BLACK_B))
		(call $build_output)
        (return)
      )
    )
    
    ;; Regular character
    (if (i32.and 
          (i32.ge_u (local.get $key) (i32.const 32))
          (i32.le_u (local.get $key) (i32.const 126)))
      (then
        (if (i32.lt_u (local.get $current_len) (i32.const 100))
          (then
            (i32.store8 
              (i32.add (global.get $input_ptr) (local.get $current_len))
              (local.get $key))
            (global.set $input_len (i32.add (local.get $current_len) (i32.const 1)))
			(call $build_output)
          )
        )
      )
    )
  )
  
  (func (export "set_direction") (param i32) nop)
)
