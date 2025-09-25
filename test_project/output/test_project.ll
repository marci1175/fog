; ModuleID = 'main'
source_filename = "main"

@"Hello world!\0A" = constant [14 x i8] c"Hello world!\0A\00"

declare void @printf(ptr, ...)

define i32 @main() {
main_fn_entry:
  %input = alloca ptr, align 8
  store ptr @"Hello world!\0A", ptr %input, align 8
  %input1 = load ptr, ptr %input, align 8
  call void (ptr, ...) @printf(ptr %input1)
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var2 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var2
}
