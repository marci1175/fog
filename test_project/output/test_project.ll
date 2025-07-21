; ModuleID = 'main'
source_filename = "main"

@"Time: %i" = constant [9 x i8] c"Time: %i\00"

declare void @sleep(i32)

declare void @printf(ptr, ...)

declare i32 @time(i32)

define i32 @main() {
main_fn_entry:
  %time = alloca i32, align 4
  %num = alloca i32, align 4
  %0 = alloca i32, align 4
  %msg = alloca ptr, align 8
  %1 = alloca i32, align 4
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %main_fn_entry
  %time1 = alloca i32, align 4
  store i32 0, ptr %time, align 4
  %num2 = load i32, ptr %time, align 4
  %function_call = call i32 @time(i32 %num2)
  store i32 %function_call, ptr %time1, align 4
  %time3 = load i32, ptr %time1, align 4
  store i32 %time3, ptr %time1, align 4
  store ptr @"Time: %i", ptr %num, align 8
  %msg4 = load i32, ptr %num, align 4
  %var_deref = load i32, ptr %time1, align 4
  store i32 %var_deref, ptr %0, align 4
  %2 = load i32, ptr %0, align 4
  call void (ptr, ...) @printf(i32 %msg4, i32 %2)
  br label %loop_body

loop_body_exit:                                   ; No predecessors!
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var5 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var5
}
