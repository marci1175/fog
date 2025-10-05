; ModuleID = 'main'
source_filename = "main"

@"Hello world!\0A" = constant [14 x i8] c"Hello world!\0A\00"
@"Number %i" = constant [10 x i8] c"Number %i\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  %input = alloca ptr, align 8
  store ptr @"Hello world!\0A", ptr %input, align 8
  %input1 = load ptr, ptr %input, align 8
  %function_call = call i32 (ptr, ...) @printf(ptr %input1)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  %szamok = alloca [4 x [2 x i32]], align 4
  %array_temp_val_var = alloca [2 x i32], align 4
  %array_temp_val_var2 = alloca i32, align 4
  store i32 0, ptr %array_temp_val_var2, align 4
  %array_temp_val_deref = load i32, ptr %array_temp_val_var2, align 4
  %array_temp_val_var3 = alloca i32, align 4
  store i32 1, ptr %array_temp_val_var3, align 4
  %array_temp_val_deref4 = load i32, ptr %array_temp_val_var3, align 4
  %array_idx_val = getelementptr [2 x i32], ptr %array_temp_val_var, i32 0, i32 0
  store i32 %array_temp_val_deref, ptr %array_idx_val, align 4
  %array_idx_val5 = getelementptr [2 x i32], ptr %array_temp_val_var, i32 0, i32 1
  store i32 %array_temp_val_deref4, ptr %array_idx_val5, align 4
  %array_temp_val_deref6 = load [2 x i32], ptr %array_temp_val_var, align 4
  %array_temp_val_var7 = alloca [2 x i32], align 4
  %array_temp_val_var8 = alloca i32, align 4
  store i32 0, ptr %array_temp_val_var8, align 4
  %array_temp_val_deref9 = load i32, ptr %array_temp_val_var8, align 4
  %array_temp_val_var10 = alloca i32, align 4
  store i32 1, ptr %array_temp_val_var10, align 4
  %array_temp_val_deref11 = load i32, ptr %array_temp_val_var10, align 4
  %array_idx_val12 = getelementptr [2 x i32], ptr %array_temp_val_var7, i32 0, i32 0
  store i32 %array_temp_val_deref9, ptr %array_idx_val12, align 4
  %array_idx_val13 = getelementptr [2 x i32], ptr %array_temp_val_var7, i32 0, i32 1
  store i32 %array_temp_val_deref11, ptr %array_idx_val13, align 4
  %array_temp_val_deref14 = load [2 x i32], ptr %array_temp_val_var7, align 4
  %array_temp_val_var15 = alloca [2 x i32], align 4
  %array_temp_val_var16 = alloca i32, align 4
  store i32 0, ptr %array_temp_val_var16, align 4
  %array_temp_val_deref17 = load i32, ptr %array_temp_val_var16, align 4
  %array_temp_val_var18 = alloca i32, align 4
  store i32 1, ptr %array_temp_val_var18, align 4
  %array_temp_val_deref19 = load i32, ptr %array_temp_val_var18, align 4
  %array_idx_val20 = getelementptr [2 x i32], ptr %array_temp_val_var15, i32 0, i32 0
  store i32 %array_temp_val_deref17, ptr %array_idx_val20, align 4
  %array_idx_val21 = getelementptr [2 x i32], ptr %array_temp_val_var15, i32 0, i32 1
  store i32 %array_temp_val_deref19, ptr %array_idx_val21, align 4
  %array_temp_val_deref22 = load [2 x i32], ptr %array_temp_val_var15, align 4
  %array_temp_val_var23 = alloca [2 x i32], align 4
  %array_temp_val_var24 = alloca i32, align 4
  store i32 0, ptr %array_temp_val_var24, align 4
  %array_temp_val_deref25 = load i32, ptr %array_temp_val_var24, align 4
  %array_temp_val_var26 = alloca i32, align 4
  store i32 1, ptr %array_temp_val_var26, align 4
  %array_temp_val_deref27 = load i32, ptr %array_temp_val_var26, align 4
  %array_idx_val28 = getelementptr [2 x i32], ptr %array_temp_val_var23, i32 0, i32 0
  store i32 %array_temp_val_deref25, ptr %array_idx_val28, align 4
  %array_idx_val29 = getelementptr [2 x i32], ptr %array_temp_val_var23, i32 0, i32 1
  store i32 %array_temp_val_deref27, ptr %array_idx_val29, align 4
  %array_temp_val_deref30 = load [2 x i32], ptr %array_temp_val_var23, align 4
  %array_idx_val31 = getelementptr [4 x [2 x i32]], ptr %szamok, i32 0, i32 0
  store [2 x i32] %array_temp_val_deref6, ptr %array_idx_val31, align 4
  %array_idx_val32 = getelementptr [4 x [2 x i32]], ptr %szamok, i32 0, i32 1
  store [2 x i32] %array_temp_val_deref14, ptr %array_idx_val32, align 4
  %array_idx_val33 = getelementptr [4 x [2 x i32]], ptr %szamok, i32 0, i32 2
  store [2 x i32] %array_temp_val_deref22, ptr %array_idx_val33, align 4
  %array_idx_val34 = getelementptr [4 x [2 x i32]], ptr %szamok, i32 0, i32 3
  store [2 x i32] %array_temp_val_deref30, ptr %array_idx_val34, align 4
  %1 = alloca i32, align 4
  store i32 0, ptr %1, align 4
  %array_idx_val35 = load i32, ptr %1, align 4
  %array_idx_elem_ptr = getelementptr [4 x [2 x i32]], ptr %szamok, i32 0, i32 %array_idx_val35
  %2 = alloca i32, align 4
  store i32 1, ptr %2, align 4
  %array_idx_val36 = load i32, ptr %2, align 4
  %array_idx_elem_ptr37 = getelementptr [2 x i32], ptr %array_idx_elem_ptr, i32 0, i32 %array_idx_val36
  store i32 2, ptr %array_idx_elem_ptr37, align 4
  %input38 = alloca ptr, align 8
  store ptr @"Number %i", ptr %input38, align 8
  %input39 = load ptr, ptr %input38, align 8
  %printf_idx_1_arg = alloca i32, align 4
  %3 = alloca i32, align 4
  store i32 0, ptr %3, align 4
  %array_idx_val40 = load i32, ptr %3, align 4
  %array_idx_elem_ptr41 = getelementptr [4 x [2 x i32]], ptr %szamok, i32 0, i32 %array_idx_val40
  %4 = alloca i32, align 4
  store i32 1, ptr %4, align 4
  %array_idx_val42 = load i32, ptr %4, align 4
  %array_idx_elem_ptr43 = getelementptr [2 x i32], ptr %array_idx_elem_ptr41, i32 0, i32 %array_idx_val42
  %idx_array_val_deref = load i32, ptr %array_idx_elem_ptr43, align 4
  store i32 %idx_array_val_deref, ptr %printf_idx_1_arg, align 4
  %printf_idx_1_arg44 = load i32, ptr %printf_idx_1_arg, align 4
  %function_call45 = call i32 (ptr, ...) @printf(ptr %input39, i32 %printf_idx_1_arg44)
  %5 = alloca i32, align 4
  store i32 %function_call45, ptr %5, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var46 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var46
}
