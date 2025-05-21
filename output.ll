; ModuleID = 'main'
source_filename = "main"

declare i32 @print(ptr)

define i32 @test(ptr %str, float %fl, i32 %opt) {
main_fn_entry:
  ret i32 0
}

define i32 @main() {
main_fn_entry:
  %string_buffer = alloca [5 x i8], align 1
  store [5 x i8] c"Fasz\00", ptr %string_buffer, align 1
  %function_call = call i32 @test(ptr %string_buffer, float 0x4037666660000000, i32 2)
  ret i32 0
}
