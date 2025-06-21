; ModuleID = 'main'
source_filename = "main"

declare i32 @printf(ptr, i32)

define i32 @main() {
main_fn_entry:
  %string_buffer3 = alloca [16 x i8], align 1
  store [16 x i8] c"User input: %i\0A\00", ptr %string_buffer3, align 1
  %buf_ptr4 = getelementptr [16 x i8], ptr %string_buffer3, i32 0, i32 0
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %main_fn_entry
  %function_call = call i32 @printf(ptr %buf_ptr4, i32 23)
  %0 = alloca i32, align 4
  store i32 3, ptr %0, align 4
  %1 = alloca i32, align 4
  store i32 1, ptr %1, align 4
  %lhs = load i32, ptr %0, align 4
  %rhs = load i32, ptr %1, align 4
  %int_add_int = add i32 %lhs, %rhs
  %function_call7 = call i32 @printf(ptr %buf_ptr4, i32 %int_add_int)
  br label %loop_body
}
