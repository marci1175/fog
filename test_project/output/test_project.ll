; ModuleID = 'main'
source_filename = "main"

%test = type { i32, i32 }

define i32 @main() {
main_fn_entry:
  %var1 = alloca %test, align 8
  %0 = alloca %test, align 8
  %field_gep = getelementptr inbounds %test, ptr %0, i32 0, i32 0
  store i32 0, ptr %field_gep, align 4
  %field_gep3 = getelementptr inbounds %test, ptr %0, i32 0, i32 1
  store i32 69, ptr %field_gep3, align 4
  %1 = load %test, ptr %0, align 4
  store %test %1, ptr %var1, align 4
  %field24 = getelementptr inbounds %test, ptr %var1, i32 0, i32 1
  %loaded_val = load i32, ptr %field24, align 4
  ret i32 %loaded_val
}
