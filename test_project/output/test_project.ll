; ModuleID = 'main'
source_filename = "main"

declare void @sleep(i32)

declare i32 @time(i32)

declare void @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  %time = alloca i32, align 4
  %num = alloca i32, align 4
  %0 = alloca i32, align 4
  br label %loop_body

loop_body:                                        ; preds = %loop_body, %main_fn_entry
  %num1 = alloca i32, align 4
  store i32 0, ptr %num1, align 4
  %num2 = load i32, ptr %num1, align 4
  %function_call = call i32 @time(i32 %num2)
  store i32 %function_call, ptr %time, align 4
  %time3 = load i32, ptr %time, align 4
  store i32 %time3, ptr %time, align 4
  br label %loop_body

loop_body_exit:                                   ; No predecessors!
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var4 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var4
}
