; ModuleID = 'main'
source_filename = "main"

declare i32 @puts(ptr)

declare i32 @gets(ptr)

define i32 @main() {
main_fn_entry:
  %string_buffer = alloca [22 x i8], align 1
  store [22 x i8] c"asdasdasdasdasdasdasd\00", ptr %string_buffer, align 1
  %function_call = call i32 @gets(ptr %string_buffer)
  %function_call4 = call i32 @puts(ptr %string_buffer)
  ret i32 0
}
