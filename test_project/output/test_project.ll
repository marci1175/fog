; ModuleID = 'main'
source_filename = "main"

@"Num is: %i\0A" = constant [12 x i8] c"Num is: %i\0A\00"
@"Baj van tesomsz!" = constant [17 x i8] c"Baj van tesomsz!\00"

declare i32 @time(i32)

declare void @printf(ptr, ...)

declare void @sleep(i32)

define void @return_2(i32 %a) {
main_fn_entry:
  %a1 = alloca i32, align 4
  store i32 %a, ptr %a1, align 4
  %msg = alloca ptr, align 8
  store ptr @"Num is: %i\0A", ptr %msg, align 8
  %msg2 = load ptr, ptr %msg, align 8
  %0 = alloca i32, align 4
  %var_deref = load i32, ptr %a1, align 4
  store i32 %var_deref, ptr %0, align 4
  %1 = load i32, ptr %0, align 4
  call void (ptr, ...) @printf(ptr %msg2, i32 %1)
  ret void
}

define i32 @main() {
main_fn_entry:
  %lhs_tmp = alloca i64, align 8
  %rhs_tmp = alloca i64, align 8
  %lhs_tmp_val = load i64, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i64, ptr %rhs_tmp, align 4
  %cmp = icmp sgt i64 %lhs_tmp_val, %rhs_tmp_val
  %cmp_result = alloca i1, align 1
  store i1 %cmp, ptr %cmp_result, align 1
  %condition = load i1, ptr %cmp_result, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

cond_branch_true:                                 ; preds = %main_fn_entry
  %msg = alloca ptr, align 8
  store ptr @"Baj van tesomsz!", ptr %msg, align 8
  %msg1 = load ptr, ptr %msg, align 8
  call void (ptr, ...) @printf(ptr %msg1)
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %main_fn_entry
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var2
}
