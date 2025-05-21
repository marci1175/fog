; ModuleID = 'main'
source_filename = "main"

declare i32 @print(ptr)

define i32 @main() {
main_fn_entry:
  %test = alloca ptr, align 8
  %string_buffer = alloca [10 x i8], align 1
  store [10 x i8] c"123456789\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %test, align 8
  %str = alloca ptr, align 8
  %var_deref = load ptr, ptr %test, align 8
  store ptr %var_deref, ptr %str, align 8
  %str1 = load ptr, ptr %str, align 8
  %function_call = call i32 @print(ptr %str1)
  %str2 = alloca ptr, align 8
  %var_deref3 = load ptr, ptr %test, align 8
  store ptr %var_deref3, ptr %str2, align 8
  %str4 = load ptr, ptr %str2, align 8
  %function_call5 = call i32 @print(ptr %str4)
  %str6 = alloca ptr, align 8
  %var_deref7 = load ptr, ptr %test, align 8
  store ptr %var_deref7, ptr %str6, align 8
  %str8 = load ptr, ptr %str6, align 8
  %function_call9 = call i32 @print(ptr %str8)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var10 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var10
}
