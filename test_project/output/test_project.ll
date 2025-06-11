; ModuleID = 'main'
source_filename = "main"

%kg = type { float }

declare i32 @printf(ptr)

define i32 @main() {
main_fn_entry:
  %suly = alloca %kg, align 8
  %strct_init = alloca %kg, align 8
  %field_gep = getelementptr inbounds %kg, ptr %strct_init, i32 0, i32 0
  store float 0x4059C7AE20000000, ptr %field_gep, align 4
  %constructed_struct = load %kg, ptr %strct_init, align 4
  store %kg %constructed_struct, ptr %suly, align 4
  %deref_strct_val = load float, ptr %suly, align 4
  %cmp = fcmp ogt float %deref_strct_val, 3.000000e+01
  br i1 %cmp, label %cond_branch_true, label %cond_branch_false

cond_branch_true:                                 ; preds = %main_fn_entry
  %str = alloca ptr, align 8
  %string_buffer = alloca [6 x i8], align 1
  store [6 x i8] c"Hello\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %str, align 8
  %str2 = load ptr, ptr %str, align 8
  %function_call = call i32 @printf(ptr %str2)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %main_fn_entry
  %str3 = alloca ptr, align 8
  %string_buffer4 = alloca [10 x i8], align 1
  store [10 x i8] c"Not Hello\00", ptr %string_buffer4, align 1
  store ptr %string_buffer4, ptr %str3, align 8
  %str5 = load ptr, ptr %str3, align 8
  %function_call6 = call i32 @printf(ptr %str5)
  %1 = alloca i32, align 4
  store i32 %function_call6, ptr %1, align 4
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var7 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var7
}
