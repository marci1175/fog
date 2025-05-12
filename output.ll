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
  %dereferenced_variable_reference2 = load i32, ptr %charin, align 4
  %function_call3 = call i32 @putchar(i32 %dereferenced_variable_reference2)
  %charout4 = alloca i32, align 4
  store i32 %function_call3, ptr %charout4, align 4
  %dereferenced_variable_reference5 = load i32, ptr %charin, align 4
  %function_call6 = call i32 @putchar(i32 %dereferenced_variable_reference5)
  %charout7 = alloca i32, align 4
  store i32 %function_call6, ptr %charout7, align 4
  %0 = call i32 @getchar()
  %variable_ref = load i32, ptr %charin, align 4
  ret i32 %variable_ref
}
