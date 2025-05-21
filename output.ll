; ModuleID = 'main'
source_filename = "main"

declare i32 @getchar()

declare i32 @print(ptr)

define i32 @main() {
main_fn_entry:
  %string_buffer = alloca [16 x i8], align 1
  store [16 x i8] c"Hello world! :3\00", ptr %string_buffer, align 1
  %function_call = call i32 @print(ptr %string_buffer)
  %function_call2 = call i32 @getchar()
  ret i32 0
}
