; ModuleID = 'main'
source_filename = "main"

declare i32 @print(ptr)

define i32 @main() {
main_fn_entry:
  %string_buffer = alloca [13 x i8], align 1
  store [13 x i8] c"Hello World!\00", ptr %string_buffer, align 1
  %function_call = call i32 @print(ptr %string_buffer)
  ret i32 0
}
