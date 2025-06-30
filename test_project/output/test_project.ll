; ModuleID = 'main'
source_filename = "main"

@"Enter 'x' to get some candy!\0A" = constant [29 x i8] c"Enter 'x' to get some candy!\0A"
@"Fatass\0A" = constant [7 x i8] c"Fatass\0A"
@"Why didnt you listen to me?\0A" = constant [28 x i8] c"Why didnt you listen to me?\0A"

declare void @printf(ptr, ...)

declare i32 @getchar()

define i32 @main() {
main_fn_entry:
  %ch = alloca i32, align 4
  store i32 0, ptr %ch, align 4
  %0 = alloca ptr, align 8
  %msg = alloca ptr, align 8
  %1 = alloca i32, align 4
  %2 = alloca i32, align 4
  %3 = alloca ptr, align 8
  %msg1 = alloca ptr, align 8
  %4 = alloca ptr, align 8
  %msg2 = alloca ptr, align 8
  br label %loop_body

loop_body:                                        ; preds = %cond_branch_uncond, %main_fn_entry
  store ptr @"Enter 'x' to get some candy!\0A", ptr %0, align 8
  %msg3 = load ptr, ptr %0, align 8
  call void (ptr, ...) @printf(ptr %msg3)
  %function_call = call i32 @getchar()
  store i32 %function_call, ptr %ch, align 4
  %ch4 = load i32, ptr %ch, align 4
  store i32 %ch4, ptr %ch, align 4
  %lhs_tmp = alloca i32, align 4
  %rhs_tmp = alloca i32, align 4
  %var_deref = load i32, ptr %ch, align 4
  store i32 %var_deref, ptr %lhs_tmp, align 4
  store i32 120, ptr %rhs_tmp, align 4
  %lhs_tmp_val = load i32, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i32, ptr %rhs_tmp, align 4
  %cmp = icmp eq i32 %lhs_tmp_val, %rhs_tmp_val
  %5 = alloca i1, align 1
  store i1 %cmp, ptr %5, align 1
  %condition = load i1, ptr %5, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

cond_branch_true:                                 ; preds = %loop_body
  %msg5 = alloca ptr, align 8
  store ptr @"Fatass\0A", ptr %msg5, align 8
  %msg6 = load ptr, ptr %msg5, align 8
  call void (ptr, ...) @printf(ptr %msg6)
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %loop_body
  %msg7 = alloca ptr, align 8
  store ptr @"Why didnt you listen to me?\0A", ptr %msg7, align 8
  %msg8 = load ptr, ptr %msg7, align 8
  call void (ptr, ...) @printf(ptr %msg8)
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  br label %loop_body
}
