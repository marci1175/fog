; ModuleID = 'main'
source_filename = "main"

declare i32 @puts(ptr)

define i32 @main() {
main_fn_entry:
  %string_buffer = alloca [24 x i8], align 1
  store [24 x i8] c"Hello Hack Club!!!!!!!!\00", ptr %string_buffer, align 1
  %function_call = call i32 @puts(ptr %string_buffer)
  ret i32 0
}
