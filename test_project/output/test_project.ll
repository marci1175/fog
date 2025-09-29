; ModuleID = 'main'
source_filename = "main"

@"Hello world!\0A" = constant [14 x i8] c"Hello world!\0A\00"
@"Number: %i" = constant [11 x i8] c"Number: %i\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  %function_call = call i32 (ptr, ...) @printf(ptr @"Hello world!\0A")
  %szamok = alloca [3 x i32], align 4
  %array_idx_val = getelementptr [3 x i32], ptr %szamok, i32 0, i32 0
  store i32 0, ptr %array_idx_val, align 4
  %array_idx_val6 = getelementptr [3 x i32], ptr %szamok, i32 0, i32 1
  store i32 0, ptr %array_idx_val6, align 4
  %array_idx_val7 = getelementptr [3 x i32], ptr %szamok, i32 0, i32 2
  store i32 0, ptr %array_idx_val7, align 4
  %array_idx_elem_ptr = getelementptr [3 x i32], ptr %szamok, i32 0, i32 2
  store i32 200, ptr %array_idx_elem_ptr, align 4
  %array_idx_elem_ptr12 = getelementptr [3 x i32], ptr %szamok, i32 0, i32 2
  %idx_array_val_deref = load i32, ptr %array_idx_elem_ptr12, align 4
  %function_call14 = call i32 (ptr, ...) @printf(ptr @"Number: %i", i32 %idx_array_val_deref)
  ret i32 0
}
