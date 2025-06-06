; ModuleID = 'main'
source_filename = "main"

define i32 @main() {
main_fn_entry:
  %q = alloca i1, align 1
  %lhs_tmp = alloca double, align 8
  %rhs_tmp = alloca double, align 8
  store double 2.300000e+00, ptr %lhs_tmp, align 8
  store double 4.300000e+00, ptr %rhs_tmp, align 8
  %lhs_tmp_val = load double, ptr %lhs_tmp, align 8
  %rhs_tmp_val = load double, ptr %rhs_tmp, align 8
  %cmp = fcmp olt double %lhs_tmp_val, %rhs_tmp_val
  store i1 %cmp, ptr %q, align 1
  %ret_tmp_var = alloca i32, align 4
  %0 = load i1, ptr %q, align 1
  %bool_to_i32 = zext i1 %0 to i32
  store i32 %bool_to_i32, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var1
}
