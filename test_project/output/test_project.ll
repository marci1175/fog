; ModuleID = 'main'
source_filename = "main"

%test = type { i32 }

declare i32 @puts(ptr)

define i32 @main() {
main_fn_entry:
  %string_buffer = alloca [13 x i8], align 1
  store [13 x i8] c"Hello World!\00", ptr %string_buffer, align 1
  %function_call = call i32 @puts(ptr %string_buffer)
  %a1 = alloca %test, align 8
  %strct_init = alloca %test, align 8
  %field_gep = getelementptr inbounds %test, ptr %strct_init, i32 0, i32 0
  store i32 32, ptr %field_gep, align 4
  %constructed_struct = load %test, ptr %strct_init, align 4
  store %test %constructed_struct, ptr %a1, align 4
  store i32 23, ptr %a1, align 4
  %deref_strct_val = load i32, ptr %a1, align 4
  ret i32 %deref_strct_val
}
