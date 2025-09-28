; ModuleID = 'main'
source_filename = "main"

@"Hello world!\0A" = constant [14 x i8] c"Hello world!\0A\00"
@"Number: %i" = constant [11 x i8] c"Number: %i\00"

declare i32 @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  %input = alloca ptr, align 8
  store ptr @"Hello world!\0A", ptr %input, align 8
  %input1 = load ptr, ptr %input, align 8
  %function_call = call i32 (ptr, ...) @printf(ptr %input1)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  %szamok = alloca [3 x i32], align 4
  %array_temp_val_var = alloca i32, align 4
  store i32 0, ptr %array_temp_val_var, align 4
  %array_temp_val_deref = load i32, ptr %array_temp_val_var, align 4
  %array_temp_val_var2 = alloca i32, align 4
  store i32 0, ptr %array_temp_val_var2, align 4
  %array_temp_val_deref3 = load i32, ptr %array_temp_val_var2, align 4
  %array_temp_val_var4 = alloca i32, align 4
  store i32 0, ptr %array_temp_val_var4, align 4
  %array_temp_val_deref5 = load i32, ptr %array_temp_val_var4, align 4
  %array_idx_val = getelementptr [3 x i32], ptr %szamok, i32 0, i32 0
  store i32 %array_temp_val_deref, ptr %array_idx_val, align 4
  %array_idx_val6 = getelementptr [3 x i32], ptr %szamok, i32 0, i32 1
  store i32 %array_temp_val_deref3, ptr %array_idx_val6, align 4
  %array_idx_val7 = getelementptr [3 x i32], ptr %szamok, i32 0, i32 2
  store i32 %array_temp_val_deref5, ptr %array_idx_val7, align 4
  %1 = alloca i32, align 4
  store i32 2, ptr %1, align 4
  %array_idx_val8 = load i32, ptr %1, align 4
  %array_idx_elem = getelementptr [3 x i32], ptr %szamok, i32 0, i32 %array_idx_val8
  store i32 200, ptr %array_idx_elem, align 4
  %input9 = alloca ptr, align 8
  store ptr @"Number: %i", ptr %input9, align 8
  %input10 = load ptr, ptr %input9, align 8
  %printf_idx_1_arg = alloca i32, align 4
  %2 = alloca i32, align 4
  store i32 2, ptr %2, align 4
  %array_idx_val11 = load i32, ptr %2, align 4
  %array_idx_elem12 = getelementptr [3 x i32], ptr %szamok, i32 0, i32 %array_idx_val11
  %idx_array_val_deref = load i32, ptr %array_idx_elem12, align 4
  store i32 %idx_array_val_deref, ptr %printf_idx_1_arg, align 4
  %printf_idx_1_arg13 = load i32, ptr %printf_idx_1_arg, align 4
  %function_call14 = call i32 (ptr, ...) @printf(ptr %input10, i32 %printf_idx_1_arg13)
  %3 = alloca i32, align 4
  store i32 %function_call14, ptr %3, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var15 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var15
}
