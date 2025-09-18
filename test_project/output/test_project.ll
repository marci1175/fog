; ModuleID = 'main'
source_filename = "main"

%alma = type { ptr, i32 }

@idared = constant [7 x i8] c"idared\00"
@"The name is %s" = constant [15 x i8] c"The name is %s\00"

declare void @printf(ptr, ...)

define i32 @return_0() {
main_fn_entry:
  ret i32 2
}

define i32 @main() {
main_fn_entry:
  %strct_init = alloca %alma, align 8
  %field_gep = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 0
  store ptr @idared, ptr %field_gep, align 8
  %field_gep3 = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 1
  store i32 69, ptr %field_gep3, align 4
  %constructed_struct = load %alma, ptr %strct_init, align 8
  %marci = alloca [5 x %alma], align 8
  %strct_init4 = alloca %alma, align 8
  %field_gep7 = getelementptr inbounds %alma, ptr %strct_init4, i32 0, i32 0
  store ptr @idared, ptr %field_gep7, align 8
  %field_gep10 = getelementptr inbounds %alma, ptr %strct_init4, i32 0, i32 1
  store i32 69, ptr %field_gep10, align 4
  %constructed_struct11 = load %alma, ptr %strct_init4, align 8
  %strct_init13 = alloca %alma, align 8
  %field_gep16 = getelementptr inbounds %alma, ptr %strct_init13, i32 0, i32 0
  store ptr @idared, ptr %field_gep16, align 8
  %field_gep19 = getelementptr inbounds %alma, ptr %strct_init13, i32 0, i32 1
  store i32 69, ptr %field_gep19, align 4
  %constructed_struct20 = load %alma, ptr %strct_init13, align 8
  %strct_init23 = alloca %alma, align 8
  %field_gep26 = getelementptr inbounds %alma, ptr %strct_init23, i32 0, i32 0
  store ptr @idared, ptr %field_gep26, align 8
  %field_gep29 = getelementptr inbounds %alma, ptr %strct_init23, i32 0, i32 1
  store i32 69, ptr %field_gep29, align 4
  %constructed_struct30 = load %alma, ptr %strct_init23, align 8
  %strct_init33 = alloca %alma, align 8
  %field_gep36 = getelementptr inbounds %alma, ptr %strct_init33, i32 0, i32 0
  store ptr @idared, ptr %field_gep36, align 8
  %field_gep39 = getelementptr inbounds %alma, ptr %strct_init33, i32 0, i32 1
  store i32 69, ptr %field_gep39, align 4
  %constructed_struct40 = load %alma, ptr %strct_init33, align 8
  %strct_init43 = alloca %alma, align 8
  %field_gep46 = getelementptr inbounds %alma, ptr %strct_init43, i32 0, i32 0
  store ptr @idared, ptr %field_gep46, align 8
  %field_gep49 = getelementptr inbounds %alma, ptr %strct_init43, i32 0, i32 1
  store i32 69, ptr %field_gep49, align 4
  %constructed_struct50 = load %alma, ptr %strct_init43, align 8
  %array_idx_val = getelementptr [5 x %alma], ptr %marci, i32 0, i32 0
  store %alma %constructed_struct11, ptr %array_idx_val, align 8
  %array_idx_val52 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 1
  store %alma %constructed_struct20, ptr %array_idx_val52, align 8
  %array_idx_val53 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 2
  store %alma %constructed_struct30, ptr %array_idx_val53, align 8
  %array_idx_val54 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 3
  store %alma %constructed_struct40, ptr %array_idx_val54, align 8
  %array_idx_val55 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 4
  store %alma %constructed_struct50, ptr %array_idx_val55, align 8
  %idared56 = alloca %alma, align 8
  %array_idx_elem = getelementptr [5 x %alma], ptr %marci, i32 0, i32 2
  %idx_array_val_deref = load %alma, ptr %array_idx_elem, align 8
  store %alma %idx_array_val_deref, ptr %idared56, align 8
  %deref_strct_val = load ptr, ptr %idared56, align 8
  call void (ptr, ...) @printf(ptr @"The name is %s", ptr %deref_strct_val)
  ret i32 0
}
