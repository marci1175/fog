; ModuleID = 'main'
source_filename = "main"

declare void @greet()

declare i32 @getchar()

define i32 @apad(i32 %x) {
main_fn_entry:
  ret i32 %x
}

define i32 @main() {
main_fn_entry:
  call void @greet()
  %function_call = call i32 @apad(i32 420)
  %function_call2 = call i32 @getchar()
  ret i32 %function_call
}
