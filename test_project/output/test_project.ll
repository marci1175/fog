; ModuleID = 'main'
source_filename = "main"

%asd = type { float }

declare i32 @puts(ptr)

define i32 @main() {
main_fn_entry:
  %a = alloca %asd, align 8
  %strct_init = alloca %asd, align 8
  %inner = alloca float, align 4
  store float 0x4037333340000000, ptr %inner, align 4
  %inner1 = load float, ptr %inner, align 4
  %field_gep = getelementptr inbounds %asd, ptr %strct_init, i32 0, i32 0
  store float %inner1, ptr %field_gep, align 4
  %constructed_struct = load %asd, ptr %strct_init, align 4
  store %asd %constructed_struct, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 123, ptr %b, align 4
  %eq = alloca i32, align 4
  %lhs_tmp = alloca float, align 4
  %rhs_tmp = alloca float, align 4
  %deref_strct_val = load float, ptr %a, align 4
  store float %deref_strct_val, ptr %lhs_tmp, align 4
  store float 0x4037333340000000, ptr %rhs_tmp, align 4
  %lhs_tmp_val = load float, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load float, ptr %rhs_tmp, align 4
  %cmp = fcmp oeq float %lhs_tmp_val, %rhs_tmp_val
  store i1 %cmp, ptr %eq, align 1
  %ret_tmp_var = alloca i32, align 4
  %var_deref = load i32, ptr %eq, align 4
  store i32 %var_deref, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var2
}
