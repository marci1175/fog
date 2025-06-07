; ModuleID = 'main'
source_filename = "main"

%marci = type { %kg }
%kg = type { float }

declare i32 @printf(ptr, i32, float)

define i32 @main() {
main_fn_entry:
  %szemely = alloca %marci, align 8
  %strct_init = alloca %marci, align 8
  %suly = alloca %kg, align 8
  %strct_init1 = alloca %kg, align 8
  %inner = alloca float, align 4
  store float 0x4052133340000000, ptr %inner, align 4
  %inner2 = load float, ptr %inner, align 4
  %field_gep = getelementptr inbounds %kg, ptr %strct_init1, i32 0, i32 0
  store float %inner2, ptr %field_gep, align 4
  %constructed_struct = load %kg, ptr %strct_init1, align 4
  store %kg %constructed_struct, ptr %suly, align 4
  %suly3 = load %kg, ptr %suly, align 4
  %field_gep4 = getelementptr inbounds %marci, ptr %strct_init, i32 0, i32 0
  store %kg %suly3, ptr %field_gep4, align 4
  %constructed_struct5 = load %marci, ptr %strct_init, align 4
  store %marci %constructed_struct5, ptr %szemely, align 4
  %str = alloca ptr, align 8
  %string_buffer = alloca [14 x i8], align 1
  store [14 x i8] c"value: %d, %f\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %str, align 8
  %str6 = load ptr, ptr %str, align 8
  %val = alloca i32, align 4
  store i32 23, ptr %val, align 4
  %val7 = load i32, ptr %val, align 4
  %val2 = alloca float, align 4
  store float 0x4037333340000000, ptr %val2, align 4
  %val28 = load float, ptr %val2, align 4
  %function_call = call i32 @printf(ptr %str6, i32 %val7, float %val28)
  %0 = alloca i32, align 4
  store i32 %function_call, ptr %0, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var9 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var9
}
