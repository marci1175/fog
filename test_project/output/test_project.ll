; ModuleID = 'main'
source_filename = "main"

declare i32 @puts(ptr)

declare i32 @gets(ptr)

define i32 @main() {
main_fn_entry:
  %input = alloca ptr, align 8
  %string_buffer = alloca [22 x i8], align 1
  store [22 x i8] c"asdasdasdasdasdasdasd\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %input, align 8
  %a = alloca float, align 4
  store float 5.000000e+00, ptr %a, align 4
  %b = alloca float, align 4
  store float 0x4038666660000000, ptr %b, align 4
  %q = alloca i1, align 1
  %lhs_tmp = alloca double, align 8
  %rhs_tmp = alloca double, align 8
  store double 2.300000e+00, ptr %lhs_tmp, align 8
  store double 4.300000e+00, ptr %rhs_tmp, align 8
  %lhs_tmp_val = load double, ptr %lhs_tmp, align 8
  %rhs_tmp_val = load double, ptr %rhs_tmp, align 8
  %cmp = fcmp ogt double %lhs_tmp_val, %rhs_tmp_val
  store i1 %cmp, ptr %q, align 1
  %ret_tmp_var = alloca i32, align 4
  %0 = load i1, ptr %q, align 1
  %ty_cast_check = icmp eq i1 %0, i32 0
}

define i1 @ty_cmp_check() {
ty_cast_val_check:
  ret i1 true
}

define i1 @ty_cmp_check.1() {
ty_cast_val_check:
  ret i1 false
  br i1 %ty_cast_check, label %ty_cast_val_check, label %ty_cast_val_check
  store i1 %0, ptr %q, align 1
  %ret_tmp_var = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var
}
