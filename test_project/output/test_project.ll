; ModuleID = 'main'
source_filename = "main"

declare i32 @printf(ptr, i32)

define i32 @main() {
main_fn_entry:
  %string_buffer1 = alloca [16 x i8], align 1
  store [16 x i8] c"User input: %i\0A\00", ptr %string_buffer1, align 1
  %buf_ptr2 = getelementptr [16 x i8], ptr %string_buffer1, i32 0, i32 0
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %main_fn_entry
  %function_call = call i32 @printf(ptr %buf_ptr2, i32 3)
  br label %loop_body
}
