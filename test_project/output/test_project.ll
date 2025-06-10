; ModuleID = 'main'
source_filename = "main"

declare i32 @printf(ptr, i32, ptr, ptr)

define i32 @asd(i32 %a, float %b) {
main_fn_entry:
  %a1 = alloca i32, align 4
  store i32 %a, ptr %a1, align 4
  %b2 = alloca float, align 4
  store float %b, ptr %b2, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 2, ptr %ret_tmp_var, align 4
  %ret_tmp_var3 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var3
}

define i32 @main() {
main_fn_entry:
  %str = alloca ptr, align 8
  %string_buffer = alloca [18 x i8], align 1
  store [18 x i8] c"value: %d, %s, %s\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %str, align 8
  %str1 = load ptr, ptr %str, align 8
  %val = alloca i32, align 4
  %a = alloca i32, align 4
  store i32 23, ptr %a, align 4
  %a2 = load i32, ptr %a, align 4
  %b = alloca float, align 4
  store float 0x4037666660000000, ptr %b, align 4
  %b3 = load float, ptr %b, align 4
  %function_call = call i32 @asd(i32 %a2, float %b3)
  store i32 %function_call, ptr %val, align 4
  %val4 = load i32, ptr %val, align 4
  store i32 %val4, ptr %val, align 4
  %val5 = load i32, ptr %val, align 4
  %val3 = alloca ptr, align 8
  %string_buffer6 = alloca [5 x i8], align 1
  store [5 x i8] c"szia\00", ptr %string_buffer6, align 1
  store ptr %string_buffer6, ptr %val3, align 8
  %val37 = load ptr, ptr %val3, align 8
  %val48 = alloca ptr, align 8
  %string_buffer9 = alloca [4 x i8], align 1
  store [4 x i8] c"udv\00", ptr %string_buffer9, align 1
  store ptr %string_buffer9, ptr %val48, align 8
  %val410 = load ptr, ptr %val48, align 8
  %function_call11 = call i32 @printf(ptr %str1, i32 %val5, ptr %val37, ptr %val410)
  %0 = alloca i32, align 4
  store i32 %function_call11, ptr %0, align 4
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var12 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var12
}
