; ModuleID = 'main'
source_filename = "main"

declare i32 @putchar(i32)

declare i32 @print(i32)

declare i32 @getchar()

declare i32 @return_1()

define i32 @main() {
fn_main_entry:
  %0 = call i32 @print()
  ret i32 0
}
