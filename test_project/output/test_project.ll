; ModuleID = 'main'
source_filename = "main"

%test = type { ptr, i32, float, ptr }

declare i32 @print(ptr)

define i32 @main() {
main_fn_entry:
  %marci = alloca %test, align 8
  %name = alloca ptr, align 8
  %string_buffer = alloca [6 x i8], align 1
  store [6 x i8] c"Marci\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %name, align 8
  %name1 = load ptr, ptr %name, align 8
  %age = alloca i32, align 4
  store i32 32, ptr %age, align 4
  %age2 = load i32, ptr %age, align 4
  %igen = alloca float, align 4
  store float 0x4037333340000000, ptr %igen, align 4
  %igen3 = load float, ptr %igen, align 4
  %misc = alloca ptr, align 8
  %string_buffer4 = alloca [4 x i8], align 1
  store [4 x i8] c"agh\00", ptr %string_buffer4, align 1
  store ptr %string_buffer4, ptr %misc, align 8
  %misc5 = load ptr, ptr %misc, align 8
  store %test { ptr %name1, i32 %age2, float %igen3, ptr %misc5 }, ptr %marci, align 8
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var6 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var6
}
