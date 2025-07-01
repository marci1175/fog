; ModuleID = 'main'
source_filename = "main"

@"Seconds since epoch: %i\0A" = constant [25 x i8] c"Seconds since epoch: %i\0A\00"
@"Destination time: %i\0A" = constant [22 x i8] c"Destination time: %i\0A\00"
@"Seconds left till destination time: %i\0A" = constant [40 x i8] c"Seconds left till destination time: %i\0A\00"
@"We have exited the loop body!!!\0A" = constant [33 x i8] c"We have exited the loop body!!!\0A\00"

declare void @printf(ptr, ...)

declare i32 @time(i32)

declare void @sleep(i32)

define i32 @main() {
main_fn_entry:
  %app_start_time = alloca i32, align 4
  %num = alloca i32, align 4
  store i32 0, ptr %num, align 4
  %num1 = load i32, ptr %num, align 4
  %function_call = call i32 @time(i32 %num1)
  store i32 %function_call, ptr %app_start_time, align 4
  %app_start_time2 = load i32, ptr %app_start_time, align 4
  store i32 %app_start_time2, ptr %app_start_time, align 4
  %msg = alloca ptr, align 8
  store ptr @"Seconds since epoch: %i\0A", ptr %msg, align 8
  %msg3 = load ptr, ptr %msg, align 8
  %0 = alloca i32, align 4
  %var_deref = load i32, ptr %app_start_time, align 4
  store i32 %var_deref, ptr %0, align 4
  %1 = load i32, ptr %0, align 4
  call void (ptr, ...) @printf(ptr %msg3, i32 %1)
  %destination_secs = alloca i32, align 4
  %2 = alloca i32, align 4
  store i32 10, ptr %2, align 4
  %lhs = load i32, ptr %app_start_time, align 4
  %rhs = load i32, ptr %2, align 4
  %int_add_int = add i32 %lhs, %rhs
  store i32 %int_add_int, ptr %destination_secs, align 4
  %msg4 = alloca ptr, align 8
  store ptr @"Destination time: %i\0A", ptr %msg4, align 8
  %msg5 = load ptr, ptr %msg4, align 8
  %3 = alloca i32, align 4
  %var_deref6 = load i32, ptr %destination_secs, align 4
  store i32 %var_deref6, ptr %3, align 4
  %4 = load i32, ptr %3, align 4
  call void (ptr, ...) @printf(ptr %msg5, i32 %4)
  %time = alloca i32, align 4
  %secs_left = alloca i32, align 4
  %5 = alloca ptr, align 8
  %msg7 = alloca ptr, align 8
  %6 = alloca i32, align 4
  %7 = alloca ptr, align 8
  %msg8 = alloca ptr, align 8
  %8 = alloca i32, align 4
  %9 = alloca i32, align 4
  %secs = alloca i32, align 4
  %10 = alloca i32, align 4
  br label %loop_body

loop_body:                                        ; preds = %cond_branch_uncond, %main_fn_entry
  %num9 = alloca i32, align 4
  store i32 0, ptr %num9, align 4
  %num10 = load i32, ptr %num9, align 4
  %function_call11 = call i32 @time(i32 %num10)
  store i32 %function_call11, ptr %time, align 4
  %time12 = load i32, ptr %time, align 4
  store i32 %time12, ptr %time, align 4
  %lhs13 = load i32, ptr %destination_secs, align 4
  %rhs14 = load i32, ptr %time, align 4
  %int_sub_int = sub i32 %lhs13, %rhs14
  store i32 %int_sub_int, ptr %secs_left, align 4
  store ptr @"Seconds since epoch: %i\0A", ptr %5, align 8
  %msg15 = load ptr, ptr %5, align 8
  %var_deref16 = load i32, ptr %time, align 4
  store i32 %var_deref16, ptr %time, align 4
  %11 = load i32, ptr %time, align 4
  call void (ptr, ...) @printf(ptr %msg15, i32 %11)
  store ptr @"Seconds left till destination time: %i\0A", ptr %7, align 8
  %msg17 = load ptr, ptr %7, align 8
  %var_deref18 = load i32, ptr %secs_left, align 4
  store i32 %var_deref18, ptr %secs_left, align 4
  %12 = load i32, ptr %secs_left, align 4
  call void (ptr, ...) @printf(ptr %msg17, i32 %12)
  store i32 1000, ptr %9, align 4
  %secs19 = load i32, ptr %9, align 4
  call void @sleep(i32 %secs19)
  %lhs_tmp = alloca i32, align 4
  %rhs_tmp = alloca i32, align 4
  %var_deref20 = load i32, ptr %secs_left, align 4
  store i32 %var_deref20, ptr %lhs_tmp, align 4
  store i32 0, ptr %rhs_tmp, align 4
  %lhs_tmp_val = load i32, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i32, ptr %rhs_tmp, align 4
  %cmp = icmp eq i32 %lhs_tmp_val, %rhs_tmp_val
  %13 = alloca i1, align 1
  store i1 %cmp, ptr %13, align 1
  %condition = load i1, ptr %13, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

loop_body_exit:                                   ; preds = %cond_branch_true
  %msg21 = alloca ptr, align 8
  store ptr @"We have exited the loop body!!!\0A", ptr %msg21, align 8
  %msg22 = load ptr, ptr %msg21, align 8
  call void (ptr, ...) @printf(ptr %msg22)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var23 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var23

cond_branch_true:                                 ; preds = %loop_body
  br label %loop_body_exit
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %loop_body
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  br label %loop_body
}
