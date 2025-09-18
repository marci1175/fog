; ModuleID = 'main'
source_filename = "main"

@"%s" = constant [3 x i8] c"%s\00"
@"Hello %i" = constant [9 x i8] c"Hello %i\00"

declare i32 @scanf(ptr, [10 x i8])

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
  %buf = alloca [10 x i8], align 1
  %array_temp_val_var = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var, align 1
  %array_temp_val_deref = load i8, ptr %array_temp_val_var, align 1
  %array_temp_val_var1 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var1, align 1
  %array_temp_val_deref2 = load i8, ptr %array_temp_val_var1, align 1
  %array_temp_val_var3 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var3, align 1
  %array_temp_val_deref4 = load i8, ptr %array_temp_val_var3, align 1
  %array_temp_val_var5 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var5, align 1
  %array_temp_val_deref6 = load i8, ptr %array_temp_val_var5, align 1
  %array_temp_val_var7 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var7, align 1
  %array_temp_val_deref8 = load i8, ptr %array_temp_val_var7, align 1
  %array_temp_val_var9 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var9, align 1
  %array_temp_val_deref10 = load i8, ptr %array_temp_val_var9, align 1
  %array_temp_val_var11 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var11, align 1
  %array_temp_val_deref12 = load i8, ptr %array_temp_val_var11, align 1
  %array_temp_val_var13 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var13, align 1
  %array_temp_val_deref14 = load i8, ptr %array_temp_val_var13, align 1
  %array_temp_val_var15 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var15, align 1
  %array_temp_val_deref16 = load i8, ptr %array_temp_val_var15, align 1
  %array_temp_val_var17 = alloca i8, align 1
  store i8 0, ptr %array_temp_val_var17, align 1
  %array_temp_val_deref18 = load i8, ptr %array_temp_val_var17, align 1
  %array_idx_val = getelementptr [10 x i8], ptr %buf, i32 0, i32 0
  store i8 %array_temp_val_deref, ptr %array_idx_val, align 1
  %array_idx_val19 = getelementptr [10 x i8], ptr %buf, i32 0, i32 1
  store i8 %array_temp_val_deref2, ptr %array_idx_val19, align 1
  %array_idx_val20 = getelementptr [10 x i8], ptr %buf, i32 0, i32 2
  store i8 %array_temp_val_deref4, ptr %array_idx_val20, align 1
  %array_idx_val21 = getelementptr [10 x i8], ptr %buf, i32 0, i32 3
  store i8 %array_temp_val_deref6, ptr %array_idx_val21, align 1
  %array_idx_val22 = getelementptr [10 x i8], ptr %buf, i32 0, i32 4
  store i8 %array_temp_val_deref8, ptr %array_idx_val22, align 1
  %array_idx_val23 = getelementptr [10 x i8], ptr %buf, i32 0, i32 5
  store i8 %array_temp_val_deref10, ptr %array_idx_val23, align 1
  %array_idx_val24 = getelementptr [10 x i8], ptr %buf, i32 0, i32 6
  store i8 %array_temp_val_deref12, ptr %array_idx_val24, align 1
  %array_idx_val25 = getelementptr [10 x i8], ptr %buf, i32 0, i32 7
  store i8 %array_temp_val_deref14, ptr %array_idx_val25, align 1
  %array_idx_val26 = getelementptr [10 x i8], ptr %buf, i32 0, i32 8
  store i8 %array_temp_val_deref16, ptr %array_idx_val26, align 1
  %array_idx_val27 = getelementptr [10 x i8], ptr %buf, i32 0, i32 9
  store i8 %array_temp_val_deref18, ptr %array_idx_val27, align 1
  %filter = alloca ptr, align 8
  store ptr @"%s", ptr %filter, align 8
  %filter28 = load ptr, ptr %filter, align 8
  %buffer = alloca [10 x i8], align 1
  %var_deref = load [10 x i8], ptr %buf, align 1
  store [10 x i8] %var_deref, ptr %buffer, align 1
  %buffer29 = load [10 x i8], ptr %buffer, align 1
  %function_call = call i32 @scanf(ptr %filter28, [10 x i8] %buffer29)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  %input = alloca ptr, align 8
  store ptr @"Hello %i", ptr %input, align 8
  %input30 = load ptr, ptr %input, align 8
  %printf_idx_1_arg = alloca i8, align 1
  %1 = alloca i32, align 4
  store i32 2, ptr %1, align 4
  %array_idx_val31 = load i32, ptr %1, align 4
  %array_idx_elem = getelementptr [10 x i8], ptr %buf, i32 0, i32 %array_idx_val31
  %idx_array_val_deref = load i8, ptr %array_idx_elem, align 1
  store i8 %idx_array_val_deref, ptr %printf_idx_1_arg, align 1
  %printf_idx_1_arg32 = load i8, ptr %printf_idx_1_arg, align 1
  call void (ptr, ...) @printf(ptr %input30, i8 %printf_idx_1_arg32)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var33 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var33
}
