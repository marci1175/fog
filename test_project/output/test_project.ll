; ModuleID = 'main'
source_filename = "main"

@Yes = constant [4 x i8] c"Yes\00"
@No = constant [3 x i8] c"No\00"

declare void @printf(ptr)

define i32 @main() {
main_fn_entry:
  %a = alloca i64, align 8
  store i64 9, ptr %a, align 4
  %lhs_tmp = alloca i64, align 8
  %rhs_tmp = alloca i64, align 8
  store i64 100, ptr %rhs_tmp, align 4
  %0 = alloca i64, align 8
  store i64 1, ptr %0, align 4
  %lhs = load i64, ptr %0, align 4
  %rhs = load i64, ptr %a, align 4
  %int_add_int = add i64 %lhs, %rhs
  %math_expr_res = alloca i64, align 8
  store i64 %int_add_int, ptr %math_expr_res, align 4
  %1 = alloca i64, align 8
  store i64 2, ptr %1, align 4
  %lhs1 = load i64, ptr %math_expr_res, align 4
  %rhs2 = load i64, ptr %1, align 4
  %int_add_int3 = add i64 %lhs1, %rhs2
  store i64 %int_add_int3, ptr %lhs_tmp, align 4
  %lhs_tmp_val = load i64, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i64, ptr %rhs_tmp, align 4
  %cmp = icmp sgt i64 %lhs_tmp_val, %rhs_tmp_val
  %cmp_result = alloca i1, align 1
  store i1 %cmp, ptr %cmp_result, align 1
  %condition = load i1, ptr %cmp_result, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

cond_branch_true:                                 ; preds = %main_fn_entry
  %msg = alloca ptr, align 8
  store ptr @Yes, ptr %msg, align 8
  %msg4 = load ptr, ptr %msg, align 8
  call void @printf(ptr %msg4)
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %main_fn_entry
  %msg5 = alloca ptr, align 8
  store ptr @No, ptr %msg5, align 8
  %msg6 = load ptr, ptr %msg5, align 8
  call void @printf(ptr %msg6)
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var7 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var7
}
