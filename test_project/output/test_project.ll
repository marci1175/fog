; ModuleID = 'main'
source_filename = "main"

@"The number is %i" = constant [17 x i8] c"The number is %i\00"

declare void @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  %marci = alloca [5 x i32], align 4
  %array_idx_val = getelementptr [5 x i32], ptr %marci, i32 0, i32 0
  store i32 2, ptr %array_idx_val, align 4
  %array_idx_val9 = getelementptr [5 x i32], ptr %marci, i32 0, i32 1
  store i32 2, ptr %array_idx_val9, align 4
  %array_idx_val10 = getelementptr [5 x i32], ptr %marci, i32 0, i32 2
  store i32 2, ptr %array_idx_val10, align 4
  %array_idx_val11 = getelementptr [5 x i32], ptr %marci, i32 0, i32 3
  store i32 2, ptr %array_idx_val11, align 4
  %array_idx_val12 = getelementptr [5 x i32], ptr %marci, i32 0, i32 4
  store i32 2, ptr %array_idx_val12, align 4
  %a = alloca i32, align 4
  %var_deref = load [5 x i32], ptr %marci, align 4
  %array_idx_elem = getelementptr [5 x i32], ptr %marci, i32 0, i64 0
  %idx_array_val_deref = load [5 x i32], ptr %array_idx_elem, align 4
  store [5 x i32] %idx_array_val_deref, ptr %a, align 4
  %var_deref15 = load i32, ptr %a, align 4
  call void (ptr, ...) @printf(ptr @"The number is %i", i32 %var_deref15)
  ret i32 0
}
