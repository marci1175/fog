; ModuleID = 'main'
source_filename = "main"

%test = type { i32, i32, { i32 } }
%ligma = type { i32 }

define i32 @main() {
main_fn_entry:
  %var1 = alloca %test, align 8
  %0 = alloca %test, align 8
  %field_gep = getelementptr inbounds %test, ptr %0, i32 0, i32 0
  store i32 0, ptr %field_gep, align 4
  %field_gep3 = getelementptr inbounds %test, ptr %0, i32 0, i32 1
  store i32 69, ptr %field_gep3, align 4
  %asd = alloca %ligma, align 8
  %1 = alloca %ligma, align 8
  %field_gep5 = getelementptr inbounds %ligma, ptr %1, i32 0, i32 0
  store i32 420, ptr %field_gep5, align 4
  %2 = load %ligma, ptr %1, align 4
  store %ligma %2, ptr %asd, align 4
  %asd6 = load { i32 }, ptr %asd, align 4
  %field_gep7 = getelementptr inbounds %test, ptr %0, i32 0, i32 2
  store { i32 } %asd6, ptr %field_gep7, align 4
  %3 = load %test, ptr %0, align 4
  store %test %3, ptr %var1, align 4
  %deref_nested_strct = getelementptr inbounds %test, ptr %var1, i32 0, i32 2
  %deref_strct_val = load i32, ptr %deref_nested_strct, align 4
  ret i32 %deref_strct_val
}
