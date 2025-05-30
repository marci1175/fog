; ModuleID = 'main'
source_filename = "main"

%test = type { i32, i32, { i32 } }
%asd = type { i32 }

define i32 @main() {
main_fn_entry:
  %var1 = alloca %test, align 8
  %0 = alloca %test, align 8
  %field_gep = getelementptr inbounds %test, ptr %0, i32 0, i32 0
  store i32 0, ptr %field_gep, align 4
  %field_gep3 = getelementptr inbounds %test, ptr %0, i32 0, i32 1
  store i32 1, ptr %field_gep3, align 4
  %field3 = alloca %asd, align 8
  %1 = alloca %asd, align 8
  %field_gep5 = getelementptr inbounds %asd, ptr %1, i32 0, i32 0
  store i32 23, ptr %field_gep5, align 4
  %2 = load %asd, ptr %1, align 4
  store %asd %2, ptr %field3, align 4
  %field36 = load { i32 }, ptr %field3, align 4
  %field_gep7 = getelementptr inbounds %test, ptr %0, i32 0, i32 2
  store { i32 } %field36, ptr %field_gep7, align 4
  %3 = load %test, ptr %0, align 4
  store %test %3, ptr %var1, align 4
  %var1_inner = load %test, ptr %var1, align 4
  %inner_ref = load i32, ptr %var1, align 4
  ret i32 %inner_ref
}
