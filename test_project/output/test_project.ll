; ModuleID = 'main'
source_filename = "main"

%a1 = type { i32 }
%b1 = type { i32, %a1 }

define i32 @main() {
main_fn_entry:
  %strct_init = alloca %a1, align 8
  %field_gep = getelementptr inbounds %a1, ptr %strct_init, i32 0, i32 0
  store i32 92, ptr %field_gep, align 4
  %constructed_struct = load %a1, ptr %strct_init, align 4
  %strct_init3 = alloca %b1, align 8
  %field_gep6 = getelementptr inbounds %b1, ptr %strct_init3, i32 0, i32 0
  store i32 92, ptr %field_gep6, align 4
  %strct_init7 = alloca %a1, align 8
  %field_gep10 = getelementptr inbounds %a1, ptr %strct_init7, i32 0, i32 0
  store i32 2, ptr %field_gep10, align 4
  %constructed_struct11 = load %a1, ptr %strct_init7, align 4
  %field_gep13 = getelementptr inbounds %b1, ptr %strct_init3, i32 0, i32 1
  store %a1 %constructed_struct11, ptr %field_gep13, align 4
  %constructed_struct14 = load %b1, ptr %strct_init3, align 4
  ret i32 0
}
