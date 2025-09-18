; ModuleID = 'main'
source_filename = "main"

%alma = type { ptr, i32 }

@idared = constant [7 x i8] c"idared\00"

define i32 @return_0() {
main_fn_entry:
  ret i32 2
}

define i32 @main() {
main_fn_entry:
  %marci1 = alloca [2 x [2 x %alma]], align 8
  %array_temp_val_var = alloca [2 x %alma], align 8
  %strct_init = alloca %alma, align 8
  %field_gep = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 0
  store ptr @idared, ptr %field_gep, align 8
  %field_gep4 = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 1
  store i32 69, ptr %field_gep4, align 4
  %constructed_struct = load %alma, ptr %strct_init, align 8
  %strct_init6 = alloca %alma, align 8
  %field_gep9 = getelementptr inbounds %alma, ptr %strct_init6, i32 0, i32 0
  store ptr @idared, ptr %field_gep9, align 8
  %field_gep12 = getelementptr inbounds %alma, ptr %strct_init6, i32 0, i32 1
  store i32 69, ptr %field_gep12, align 4
  %constructed_struct13 = load %alma, ptr %strct_init6, align 8
  %array_idx_val = getelementptr [2 x %alma], ptr %array_temp_val_var, i32 0, i32 0
  store %alma %constructed_struct, ptr %array_idx_val, align 8
  %array_idx_val15 = getelementptr [2 x %alma], ptr %array_temp_val_var, i32 0, i32 1
  store %alma %constructed_struct13, ptr %array_idx_val15, align 8
  %array_temp_val_deref16 = load [2 x %alma], ptr %array_temp_val_var, align 8
  %array_temp_val_var17 = alloca [2 x %alma], align 8
  %strct_init19 = alloca %alma, align 8
  %field_gep22 = getelementptr inbounds %alma, ptr %strct_init19, i32 0, i32 0
  store ptr @idared, ptr %field_gep22, align 8
  %field_gep25 = getelementptr inbounds %alma, ptr %strct_init19, i32 0, i32 1
  store i32 69, ptr %field_gep25, align 4
  %constructed_struct26 = load %alma, ptr %strct_init19, align 8
  %strct_init29 = alloca %alma, align 8
  %field_gep32 = getelementptr inbounds %alma, ptr %strct_init29, i32 0, i32 0
  store ptr @idared, ptr %field_gep32, align 8
  %field_gep35 = getelementptr inbounds %alma, ptr %strct_init29, i32 0, i32 1
  store i32 69, ptr %field_gep35, align 4
  %constructed_struct36 = load %alma, ptr %strct_init29, align 8
  %array_idx_val38 = getelementptr [2 x %alma], ptr %array_temp_val_var17, i32 0, i32 0
  store %alma %constructed_struct26, ptr %array_idx_val38, align 8
  %array_idx_val39 = getelementptr [2 x %alma], ptr %array_temp_val_var17, i32 0, i32 1
  store %alma %constructed_struct36, ptr %array_idx_val39, align 8
  %array_temp_val_deref40 = load [2 x %alma], ptr %array_temp_val_var17, align 8
  %array_idx_val41 = getelementptr [2 x [2 x %alma]], ptr %marci1, i32 0, i32 0
  store [2 x %alma] %array_temp_val_deref16, ptr %array_idx_val41, align 8
  %array_idx_val42 = getelementptr [2 x [2 x %alma]], ptr %marci1, i32 0, i32 1
  store [2 x %alma] %array_temp_val_deref40, ptr %array_idx_val42, align 8
  ret i32 0
}
