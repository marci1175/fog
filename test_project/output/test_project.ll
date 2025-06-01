; ModuleID = 'main'
source_filename = "main"

declare i32 @puts(ptr)

define i32 @main() {
main_fn_entry:
  %msg = alloca ptr, align 8
  %string_buffer = alloca [13 x i8], align 1
  store [13 x i8] c"Hello World!\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %msg, align 8
  %msg1 = load ptr, ptr %msg, align 8
  %function_call = call i32 @puts(ptr %msg1)
  %rhs = alloca i32, align 4
  store i32 2, ptr %rhs, align 4
  %a = alloca i32, align 4
  store i32 1000, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 1, ptr %b, align 4
  %a2 = alloca i32, align 4
  store i1 true, ptr %a2, align 1
  %b3 = alloca i32, align 4
  store i1 true, ptr %b3, align 1
  %eq = alloca i32, align 4
  %lhs_tmp = alloca i32, align 4
  %rhs_tmp = alloca i32, align 4
  %var_deref = load i32, ptr %a2, align 4
  store i32 %var_deref, ptr %lhs_tmp, align 4
  %var_deref4 = load i32, ptr %b3, align 4
  store i32 %var_deref4, ptr %rhs_tmp, align 4
  %lhs_tmp_val = load i32, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i32, ptr %rhs_tmp, align 4
  %cmp = icmp eq i32 %lhs_tmp_val, %rhs_tmp_val
  store i1 %cmp, ptr %eq, align 1
  %ret_tmp_var = alloca i32, align 4
  %var_deref5 = load i32, ptr %eq, align 4
  store i32 %var_deref5, ptr %ret_tmp_var, align 4
  %ret_tmp_var6 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var6
}
