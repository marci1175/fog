; ModuleID = 'main'
source_filename = "main"

%alma = type { ptr, i32 }

@idared = constant [7 x i8] c"idared\00"

declare void @printf(ptr, ...)

define i32 @return_0() {
main_fn_entry:
  %ret_tmp_var = alloca i32, align 4
  store i32 2, ptr %ret_tmp_var, align 4
  %ret_tmp_var1 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var1
}

define i32 @main() {
main_fn_entry:
  %idared = alloca %alma, align 8
  %strct_init = alloca %alma, align 8
  %nev = alloca ptr, align 8
  store ptr @idared, ptr %nev, align 8
  %nev1 = load ptr, ptr %nev, align 8
  %field_gep = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 0
  store ptr %nev1, ptr %field_gep, align 8
  %szin = alloca i32, align 4
  store i32 69, ptr %szin, align 4
  %szin2 = load i32, ptr %szin, align 4
  %field_gep3 = getelementptr inbounds %alma, ptr %strct_init, i32 0, i32 1
  store i32 %szin2, ptr %field_gep3, align 4
  %constructed_struct = load %alma, ptr %strct_init, align 8
  store %alma %constructed_struct, ptr %idared, align 8
  %marci = alloca [5 x %alma], align 8
  %array_temp_val_var = alloca %alma, align 8
  %strct_init4 = alloca %alma, align 8
  %nev5 = alloca ptr, align 8
  store ptr @idared, ptr %nev5, align 8
  %nev6 = load ptr, ptr %nev5, align 8
  %field_gep7 = getelementptr inbounds %alma, ptr %strct_init4, i32 0, i32 0
  store ptr %nev6, ptr %field_gep7, align 8
  %szin8 = alloca i32, align 4
  store i32 69, ptr %szin8, align 4
  %szin9 = load i32, ptr %szin8, align 4
  %field_gep10 = getelementptr inbounds %alma, ptr %strct_init4, i32 0, i32 1
  store i32 %szin9, ptr %field_gep10, align 4
  %constructed_struct11 = load %alma, ptr %strct_init4, align 8
  store %alma %constructed_struct11, ptr %array_temp_val_var, align 8
  %array_temp_val_deref = load %alma, ptr %array_temp_val_var, align 8
  %array_temp_val_var12 = alloca %alma, align 8
  %strct_init13 = alloca %alma, align 8
  %nev14 = alloca ptr, align 8
  store ptr @idared, ptr %nev14, align 8
  %nev15 = load ptr, ptr %nev14, align 8
  %field_gep16 = getelementptr inbounds %alma, ptr %strct_init13, i32 0, i32 0
  store ptr %nev15, ptr %field_gep16, align 8
  %szin17 = alloca i32, align 4
  store i32 69, ptr %szin17, align 4
  %szin18 = load i32, ptr %szin17, align 4
  %field_gep19 = getelementptr inbounds %alma, ptr %strct_init13, i32 0, i32 1
  store i32 %szin18, ptr %field_gep19, align 4
  %constructed_struct20 = load %alma, ptr %strct_init13, align 8
  store %alma %constructed_struct20, ptr %array_temp_val_var12, align 8
  %array_temp_val_deref21 = load %alma, ptr %array_temp_val_var12, align 8
  %array_temp_val_var22 = alloca %alma, align 8
  %strct_init23 = alloca %alma, align 8
  %nev24 = alloca ptr, align 8
  store ptr @idared, ptr %nev24, align 8
  %nev25 = load ptr, ptr %nev24, align 8
  %field_gep26 = getelementptr inbounds %alma, ptr %strct_init23, i32 0, i32 0
  store ptr %nev25, ptr %field_gep26, align 8
  %szin27 = alloca i32, align 4
  store i32 69, ptr %szin27, align 4
  %szin28 = load i32, ptr %szin27, align 4
  %field_gep29 = getelementptr inbounds %alma, ptr %strct_init23, i32 0, i32 1
  store i32 %szin28, ptr %field_gep29, align 4
  %constructed_struct30 = load %alma, ptr %strct_init23, align 8
  store %alma %constructed_struct30, ptr %array_temp_val_var22, align 8
  %array_temp_val_deref31 = load %alma, ptr %array_temp_val_var22, align 8
  %array_temp_val_var32 = alloca %alma, align 8
  %strct_init33 = alloca %alma, align 8
  %nev34 = alloca ptr, align 8
  store ptr @idared, ptr %nev34, align 8
  %nev35 = load ptr, ptr %nev34, align 8
  %field_gep36 = getelementptr inbounds %alma, ptr %strct_init33, i32 0, i32 0
  store ptr %nev35, ptr %field_gep36, align 8
  %szin37 = alloca i32, align 4
  store i32 69, ptr %szin37, align 4
  %szin38 = load i32, ptr %szin37, align 4
  %field_gep39 = getelementptr inbounds %alma, ptr %strct_init33, i32 0, i32 1
  store i32 %szin38, ptr %field_gep39, align 4
  %constructed_struct40 = load %alma, ptr %strct_init33, align 8
  store %alma %constructed_struct40, ptr %array_temp_val_var32, align 8
  %array_temp_val_deref41 = load %alma, ptr %array_temp_val_var32, align 8
  %array_temp_val_var42 = alloca %alma, align 8
  %strct_init43 = alloca %alma, align 8
  %nev44 = alloca ptr, align 8
  store ptr @idared, ptr %nev44, align 8
  %nev45 = load ptr, ptr %nev44, align 8
  %field_gep46 = getelementptr inbounds %alma, ptr %strct_init43, i32 0, i32 0
  store ptr %nev45, ptr %field_gep46, align 8
  %szin47 = alloca i32, align 4
  store i32 69, ptr %szin47, align 4
  %szin48 = load i32, ptr %szin47, align 4
  %field_gep49 = getelementptr inbounds %alma, ptr %strct_init43, i32 0, i32 1
  store i32 %szin48, ptr %field_gep49, align 4
  %constructed_struct50 = load %alma, ptr %strct_init43, align 8
  store %alma %constructed_struct50, ptr %array_temp_val_var42, align 8
  %array_temp_val_deref51 = load %alma, ptr %array_temp_val_var42, align 8
  %array_idx_val = getelementptr [5 x %alma], ptr %marci, i32 0, i32 0
  store %alma %array_temp_val_deref, ptr %array_idx_val, align 8
  %array_idx_val52 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 1
  store %alma %array_temp_val_deref21, ptr %array_idx_val52, align 8
  %array_idx_val53 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 2
  store %alma %array_temp_val_deref31, ptr %array_idx_val53, align 8
  %array_idx_val54 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 3
  store %alma %array_temp_val_deref41, ptr %array_idx_val54, align 8
  %array_idx_val55 = getelementptr [5 x %alma], ptr %marci, i32 0, i32 4
  store %alma %array_temp_val_deref51, ptr %array_idx_val55, align 8
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var56 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var56
}
