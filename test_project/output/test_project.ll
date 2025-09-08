; ModuleID = 'main'
source_filename = "main"

@"The number is %i" = constant [17 x i8] c"The number is %i\00"

declare void @printf(ptr, ...)

define i32 @return_0() {
main_fn_entry:
  %ret_tmp_var = alloca i32, align 4
  store i32 2, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var1
}

define i32 @main() {
main_fn_entry:
  %marci = alloca [5 x i32], align 4
  %array_temp_val_var = alloca i32, align 4
  store i32 90, ptr %array_temp_val_var, align 4
  %array_temp_val_deref = load i32, ptr %array_temp_val_var, align 4
  %array_temp_val_var1 = alloca i32, align 4
  store i32 4, ptr %array_temp_val_var1, align 4
  %array_temp_val_deref2 = load i32, ptr %array_temp_val_var1, align 4
  %array_temp_val_var3 = alloca i32, align 4
  store i32 5, ptr %array_temp_val_var3, align 4
  %array_temp_val_deref4 = load i32, ptr %array_temp_val_var3, align 4
  %array_temp_val_var5 = alloca i32, align 4
  store i32 6, ptr %array_temp_val_var5, align 4
  %array_temp_val_deref6 = load i32, ptr %array_temp_val_var5, align 4
  %array_temp_val_var7 = alloca i32, align 4
  store i32 7, ptr %array_temp_val_var7, align 4
  %array_temp_val_deref8 = load i32, ptr %array_temp_val_var7, align 4
  %array_idx_val = getelementptr [5 x i32], ptr %marci, i32 0, i32 0
  store i32 %array_temp_val_deref, ptr %array_idx_val, align 4
  %array_idx_val9 = getelementptr [5 x i32], ptr %marci, i32 0, i32 1
  store i32 %array_temp_val_deref2, ptr %array_idx_val9, align 4
  %array_idx_val10 = getelementptr [5 x i32], ptr %marci, i32 0, i32 2
  store i32 %array_temp_val_deref4, ptr %array_idx_val10, align 4
  %array_idx_val11 = getelementptr [5 x i32], ptr %marci, i32 0, i32 3
  store i32 %array_temp_val_deref6, ptr %array_idx_val11, align 4
  %array_idx_val12 = getelementptr [5 x i32], ptr %marci, i32 0, i32 4
  store i32 %array_temp_val_deref8, ptr %array_idx_val12, align 4
  %function_call = call i32 @return_0()
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  %array_idx_val13 = load i32, ptr %0, align 4
  %array_idx_elem = getelementptr [5 x i32], ptr %marci, i32 0, i32 %array_idx_val13
  %idx_array_val_deref = load i32, ptr %array_idx_elem, align 4
  %temp_deref_var = alloca i32, align 4
  store i32 %idx_array_val_deref, ptr %temp_deref_var, align 4
  %input = alloca ptr, align 8
  store ptr @"The number is %i", ptr %input, align 8
  %input14 = load ptr, ptr %input, align 8
  %1 = alloca i32, align 4
  %function_call15 = call i32 @return_0()
  %2 = alloca i32, align 4
  store i32 %function_call15, ptr %2, align 4
  %array_idx_val16 = load i32, ptr %2, align 4
  %array_idx_elem17 = getelementptr [5 x i32], ptr %marci, i32 0, i32 %array_idx_val16
  %idx_array_val_deref18 = load i32, ptr %array_idx_elem17, align 4
  %temp_deref_var19 = alloca i32, align 4
  store i32 %idx_array_val_deref18, ptr %temp_deref_var19, align 4
  %3 = load i32, ptr %temp_deref_var19, align 4
  store i32 %3, ptr %1, align 4
  %4 = load i32, ptr %1, align 4
  call void (ptr, ...) @printf(ptr %input14, i32 %4)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var20 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var20
}
