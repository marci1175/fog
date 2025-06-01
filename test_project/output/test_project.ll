; ModuleID = 'main'
source_filename = "main"

%test = type { i32, %asd }
%asd = type { ptr }

declare i32 @puts(ptr)

define i32 @main() {
main_fn_entry:
  %msg = alloca ptr, align 8
  %string_buffer = alloca [13 x i8], align 1
  store [13 x i8] c"Hello World!\00", ptr %string_buffer, align 1
  store ptr %string_buffer, ptr %msg, align 8
  %msg1 = load ptr, ptr %msg, align 8
  %function_call = call i32 @puts(ptr %msg1)
  %a1 = alloca %test, align 8
  %strct_init = alloca %test, align 8
  %inner = alloca i32, align 4
  store i32 32, ptr %inner, align 4
  %inner2 = load i32, ptr %inner, align 4
  %field_gep = getelementptr inbounds %test, ptr %strct_init, i32 0, i32 0
  store i32 %inner2, ptr %field_gep, align 4
  %asd = alloca %asd, align 8
  %strct_init3 = alloca %asd, align 8
  %asd4 = alloca ptr, align 8
  %string_buffer5 = alloca [20 x i8], align 1
  store [20 x i8] c"Inner value of asd.\00", ptr %string_buffer5, align 1
  store ptr %string_buffer5, ptr %asd4, align 8
  %asd6 = load ptr, ptr %asd4, align 8
  %field_gep7 = getelementptr inbounds %asd, ptr %strct_init3, i32 0, i32 0
  store ptr %asd6, ptr %field_gep7, align 8
  %constructed_struct = load %asd, ptr %strct_init3, align 8
  store %asd %constructed_struct, ptr %asd, align 8
  %asd8 = load %asd, ptr %asd, align 8
  %field_gep9 = getelementptr inbounds %test, ptr %strct_init, i32 0, i32 1
  store %asd %asd8, ptr %field_gep9, align 8
  %constructed_struct10 = load %test, ptr %strct_init, align 8
  store %test %constructed_struct10, ptr %a1, align 8
  store i32 23, ptr %a1, align 4
  %msg11 = alloca ptr, align 8
  %deref_nested_strct = getelementptr inbounds %test, ptr %a1, i32 0, i32 1
  %deref_strct_val = load ptr, ptr %deref_nested_strct, align 8
  store ptr %deref_strct_val, ptr %msg11, align 8
  %msg12 = load ptr, ptr %msg11, align 8
  %function_call13 = call i32 @puts(ptr %msg12)
  %ret_tmp_var = alloca i32, align 4
  %deref_strct_val14 = load i32, ptr %a1, align 4
  store i32 %deref_strct_val14, ptr %ret_tmp_var, align 4
  %ret_tmp_var15 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var15
}
