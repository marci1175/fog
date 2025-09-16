; ModuleID = 'main'
source_filename = "main"

%alma = type { ptr, i32 }

@idared = constant [7 x i8] c"idared\00"
@"The number is %i %s" = constant [20 x i8] c"The number is %i %s\00"

declare void @printf(ptr, ...)

define i32 @return_0() {
main_fn_entry:
  ret i32 2
}

define i32 @main() {
main_fn_entry:
  %idared = alloca %alma, align 8
  %strct_init = alloca %alma, align 8
  %field_gep = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 0
  store ptr @idared, ptr %field_gep, align 8
  %field_gep3 = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 1
  store i32 69, ptr %field_gep3, align 4
  %constructed_struct = load %alma, ptr %strct_init, align 8
  store %alma %constructed_struct, ptr %idared, align 8
  %marci = alloca [5 x i32], align 4
  %array_idx_val = getelementptr [5 x i32], ptr %marci, i32 0, i32 0
  store i32 90, ptr %array_idx_val, align 4
  %array_idx_val12 = getelementptr [5 x i32], ptr %marci, i32 0, i32 1
  store i32 4, ptr %array_idx_val12, align 4
  %array_idx_val13 = getelementptr [5 x i32], ptr %marci, i32 0, i32 2
  store i32 5, ptr %array_idx_val13, align 4
  %array_idx_val14 = getelementptr [5 x i32], ptr %marci, i32 0, i32 3
  store i32 6, ptr %array_idx_val14, align 4
  %array_idx_val15 = getelementptr [5 x i32], ptr %marci, i32 0, i32 4
  store i32 7, ptr %array_idx_val15, align 4
  %array_idx_elem = getelementptr [5 x i32], ptr %marci, i32 0, i32 2
  %idx_array_val_deref = load i32, ptr %array_idx_elem, align 4
  %deref_strct_val = load ptr, ptr %idared, align 8
  call void (ptr, ...) @printf(ptr @"The number is %i %s", i32 %idx_array_val_deref, ptr %deref_strct_val)
  ret i32 0
}
