; ModuleID = 'main'
source_filename = "main"

@Yes = constant [4 x i8] c"Yes\00"
@No = constant [3 x i8] c"No\00"

declare void @printf(ptr)

define i32 @main() {
main_fn_entry:
  %a = alloca i32, align 4
  store i32 9, ptr %a, align 4
  %lhs_tmp = alloca i32, align 4
  %rhs_tmp = alloca i32, align 4
  %0 = alloca i32, align 4
  store i32 100, ptr %0, align 4
  %1 = load i32, ptr %0, align 4
  store i32 %1, ptr %rhs_tmp, align 4
  %2 = alloca i64, align 8
  store i64 1, ptr %2, align 4
  %3 = load i64, ptr %2, align 4
  %i64_to_i32 = trunc i64 %3 to i32
  store i32 %i64_to_i32, ptr %lhs_tmp, align 4
  %lhs_tmp_val = load i32, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i32, ptr %rhs_tmp, align 4
  %cmp = icmp sgt i32 %lhs_tmp_val, %rhs_tmp_val
  %cmp_result = alloca i1, align 1
  store i1 %cmp, ptr %cmp_result, align 1
  %condition = load i1, ptr %cmp_result, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

cond_branch_true:                                 ; preds = %main_fn_entry
  %msg = alloca ptr, align 8
  store ptr @Yes, ptr %msg, align 8
  %msg1 = load ptr, ptr %msg, align 8
  call void @printf(ptr %msg1)
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %main_fn_entry
  %msg2 = alloca ptr, align 8
  store ptr @No, ptr %msg2, align 8
  %msg3 = load ptr, ptr %msg2, align 8
  call void @printf(ptr %msg3)
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var4 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var4
}
