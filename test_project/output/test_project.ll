; ModuleID = 'main'
source_filename = "main"

@"Num is: %i\0A" = constant [12 x i8] c"Num is: %i\0A\00"
@"math is still intact!\0A" = constant [23 x i8] c"math is still intact!\0A\00"
@"math broke!\0A" = constant [13 x i8] c"math broke!\0A\00"

declare void @printf(ptr, ...)

define void @return_2(i32 %a) {
main_fn_entry:
  call void (ptr, ...) @printf(ptr @"Num is: %i\0A", i32 %a)
  ret void
}

define i32 @main() {
main_fn_entry:
  br label %loop_body

loop_body:                                        ; preds = %cond_branch_uncond, %main_fn_entry
  %int_add_int = add i32 1, 2
  %int_sub_int = sub i32 6, 9
  %cmp = icmp sgt i32 %int_add_int, %int_sub_int
  br i1 %cmp, label %cond_branch_true, label %cond_branch_false

loop_body_exit:                                   ; No predecessors!
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var7 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var7

cond_branch_true:                                 ; preds = %loop_body
  call void (ptr, ...) @printf(ptr @"math is still intact!\0A")
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %loop_body
  call void (ptr, ...) @printf(ptr @"math broke!\0A")
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  br label %loop_body
}
