; ModuleID = 'main'
source_filename = "main"

@"Oh no! Math broke!" = constant [19 x i8] c"Oh no! Math broke!\00"
@"Oh yes! Math is didn't break!" = constant [30 x i8] c"Oh yes! Math is didn't break!\00"

declare void @printf(ptr)

define i32 @main() {
main_fn_entry:
  %lhs_tmp = alloca i64, align 8
  %rhs_tmp = alloca i64, align 8
  store i64 8, ptr %rhs_tmp, align 4
  store i64 3, ptr %lhs_tmp, align 4
  %lhs_tmp_val = load i64, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i64, ptr %rhs_tmp, align 4
  %cmp = icmp sgt i64 %lhs_tmp_val, %rhs_tmp_val
  %cmp_result = alloca i1, align 1
  store i1 %cmp, ptr %cmp_result, align 1
  %condition = load i1, ptr %cmp_result, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

cond_branch_true:                                 ; preds = %main_fn_entry
  %msg = alloca ptr, align 8
  store ptr @"Oh no! Math broke!", ptr %msg, align 8
  %msg1 = load ptr, ptr %msg, align 8
  call void @printf(ptr %msg1)
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %main_fn_entry
  %msg2 = alloca ptr, align 8
  store ptr @"Oh yes! Math is didn't break!", ptr %msg2, align 8
  %msg3 = load ptr, ptr %msg2, align 8
  call void @printf(ptr %msg3)
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var4 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var4
}
