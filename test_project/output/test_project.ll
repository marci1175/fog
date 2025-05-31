; ModuleID = 'main'
source_filename = "main"

declare i32 @puts(ptr)

define i32 @main() {
main_fn_entry:
  %msg = alloca ptr, align 8
  %string_buffer = alloca [13 x i8], align 1
  store [13 x i8] c"Hello World!\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %msg, align 8
  %msg1 = load ptr, ptr %msg, align 8
  %function_call = call i32 @puts(ptr %msg1)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var2
}
