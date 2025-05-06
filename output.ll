; ModuleID = 'main'
source_filename = "main"

define i32 @main() {
fn_main_entry:
  %val_1 = alloca i32, align 4
  store i32 69, ptr %val_1, align 4
  %val_11 = load i32, ptr %val_1, align 4
  ret i32 420
}
