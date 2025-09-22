; ModuleID = 'main'
source_filename = "main"

%alma = type { ptr, i32 }

@granny = constant [7 x i8] c"granny\00"
@finom = constant [6 x i8] c"finom\00"
@szhar = constant [6 x i8] c"szhar\00"
@"Alma neve: %s" = constant [14 x i8] c"Alma neve: %s\00"

declare void @printf(ptr, ...)

define i32 @return_0() {
main_fn_entry:
  ret i32 2
}

define i32 @main() {
main_fn_entry:
  %kosar = alloca [3 x %alma], align 8
  %strct_init = alloca %alma, align 8
  %field_gep = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 0
  store ptr @granny, ptr %field_gep, align 8
  %field_gep3 = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 1
  store i32 1, ptr %field_gep3, align 4
  %constructed_struct = load %alma, ptr %strct_init, align 8
  %strct_init5 = alloca %alma, align 8
  %field_gep8 = getelementptr inbounds %alma, ptr %strct_init5, i32 0, i32 0
  store ptr @finom, ptr %field_gep8, align 8
  %field_gep11 = getelementptr inbounds %alma, ptr %strct_init5, i32 0, i32 1
  store i32 2, ptr %field_gep11, align 4
  %constructed_struct12 = load %alma, ptr %strct_init5, align 8
  %strct_init15 = alloca %alma, align 8
  %field_gep18 = getelementptr inbounds %alma, ptr %strct_init15, i32 0, i32 0
  store ptr @szhar, ptr %field_gep18, align 8
  %field_gep21 = getelementptr inbounds %alma, ptr %strct_init15, i32 0, i32 1
  store i32 3, ptr %field_gep21, align 4
  %constructed_struct22 = load %alma, ptr %strct_init15, align 8
  %array_idx_val = getelementptr [3 x %alma], ptr %kosar, i32 0, i32 0
  store %alma %constructed_struct, ptr %array_idx_val, align 8
  %array_idx_val24 = getelementptr [3 x %alma], ptr %kosar, i32 0, i32 1
  store %alma %constructed_struct12, ptr %array_idx_val24, align 8
  %array_idx_val25 = getelementptr [3 x %alma], ptr %kosar, i32 0, i32 2
  store %alma %constructed_struct22, ptr %array_idx_val25, align 8
  %szhar_alma = alloca %alma, align 8
  %array_idx_elem = getelementptr [3 x %alma], ptr %kosar, i32 0, i32 2
  %idx_array_val_deref = load %alma, ptr %array_idx_elem, align 8
  store %alma %idx_array_val_deref, ptr %szhar_alma, align 8
  %deref_strct_val = load ptr, ptr %szhar_alma, align 8
  call void (ptr, ...) @printf(ptr @"Alma neve: %s", ptr %deref_strct_val)
  ret i32 0
}
