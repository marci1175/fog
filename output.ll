; ModuleID = 'main'
source_filename = "main"

declare i32 @putchar(i32)

declare i32 @printchar(i32)

declare i32 @getchar()

declare i32 @return_1()

define i32 @main() {
fn_main_entry:
  %function_call = call i32 @getchar()
  %charin = alloca i32, align 4
  store i32 %function_call, ptr %charin, align 4
  %dereferenced_variable_reference = load i32, ptr %charin, align 4
  %function_call1 = call i32 @putchar(i32 %dereferenced_variable_reference)
  %charout = alloca i32, align 4
  store i32 %function_call1, ptr %charout, align 4
  %variable_ref = load i32, ptr %charin, align 4
  ret i32 %variable_ref
}
