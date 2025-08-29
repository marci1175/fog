; ModuleID = 'main'
source_filename = "main"

define i32 @main() {
main_fn_entry:
  %marci = alloca [4 x i32], align 4
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
  store [4 x i32] [i32 %array_temp_val_deref, i32 %array_temp_val_deref2, i32 %array_temp_val_deref4, i32 %array_temp_val_deref6], ptr %marci, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var7 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var7
}
