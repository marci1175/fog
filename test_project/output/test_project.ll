; ModuleID = 'main'
source_filename = "main"

%osztaly = type { [3 x ptr] }

@marci = constant [6 x i8] c"marci\00"

define i32 @return_0() {
main_fn_entry:
  ret i32 2
}

define i32 @main() {
main_fn_entry:
  %strct_init = alloca %osztaly, align 8
  %diakok = alloca [3 x ptr], align 8
  %array_idx_val = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 0
  store ptr @marci, ptr %array_idx_val, align 8
  %array_idx_val5 = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val5, align 8
  %array_idx_val6 = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val6, align 8
  %diakok7 = load [3 x ptr], ptr %diakok, align 8
  %field_gep = getelementptr inbounds %osztaly, ptr %strct_init, i32 0, i32 0
  store [3 x ptr] %diakok7, ptr %field_gep, align 8
  %constructed_struct = load %osztaly, ptr %strct_init, align 8
  ret i32 0
}
