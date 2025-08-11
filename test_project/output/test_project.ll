; ModuleID = 'main'
source_filename = "main"

declare void @return_2(i32)

declare void @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  call void @return_2()
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var1
}
