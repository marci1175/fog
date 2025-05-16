; ModuleID = 'main'
source_filename = "main"

declare i32 @putchar(i32)

declare i32 @printchar(i32)

declare i32 @getchar()

declare i32 @return_1()

define i32 @return_23() {
main_fn_entry:
  %ret_tmp_var = alloca i32, align 4
  store i32 23, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var1
}

define i32 @main() {
main_fn_entry:
  %a = alloca i32, align 4
  store i32 5677, ptr %a, align 4
  %b = alloca i32, align 4
  %function_call = call i32 @return_23()
  store i32 %function_call, ptr %b, align 4
  %function_call1 = call i32 @putchar()
  %ret_tmp_var = alloca i32, align 4
  %var_deref = load i32, ptr %b, align 4
  store i32 %var_deref, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var2
}
