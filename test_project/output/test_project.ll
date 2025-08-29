; ModuleID = 'main'
source_filename = "main"

@Yes = constant [4 x i8] c"Yes\00"

declare void @printf(ptr)

define i32 @main() {
main_fn_entry:
  %msg = alloca ptr, align 8
  store ptr @Yes, ptr %msg, align 8
  %msg1 = load ptr, ptr %msg, align 8
  call void @printf(ptr %msg1)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var2
}
