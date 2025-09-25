; ModuleID = 'main'
source_filename = "main"

%osztaly = type { [3 x ptr] }

@marci30 = constant [8 x i8] c"marci30\00"
@marci = constant [6 x i8] c"marci\00"
@Hello = constant [6 x i8] c"Hello\00"
@"Termeszporkolt: %s" = constant [19 x i8] c"Termeszporkolt: %s\00"

declare void @printf(ptr, ...)

define i32 @return_0() {
main_fn_entry:
  ret i32 2
}

define i32 @main() {
main_fn_entry:
  %osztalyok = alloca [4 x [2 x %osztaly]], align 8
  %array_temp_val_var = alloca [2 x %osztaly], align 8
  %strct_init = alloca %osztaly, align 8
  %diakok = alloca [3 x ptr], align 8
  %array_idx_val = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val, align 8
  %array_idx_val7 = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val7, align 8
  %array_idx_val8 = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val8, align 8
  %diakok9 = load [3 x ptr], ptr %diakok, align 8
  %field_gep = getelementptr inbounds %osztaly, ptr %strct_init, i32 0, i32 0
  store [3 x ptr] %diakok9, ptr %field_gep, align 8
  %constructed_struct = load %osztaly, ptr %strct_init, align 8
  %strct_init12 = alloca %osztaly, align 8
  %diakok13 = alloca [3 x ptr], align 8
  %array_idx_val20 = getelementptr [3 x ptr], ptr %diakok13, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val20, align 8
  %array_idx_val21 = getelementptr [3 x ptr], ptr %diakok13, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val21, align 8
  %array_idx_val22 = getelementptr [3 x ptr], ptr %diakok13, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val22, align 8
  %diakok23 = load [3 x ptr], ptr %diakok13, align 8
  %field_gep24 = getelementptr inbounds %osztaly, ptr %strct_init12, i32 0, i32 0
  store [3 x ptr] %diakok23, ptr %field_gep24, align 8
  %constructed_struct25 = load %osztaly, ptr %strct_init12, align 8
  %array_idx_val27 = getelementptr [2 x %osztaly], ptr %array_temp_val_var, i32 0, i32 0
  store %osztaly %constructed_struct, ptr %array_idx_val27, align 8
  %array_idx_val28 = getelementptr [2 x %osztaly], ptr %array_temp_val_var, i32 0, i32 1
  store %osztaly %constructed_struct25, ptr %array_idx_val28, align 8
  %array_temp_val_deref29 = load [2 x %osztaly], ptr %array_temp_val_var, align 8
  %array_temp_val_var30 = alloca [2 x %osztaly], align 8
  %strct_init32 = alloca %osztaly, align 8
  %diakok33 = alloca [3 x ptr], align 8
  %array_idx_val40 = getelementptr [3 x ptr], ptr %diakok33, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val40, align 8
  %array_idx_val41 = getelementptr [3 x ptr], ptr %diakok33, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val41, align 8
  %array_idx_val42 = getelementptr [3 x ptr], ptr %diakok33, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val42, align 8
  %diakok43 = load [3 x ptr], ptr %diakok33, align 8
  %field_gep44 = getelementptr inbounds %osztaly, ptr %strct_init32, i32 0, i32 0
  store [3 x ptr] %diakok43, ptr %field_gep44, align 8
  %constructed_struct45 = load %osztaly, ptr %strct_init32, align 8
  %strct_init48 = alloca %osztaly, align 8
  %diakok49 = alloca [3 x ptr], align 8
  %array_idx_val56 = getelementptr [3 x ptr], ptr %diakok49, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val56, align 8
  %array_idx_val57 = getelementptr [3 x ptr], ptr %diakok49, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val57, align 8
  %array_idx_val58 = getelementptr [3 x ptr], ptr %diakok49, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val58, align 8
  %diakok59 = load [3 x ptr], ptr %diakok49, align 8
  %field_gep60 = getelementptr inbounds %osztaly, ptr %strct_init48, i32 0, i32 0
  store [3 x ptr] %diakok59, ptr %field_gep60, align 8
  %constructed_struct61 = load %osztaly, ptr %strct_init48, align 8
  %array_idx_val63 = getelementptr [2 x %osztaly], ptr %array_temp_val_var30, i32 0, i32 0
  store %osztaly %constructed_struct45, ptr %array_idx_val63, align 8
  %array_idx_val64 = getelementptr [2 x %osztaly], ptr %array_temp_val_var30, i32 0, i32 1
  store %osztaly %constructed_struct61, ptr %array_idx_val64, align 8
  %array_temp_val_deref65 = load [2 x %osztaly], ptr %array_temp_val_var30, align 8
  %array_temp_val_var66 = alloca [2 x %osztaly], align 8
  %strct_init68 = alloca %osztaly, align 8
  %diakok69 = alloca [3 x ptr], align 8
  %array_idx_val76 = getelementptr [3 x ptr], ptr %diakok69, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val76, align 8
  %array_idx_val77 = getelementptr [3 x ptr], ptr %diakok69, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val77, align 8
  %array_idx_val78 = getelementptr [3 x ptr], ptr %diakok69, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val78, align 8
  %diakok79 = load [3 x ptr], ptr %diakok69, align 8
  %field_gep80 = getelementptr inbounds %osztaly, ptr %strct_init68, i32 0, i32 0
  store [3 x ptr] %diakok79, ptr %field_gep80, align 8
  %constructed_struct81 = load %osztaly, ptr %strct_init68, align 8
  %strct_init84 = alloca %osztaly, align 8
  %diakok85 = alloca [3 x ptr], align 8
  %array_idx_val92 = getelementptr [3 x ptr], ptr %diakok85, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val92, align 8
  %array_idx_val93 = getelementptr [3 x ptr], ptr %diakok85, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val93, align 8
  %array_idx_val94 = getelementptr [3 x ptr], ptr %diakok85, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val94, align 8
  %diakok95 = load [3 x ptr], ptr %diakok85, align 8
  %field_gep96 = getelementptr inbounds %osztaly, ptr %strct_init84, i32 0, i32 0
  store [3 x ptr] %diakok95, ptr %field_gep96, align 8
  %constructed_struct97 = load %osztaly, ptr %strct_init84, align 8
  %array_idx_val99 = getelementptr [2 x %osztaly], ptr %array_temp_val_var66, i32 0, i32 0
  store %osztaly %constructed_struct81, ptr %array_idx_val99, align 8
  %array_idx_val100 = getelementptr [2 x %osztaly], ptr %array_temp_val_var66, i32 0, i32 1
  store %osztaly %constructed_struct97, ptr %array_idx_val100, align 8
  %array_temp_val_deref101 = load [2 x %osztaly], ptr %array_temp_val_var66, align 8
  %array_temp_val_var102 = alloca [2 x %osztaly], align 8
  %strct_init104 = alloca %osztaly, align 8
  %diakok105 = alloca [3 x ptr], align 8
  %array_idx_val112 = getelementptr [3 x ptr], ptr %diakok105, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val112, align 8
  %array_idx_val113 = getelementptr [3 x ptr], ptr %diakok105, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val113, align 8
  %array_idx_val114 = getelementptr [3 x ptr], ptr %diakok105, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val114, align 8
  %diakok115 = load [3 x ptr], ptr %diakok105, align 8
  %field_gep116 = getelementptr inbounds %osztaly, ptr %strct_init104, i32 0, i32 0
  store [3 x ptr] %diakok115, ptr %field_gep116, align 8
  %constructed_struct117 = load %osztaly, ptr %strct_init104, align 8
  %strct_init120 = alloca %osztaly, align 8
  %diakok121 = alloca [3 x ptr], align 8
  %array_idx_val128 = getelementptr [3 x ptr], ptr %diakok121, i32 0, i32 0
  store ptr @marci30, ptr %array_idx_val128, align 8
  %array_idx_val129 = getelementptr [3 x ptr], ptr %diakok121, i32 0, i32 1
  store ptr @marci, ptr %array_idx_val129, align 8
  %array_idx_val130 = getelementptr [3 x ptr], ptr %diakok121, i32 0, i32 2
  store ptr @marci, ptr %array_idx_val130, align 8
  %diakok131 = load [3 x ptr], ptr %diakok121, align 8
  %field_gep132 = getelementptr inbounds %osztaly, ptr %strct_init120, i32 0, i32 0
  store [3 x ptr] %diakok131, ptr %field_gep132, align 8
  %constructed_struct133 = load %osztaly, ptr %strct_init120, align 8
  %array_idx_val135 = getelementptr [2 x %osztaly], ptr %array_temp_val_var102, i32 0, i32 0
  store %osztaly %constructed_struct117, ptr %array_idx_val135, align 8
  %array_idx_val136 = getelementptr [2 x %osztaly], ptr %array_temp_val_var102, i32 0, i32 1
  store %osztaly %constructed_struct133, ptr %array_idx_val136, align 8
  %array_temp_val_deref137 = load [2 x %osztaly], ptr %array_temp_val_var102, align 8
  %array_idx_val138 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 0
  store [2 x %osztaly] %array_temp_val_deref29, ptr %array_idx_val138, align 8
  %array_idx_val139 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 1
  store [2 x %osztaly] %array_temp_val_deref65, ptr %array_idx_val139, align 8
  %array_idx_val140 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 2
  store [2 x %osztaly] %array_temp_val_deref101, ptr %array_idx_val140, align 8
  %array_idx_val141 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 3
  store [2 x %osztaly] %array_temp_val_deref137, ptr %array_idx_val141, align 8
  call void (ptr, ...) @printf(ptr @Hello)
  %array_idx_elem = getelementptr [3 x ptr], ptr %osztalyok, i32 0, i32 0
  %idx_array_val_deref = load ptr, ptr %array_idx_elem, align 8
  call void (ptr, ...) @printf(ptr @"Termeszporkolt: %s", ptr %idx_array_val_deref)
  ret i32 0
}
