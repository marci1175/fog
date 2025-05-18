; ModuleID = 'main'
source_filename = "main"

declare i32 @getchar()

declare void @greet()

define i32 @main() {
main_fn_entry:
  call void @greet()
  %function_call = call i32 @getchar()
  ret i32 %function_call
}
