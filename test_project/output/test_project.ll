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
  %marci = alloca [2 x [2 x %alma]], align 8
  %array_temp_val_var = alloca [2 x %alma], align 8
  %strct_init5 = alloca %alma, align 8
  %field_gep8 = getelementptr inbounds %alma, ptr %strct_init5, i32 0, i32 0
  store ptr @idared, ptr %field_gep8, align 8
  %field_gep11 = getelementptr inbounds %alma, ptr %strct_init5, i32 0, i32 1
  store i32 69, ptr %field_gep11, align 4
  %constructed_struct12 = load %alma, ptr %strct_init5, align 8
  %strct_init14 = alloca %alma, align 8
  %field_gep17 = getelementptr inbounds %alma, ptr %strct_init14, i32 0, i32 0
  store ptr @idared, ptr %field_gep17, align 8
  %field_gep20 = getelementptr inbounds %alma, ptr %strct_init14, i32 0, i32 1
  store i32 69, ptr %field_gep20, align 4
  %constructed_struct21 = load %alma, ptr %strct_init14, align 8
  %array_idx_val = getelementptr [2 x %alma], ptr %array_temp_val_var, i32 0, i32 0
  store %alma %constructed_struct12, ptr %array_idx_val, align 8
  %array_idx_val23 = getelementptr [2 x %alma], ptr %array_temp_val_var, i32 0, i32 1
  store %alma %constructed_struct21, ptr %array_idx_val23, align 8
  %array_temp_val_deref24 = load [2 x %alma], ptr %array_temp_val_var, align 8
  %array_temp_val_var25 = alloca [2 x %alma], align 8
  %strct_init27 = alloca %alma, align 8
  %field_gep30 = getelementptr inbounds %alma, ptr %strct_init27, i32 0, i32 0
  store ptr @idared, ptr %field_gep30, align 8
  %field_gep33 = getelementptr inbounds %alma, ptr %strct_init27, i32 0, i32 1
  store i32 69, ptr %field_gep33, align 4
  %constructed_struct34 = load %alma, ptr %strct_init27, align 8
  %strct_init37 = alloca %alma, align 8
  %field_gep40 = getelementptr inbounds %alma, ptr %strct_init37, i32 0, i32 0
  store ptr @idared, ptr %field_gep40, align 8
  %field_gep43 = getelementptr inbounds %alma, ptr %strct_init37, i32 0, i32 1
  store i32 69, ptr %field_gep43, align 4
  %constructed_struct44 = load %alma, ptr %strct_init37, align 8
  %array_idx_val46 = getelementptr [2 x %alma], ptr %array_temp_val_var25, i32 0, i32 0
  store %alma %constructed_struct34, ptr %array_idx_val46, align 8
  %array_idx_val47 = getelementptr [2 x %alma], ptr %array_temp_val_var25, i32 0, i32 1
  store %alma %constructed_struct44, ptr %array_idx_val47, align 8
  %array_temp_val_deref48 = load [2 x %alma], ptr %array_temp_val_var25, align 8
  %array_idx_val49 = getelementptr [2 x [2 x %alma]], ptr %marci, i32 0, i32 0
  store [2 x %alma] %array_temp_val_deref24, ptr %array_idx_val49, align 8
  %array_idx_val50 = getelementptr [2 x [2 x %alma]], ptr %marci, i32 0, i32 1
  store [2 x %alma] %array_temp_val_deref48, ptr %array_idx_val50, align 8
  %idared_list = alloca [2 x %alma], align 8
  %array_idx_elem = getelementptr [2 x [2 x %alma]], ptr %marci, i32 0, i32 2
  %idx_array_val_deref = load [2 x %alma], ptr %array_idx_elem, align 8
  store [2 x %alma] %idx_array_val_deref, ptr %idared_list, align 8
  %idared52 = alloca %alma, align 8
  %array_idx_elem54 = getelementptr [2 x %alma], ptr %idared_list, i32 0, i32 0
  %idx_array_val_deref55 = load %alma, ptr %array_idx_elem54, align 8
  store %alma %idx_array_val_deref55, ptr %idared52, align 8
  %deref_strct_val = load ptr, ptr %idared52, align 8
  call void (ptr, ...) @printf(ptr @"The name is %s", ptr %deref_strct_val)
  ret i32 0
}
