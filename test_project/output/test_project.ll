; ModuleID = 'main'
source_filename = "main"

@"Seconds since epoch: %i" = constant [23 x i8] c"Seconds since epoch: %i"
@"Seconds left till destination time: %i" = constant [38 x i8] c"Seconds left till destination time: %i"
@"We have exited the loop body!!!\0A" = constant [32 x i8] c"We have exited the loop body!!!\0A"

declare void @sleep(i32)

declare void @printf(ptr, ...)

declare i32 @time(i32)

define i32 @main() {
main_fn_entry:
  %curr_time = alloca i32, align 4
  %num = alloca i32, align 4
  store i32 0, ptr %num, align 4
  %num1 = load i32, ptr %num, align 4
  %function_call = call i32 @time(i32 %num1)
  store i32 %function_call, ptr %curr_time, align 4
  %curr_time2 = load i32, ptr %curr_time, align 4
  store i32 %curr_time2, ptr %curr_time, align 4
  %msg = alloca ptr, align 8
  store ptr @"Seconds since epoch: %i", ptr %msg, align 8
  %msg3 = load ptr, ptr %msg, align 8
  %0 = alloca i32, align 4
  %var_deref = load i32, ptr %curr_time, align 4
  store i32 %var_deref, ptr %0, align 4
  %1 = load i32, ptr %0, align 4
  call void (ptr, ...) @printf(ptr %msg3, i32 %1)
  %destination_secs = alloca i32, align 4
  %2 = alloca i32, align 4
  store i32 10, ptr %2, align 4
  %lhs = load i32, ptr %curr_time, align 4
  %rhs = load i32, ptr %2, align 4
  %int_add_int = add i32 %lhs, %rhs
  store i32 %int_add_int, ptr %destination_secs, align 4
  %secs_left = alloca i32, align 4
  %3 = alloca ptr, align 8
  %msg4 = alloca ptr, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  %secs = alloca i32, align 4
  %6 = alloca i32, align 4
  br label %loop_body

loop_body:                                        ; preds = %cond_branch_uncond, %main_fn_entry
  %7 = load i32, ptr %secs_left, align 4
  store i32 %7, ptr %secs_left, align 4
  store ptr @"Seconds left till destination time: %i", ptr %3, align 8
  %msg5 = load ptr, ptr %3, align 8
  %var_deref6 = load i32, ptr %secs_left, align 4
  store i32 %var_deref6, ptr %secs_left, align 4
  %8 = load i32, ptr %secs_left, align 4
  call void (ptr, ...) @printf(ptr %msg5, i32 %8)
  store i32 1, ptr %5, align 4
  %secs7 = load i32, ptr %5, align 4
  call void @sleep(i32 %secs7)
  %lhs_tmp = alloca i32, align 4
  %rhs_tmp = alloca i32, align 4
  %var_deref8 = load i32, ptr %secs_left, align 4
  store i32 %var_deref8, ptr %lhs_tmp, align 4
  store i32 0, ptr %rhs_tmp, align 4
  %lhs_tmp_val = load i32, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i32, ptr %rhs_tmp, align 4
  %cmp = icmp eq i32 %lhs_tmp_val, %rhs_tmp_val
  %9 = alloca i1, align 1
  store i1 %cmp, ptr %9, align 1
  %condition = load i1, ptr %9, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

loop_body_exit:                                   ; preds = %cond_branch_true
  %msg9 = alloca ptr, align 8
  store ptr @"We have exited the loop body!!!\0A", ptr %msg9, align 8
  %msg10 = load ptr, ptr %msg9, align 8
  call void (ptr, ...) @printf(ptr %msg10)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var11 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var11

cond_branch_true:                                 ; preds = %loop_body
  br label %loop_body_exit
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %loop_body
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  br label %loop_body
}
