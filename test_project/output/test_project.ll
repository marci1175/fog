; ModuleID = 'main'
source_filename = "main"

@"The number is %i" = constant [17 x i8] c"The number is %i\00"

declare void @printf(ptr, ...)

define i32 @return_0() {
main_fn_entry:
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var1
}

define i32 @main() {
main_fn_entry:
  %marci = alloca [5 x i32], align 4
  %array_temp_val_var = alloca i32, align 4
  store i32 2, ptr %array_temp_val_var, align 4
  %array_temp_val_deref = load i32, ptr %array_temp_val_var, align 4
  %array_temp_val_var1 = alloca i32, align 4
  store i32 2, ptr %array_temp_val_var1, align 4
  %array_temp_val_deref2 = load i32, ptr %array_temp_val_var1, align 4
  %array_temp_val_var3 = alloca i32, align 4
  store i32 2, ptr %array_temp_val_var3, align 4
  %array_temp_val_deref4 = load i32, ptr %array_temp_val_var3, align 4
  %array_temp_val_var5 = alloca i32, align 4
  store i32 2, ptr %array_temp_val_var5, align 4
  %array_temp_val_deref6 = load i32, ptr %array_temp_val_var5, align 4
  %array_temp_val_var7 = alloca i32, align 4
  store i32 2, ptr %array_temp_val_var7, align 4
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
  %a = alloca i32, align 4
  %function_call = call i32 @return_0()
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  %ty_cast_temp_val = alloca i32, align 4
  %1 = load i32, ptr %0, align 4
  %i64_to_u64 = sext i32 %1 to i64
  store i64 %i64_to_u64, ptr %ty_cast_temp_val, align 4
  %var_deref = load [5 x i32], ptr %marci, align 4
  %array_idx_val13 = load i32, ptr %ty_cast_temp_val, align 4
  %array_idx_elem = getelementptr [5 x i32], ptr %marci, i32 0, i32 %array_idx_val13
  %idx_array_val_deref = load [5 x i32], ptr %array_idx_elem, align 4
  store [5 x i32] %idx_array_val_deref, ptr %a, align 4
  %input = alloca ptr, align 8
  store ptr @"The number is %i", ptr %input, align 8
  %input14 = load ptr, ptr %input, align 8
  %2 = alloca i32, align 4
  %var_deref15 = load i32, ptr %a, align 4
  store i32 %var_deref15, ptr %2, align 4
  %3 = load i32, ptr %2, align 4
  call void (ptr, ...) @printf(ptr %input14, i32 %3)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var16 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var16
}
