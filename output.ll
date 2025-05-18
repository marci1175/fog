; ModuleID = 'main'
source_filename = "main"

declare i32 @putchar(i32)

declare i32 @getchar()

define i32 @main() {
main_fn_entry:
  %function_call = call i32 @putchar(i32 23)
  %function_call2 = call i32 @getchar()
  ret i32 %function_call2
}
