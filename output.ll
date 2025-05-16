; ModuleID = 'main'
source_filename = "main"

declare i32 @putchar(i32)

declare i32 @printchar(i32)

declare i32 @getchar()

declare i32 @return_1()

define i32 @main() {
main_fn_entry:
  %a = alloca i32, align 4
  store i32 60, ptr %a, align 4
  %char = alloca i32, align 4
  %var_deref = load i32, ptr %a, align 4
  store i32 %var_deref, ptr %char, align 4
  %char1 = load i32, ptr %char, align 4
  %function_call = call i32 @putchar(i32 %char1)
  %get_char_res = alloca i32, align 4
  %function_call2 = call i32 @getchar()
  store i32 %function_call2, ptr %get_char_res, align 4
  %ret_tmp_var = alloca i32, align 4
  %var_deref3 = load i32, ptr %get_char_res, align 4
  store i32 %var_deref3, ptr %ret_tmp_var, align 4
  %ret_tmp_var4 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var4
}
