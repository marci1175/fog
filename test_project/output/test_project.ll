; ModuleID = 'main'
source_filename = "main"

%osztaly = type { [3 x ptr] }

@marci30 = constant [8 x i8] c"marci30\00"
@marci = constant [6 x i8] c"marci\00"
@"Hello\0A" = constant [7 x i8] c"Hello\0A\00"
@"Termeszporkolt: %s\0A" = constant [20 x i8] c"Termeszporkolt: %s\0A\00"
@Apad = constant [5 x i8] c"Apad\00"
@Anyad = constant [6 x i8] c"Anyad\00"
@Cicad = constant [6 x i8] c"Cicad\00"
@"Hello: %s\0A" = constant [11 x i8] c"Hello: %s\0A\00"

declare i32 @scanf(ptr, [10 x i8])

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
  %osztalyok = alloca [4 x [2 x %osztaly]], align 8
  %array_temp_val_var = alloca [2 x %osztaly], align 8
  %array_temp_val_var1 = alloca %osztaly, align 8
  %strct_init = alloca %osztaly, align 8
  %diakok = alloca [3 x ptr], align 8
  %array_temp_val_var2 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var2, align 8
  %array_temp_val_deref = load ptr, ptr %array_temp_val_var2, align 8
  %array_temp_val_var3 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var3, align 8
  %array_temp_val_deref4 = load ptr, ptr %array_temp_val_var3, align 8
  %array_temp_val_var5 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var5, align 8
  %array_temp_val_deref6 = load ptr, ptr %array_temp_val_var5, align 8
  %array_idx_val = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 0
  store ptr %array_temp_val_deref, ptr %array_idx_val, align 8
  %array_idx_val7 = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 1
  store ptr %array_temp_val_deref4, ptr %array_idx_val7, align 8
  %array_idx_val8 = getelementptr [3 x ptr], ptr %diakok, i32 0, i32 2
  store ptr %array_temp_val_deref6, ptr %array_idx_val8, align 8
  %diakok9 = load [3 x ptr], ptr %diakok, align 8
  %field_gep = getelementptr inbounds %osztaly, ptr %strct_init, i32 0, i32 0
  store [3 x ptr] %diakok9, ptr %field_gep, align 8
  %constructed_struct = load %osztaly, ptr %strct_init, align 8
  store %osztaly %constructed_struct, ptr %array_temp_val_var1, align 8
  %array_temp_val_deref10 = load %osztaly, ptr %array_temp_val_var1, align 8
  %array_temp_val_var11 = alloca %osztaly, align 8
  %strct_init12 = alloca %osztaly, align 8
  %diakok13 = alloca [3 x ptr], align 8
  %array_temp_val_var14 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var14, align 8
  %array_temp_val_deref15 = load ptr, ptr %array_temp_val_var14, align 8
  %array_temp_val_var16 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var16, align 8
  %array_temp_val_deref17 = load ptr, ptr %array_temp_val_var16, align 8
  %array_temp_val_var18 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var18, align 8
  %array_temp_val_deref19 = load ptr, ptr %array_temp_val_var18, align 8
  %array_idx_val20 = getelementptr [3 x ptr], ptr %diakok13, i32 0, i32 0
  store ptr %array_temp_val_deref15, ptr %array_idx_val20, align 8
  %array_idx_val21 = getelementptr [3 x ptr], ptr %diakok13, i32 0, i32 1
  store ptr %array_temp_val_deref17, ptr %array_idx_val21, align 8
  %array_idx_val22 = getelementptr [3 x ptr], ptr %diakok13, i32 0, i32 2
  store ptr %array_temp_val_deref19, ptr %array_idx_val22, align 8
  %diakok23 = load [3 x ptr], ptr %diakok13, align 8
  %field_gep24 = getelementptr inbounds %osztaly, ptr %strct_init12, i32 0, i32 0
  store [3 x ptr] %diakok23, ptr %field_gep24, align 8
  %constructed_struct25 = load %osztaly, ptr %strct_init12, align 8
  store %osztaly %constructed_struct25, ptr %array_temp_val_var11, align 8
  %array_temp_val_deref26 = load %osztaly, ptr %array_temp_val_var11, align 8
  %array_idx_val27 = getelementptr [2 x %osztaly], ptr %array_temp_val_var, i32 0, i32 0
  store %osztaly %array_temp_val_deref10, ptr %array_idx_val27, align 8
  %array_idx_val28 = getelementptr [2 x %osztaly], ptr %array_temp_val_var, i32 0, i32 1
  store %osztaly %array_temp_val_deref26, ptr %array_idx_val28, align 8
  %array_temp_val_deref29 = load [2 x %osztaly], ptr %array_temp_val_var, align 8
  %array_temp_val_var30 = alloca [2 x %osztaly], align 8
  %array_temp_val_var31 = alloca %osztaly, align 8
  %strct_init32 = alloca %osztaly, align 8
  %diakok33 = alloca [3 x ptr], align 8
  %array_temp_val_var34 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var34, align 8
  %array_temp_val_deref35 = load ptr, ptr %array_temp_val_var34, align 8
  %array_temp_val_var36 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var36, align 8
  %array_temp_val_deref37 = load ptr, ptr %array_temp_val_var36, align 8
  %array_temp_val_var38 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var38, align 8
  %array_temp_val_deref39 = load ptr, ptr %array_temp_val_var38, align 8
  %array_idx_val40 = getelementptr [3 x ptr], ptr %diakok33, i32 0, i32 0
  store ptr %array_temp_val_deref35, ptr %array_idx_val40, align 8
  %array_idx_val41 = getelementptr [3 x ptr], ptr %diakok33, i32 0, i32 1
  store ptr %array_temp_val_deref37, ptr %array_idx_val41, align 8
  %array_idx_val42 = getelementptr [3 x ptr], ptr %diakok33, i32 0, i32 2
  store ptr %array_temp_val_deref39, ptr %array_idx_val42, align 8
  %diakok43 = load [3 x ptr], ptr %diakok33, align 8
  %field_gep44 = getelementptr inbounds %osztaly, ptr %strct_init32, i32 0, i32 0
  store [3 x ptr] %diakok43, ptr %field_gep44, align 8
  %constructed_struct45 = load %osztaly, ptr %strct_init32, align 8
  store %osztaly %constructed_struct45, ptr %array_temp_val_var31, align 8
  %array_temp_val_deref46 = load %osztaly, ptr %array_temp_val_var31, align 8
  %array_temp_val_var47 = alloca %osztaly, align 8
  %strct_init48 = alloca %osztaly, align 8
  %diakok49 = alloca [3 x ptr], align 8
  %array_temp_val_var50 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var50, align 8
  %array_temp_val_deref51 = load ptr, ptr %array_temp_val_var50, align 8
  %array_temp_val_var52 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var52, align 8
  %array_temp_val_deref53 = load ptr, ptr %array_temp_val_var52, align 8
  %array_temp_val_var54 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var54, align 8
  %array_temp_val_deref55 = load ptr, ptr %array_temp_val_var54, align 8
  %array_idx_val56 = getelementptr [3 x ptr], ptr %diakok49, i32 0, i32 0
  store ptr %array_temp_val_deref51, ptr %array_idx_val56, align 8
  %array_idx_val57 = getelementptr [3 x ptr], ptr %diakok49, i32 0, i32 1
  store ptr %array_temp_val_deref53, ptr %array_idx_val57, align 8
  %array_idx_val58 = getelementptr [3 x ptr], ptr %diakok49, i32 0, i32 2
  store ptr %array_temp_val_deref55, ptr %array_idx_val58, align 8
  %diakok59 = load [3 x ptr], ptr %diakok49, align 8
  %field_gep60 = getelementptr inbounds %osztaly, ptr %strct_init48, i32 0, i32 0
  store [3 x ptr] %diakok59, ptr %field_gep60, align 8
  %constructed_struct61 = load %osztaly, ptr %strct_init48, align 8
  store %osztaly %constructed_struct61, ptr %array_temp_val_var47, align 8
  %array_temp_val_deref62 = load %osztaly, ptr %array_temp_val_var47, align 8
  %array_idx_val63 = getelementptr [2 x %osztaly], ptr %array_temp_val_var30, i32 0, i32 0
  store %osztaly %array_temp_val_deref46, ptr %array_idx_val63, align 8
  %array_idx_val64 = getelementptr [2 x %osztaly], ptr %array_temp_val_var30, i32 0, i32 1
  store %osztaly %array_temp_val_deref62, ptr %array_idx_val64, align 8
  %array_temp_val_deref65 = load [2 x %osztaly], ptr %array_temp_val_var30, align 8
  %array_temp_val_var66 = alloca [2 x %osztaly], align 8
  %array_temp_val_var67 = alloca %osztaly, align 8
  %strct_init68 = alloca %osztaly, align 8
  %diakok69 = alloca [3 x ptr], align 8
  %array_temp_val_var70 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var70, align 8
  %array_temp_val_deref71 = load ptr, ptr %array_temp_val_var70, align 8
  %array_temp_val_var72 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var72, align 8
  %array_temp_val_deref73 = load ptr, ptr %array_temp_val_var72, align 8
  %array_temp_val_var74 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var74, align 8
  %array_temp_val_deref75 = load ptr, ptr %array_temp_val_var74, align 8
  %array_idx_val76 = getelementptr [3 x ptr], ptr %diakok69, i32 0, i32 0
  store ptr %array_temp_val_deref71, ptr %array_idx_val76, align 8
  %array_idx_val77 = getelementptr [3 x ptr], ptr %diakok69, i32 0, i32 1
  store ptr %array_temp_val_deref73, ptr %array_idx_val77, align 8
  %array_idx_val78 = getelementptr [3 x ptr], ptr %diakok69, i32 0, i32 2
  store ptr %array_temp_val_deref75, ptr %array_idx_val78, align 8
  %diakok79 = load [3 x ptr], ptr %diakok69, align 8
  %field_gep80 = getelementptr inbounds %osztaly, ptr %strct_init68, i32 0, i32 0
  store [3 x ptr] %diakok79, ptr %field_gep80, align 8
  %constructed_struct81 = load %osztaly, ptr %strct_init68, align 8
  store %osztaly %constructed_struct81, ptr %array_temp_val_var67, align 8
  %array_temp_val_deref82 = load %osztaly, ptr %array_temp_val_var67, align 8
  %array_temp_val_var83 = alloca %osztaly, align 8
  %strct_init84 = alloca %osztaly, align 8
  %diakok85 = alloca [3 x ptr], align 8
  %array_temp_val_var86 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var86, align 8
  %array_temp_val_deref87 = load ptr, ptr %array_temp_val_var86, align 8
  %array_temp_val_var88 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var88, align 8
  %array_temp_val_deref89 = load ptr, ptr %array_temp_val_var88, align 8
  %array_temp_val_var90 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var90, align 8
  %array_temp_val_deref91 = load ptr, ptr %array_temp_val_var90, align 8
  %array_idx_val92 = getelementptr [3 x ptr], ptr %diakok85, i32 0, i32 0
  store ptr %array_temp_val_deref87, ptr %array_idx_val92, align 8
  %array_idx_val93 = getelementptr [3 x ptr], ptr %diakok85, i32 0, i32 1
  store ptr %array_temp_val_deref89, ptr %array_idx_val93, align 8
  %array_idx_val94 = getelementptr [3 x ptr], ptr %diakok85, i32 0, i32 2
  store ptr %array_temp_val_deref91, ptr %array_idx_val94, align 8
  %diakok95 = load [3 x ptr], ptr %diakok85, align 8
  %field_gep96 = getelementptr inbounds %osztaly, ptr %strct_init84, i32 0, i32 0
  store [3 x ptr] %diakok95, ptr %field_gep96, align 8
  %constructed_struct97 = load %osztaly, ptr %strct_init84, align 8
  store %osztaly %constructed_struct97, ptr %array_temp_val_var83, align 8
  %array_temp_val_deref98 = load %osztaly, ptr %array_temp_val_var83, align 8
  %array_idx_val99 = getelementptr [2 x %osztaly], ptr %array_temp_val_var66, i32 0, i32 0
  store %osztaly %array_temp_val_deref82, ptr %array_idx_val99, align 8
  %array_idx_val100 = getelementptr [2 x %osztaly], ptr %array_temp_val_var66, i32 0, i32 1
  store %osztaly %array_temp_val_deref98, ptr %array_idx_val100, align 8
  %array_temp_val_deref101 = load [2 x %osztaly], ptr %array_temp_val_var66, align 8
  %array_temp_val_var102 = alloca [2 x %osztaly], align 8
  %array_temp_val_var103 = alloca %osztaly, align 8
  %strct_init104 = alloca %osztaly, align 8
  %diakok105 = alloca [3 x ptr], align 8
  %array_temp_val_var106 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var106, align 8
  %array_temp_val_deref107 = load ptr, ptr %array_temp_val_var106, align 8
  %array_temp_val_var108 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var108, align 8
  %array_temp_val_deref109 = load ptr, ptr %array_temp_val_var108, align 8
  %array_temp_val_var110 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var110, align 8
  %array_temp_val_deref111 = load ptr, ptr %array_temp_val_var110, align 8
  %array_idx_val112 = getelementptr [3 x ptr], ptr %diakok105, i32 0, i32 0
  store ptr %array_temp_val_deref107, ptr %array_idx_val112, align 8
  %array_idx_val113 = getelementptr [3 x ptr], ptr %diakok105, i32 0, i32 1
  store ptr %array_temp_val_deref109, ptr %array_idx_val113, align 8
  %array_idx_val114 = getelementptr [3 x ptr], ptr %diakok105, i32 0, i32 2
  store ptr %array_temp_val_deref111, ptr %array_idx_val114, align 8
  %diakok115 = load [3 x ptr], ptr %diakok105, align 8
  %field_gep116 = getelementptr inbounds %osztaly, ptr %strct_init104, i32 0, i32 0
  store [3 x ptr] %diakok115, ptr %field_gep116, align 8
  %constructed_struct117 = load %osztaly, ptr %strct_init104, align 8
  store %osztaly %constructed_struct117, ptr %array_temp_val_var103, align 8
  %array_temp_val_deref118 = load %osztaly, ptr %array_temp_val_var103, align 8
  %array_temp_val_var119 = alloca %osztaly, align 8
  %strct_init120 = alloca %osztaly, align 8
  %diakok121 = alloca [3 x ptr], align 8
  %array_temp_val_var122 = alloca ptr, align 8
  store ptr @marci30, ptr %array_temp_val_var122, align 8
  %array_temp_val_deref123 = load ptr, ptr %array_temp_val_var122, align 8
  %array_temp_val_var124 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var124, align 8
  %array_temp_val_deref125 = load ptr, ptr %array_temp_val_var124, align 8
  %array_temp_val_var126 = alloca ptr, align 8
  store ptr @marci, ptr %array_temp_val_var126, align 8
  %array_temp_val_deref127 = load ptr, ptr %array_temp_val_var126, align 8
  %array_idx_val128 = getelementptr [3 x ptr], ptr %diakok121, i32 0, i32 0
  store ptr %array_temp_val_deref123, ptr %array_idx_val128, align 8
  %array_idx_val129 = getelementptr [3 x ptr], ptr %diakok121, i32 0, i32 1
  store ptr %array_temp_val_deref125, ptr %array_idx_val129, align 8
  %array_idx_val130 = getelementptr [3 x ptr], ptr %diakok121, i32 0, i32 2
  store ptr %array_temp_val_deref127, ptr %array_idx_val130, align 8
  %diakok131 = load [3 x ptr], ptr %diakok121, align 8
  %field_gep132 = getelementptr inbounds %osztaly, ptr %strct_init120, i32 0, i32 0
  store [3 x ptr] %diakok131, ptr %field_gep132, align 8
  %constructed_struct133 = load %osztaly, ptr %strct_init120, align 8
  store %osztaly %constructed_struct133, ptr %array_temp_val_var119, align 8
  %array_temp_val_deref134 = load %osztaly, ptr %array_temp_val_var119, align 8
  %array_idx_val135 = getelementptr [2 x %osztaly], ptr %array_temp_val_var102, i32 0, i32 0
  store %osztaly %array_temp_val_deref118, ptr %array_idx_val135, align 8
  %array_idx_val136 = getelementptr [2 x %osztaly], ptr %array_temp_val_var102, i32 0, i32 1
  store %osztaly %array_temp_val_deref134, ptr %array_idx_val136, align 8
  %array_temp_val_deref137 = load [2 x %osztaly], ptr %array_temp_val_var102, align 8
  %array_idx_val138 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 0
  store [2 x %osztaly] %array_temp_val_deref29, ptr %array_idx_val138, align 8
  %array_idx_val139 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 1
  store [2 x %osztaly] %array_temp_val_deref65, ptr %array_idx_val139, align 8
  %array_idx_val140 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 2
  store [2 x %osztaly] %array_temp_val_deref101, ptr %array_idx_val140, align 8
  %array_idx_val141 = getelementptr [4 x [2 x %osztaly]], ptr %osztalyok, i32 0, i32 3
  store [2 x %osztaly] %array_temp_val_deref137, ptr %array_idx_val141, align 8
  %iq = alloca i32, align 4
  store i32 2000, ptr %iq, align 4
  %input = alloca ptr, align 8
  store ptr @"Hello\0A", ptr %input, align 8
  %input142 = load ptr, ptr %input, align 8
  call void (ptr, ...) @printf(ptr %input142)
  %input143 = alloca ptr, align 8
  store ptr @"Termeszporkolt: %s\0A", ptr %input143, align 8
  %input144 = load ptr, ptr %input143, align 8
  %printf_idx_1_arg = alloca ptr, align 8
  %0 = alloca i32, align 4
  store i32 0, ptr %0, align 4
  %array_idx_val145 = load i32, ptr %0, align 4
  %array_idx_elem = getelementptr [3 x ptr], ptr %osztalyok, i32 0, i32 %array_idx_val145
  %idx_array_val_deref = load ptr, ptr %array_idx_elem, align 8
  store ptr %idx_array_val_deref, ptr %printf_idx_1_arg, align 8
  %printf_idx_1_arg146 = load ptr, ptr %printf_idx_1_arg, align 8
  call void (ptr, ...) @printf(ptr %input144, ptr %printf_idx_1_arg146)
  %lhs_tmp = alloca i32, align 4
  %rhs_tmp = alloca i32, align 4
  store i32 2000, ptr %rhs_tmp, align 4
  %var_deref = load i32, ptr %iq, align 4
  store i32 %var_deref, ptr %lhs_tmp, align 4
  %lhs_tmp_val = load i32, ptr %lhs_tmp, align 4
  %rhs_tmp_val = load i32, ptr %rhs_tmp, align 4
  %cmp = icmp eq i32 %lhs_tmp_val, %rhs_tmp_val
  %cmp_result = alloca i1, align 1
  store i1 %cmp, ptr %cmp_result, align 1
  %condition = load i1, ptr %cmp_result, align 1
  br i1 %condition, label %cond_branch_true, label %cond_branch_false

cond_branch_true:                                 ; preds = %main_fn_entry
  %haram = alloca %osztaly, align 8
  %strct_init147 = alloca %osztaly, align 8
  %diakok148 = alloca [3 x ptr], align 8
  %array_temp_val_var149 = alloca ptr, align 8
  store ptr @Apad, ptr %array_temp_val_var149, align 8
  %array_temp_val_deref150 = load ptr, ptr %array_temp_val_var149, align 8
  %array_temp_val_var151 = alloca ptr, align 8
  store ptr @Anyad, ptr %array_temp_val_var151, align 8
  %array_temp_val_deref152 = load ptr, ptr %array_temp_val_var151, align 8
  %array_temp_val_var153 = alloca ptr, align 8
  store ptr @Cicad, ptr %array_temp_val_var153, align 8
  %array_temp_val_deref154 = load ptr, ptr %array_temp_val_var153, align 8
  %array_idx_val155 = getelementptr [3 x ptr], ptr %diakok148, i32 0, i32 0
  store ptr %array_temp_val_deref150, ptr %array_idx_val155, align 8
  %array_idx_val156 = getelementptr [3 x ptr], ptr %diakok148, i32 0, i32 1
  store ptr %array_temp_val_deref152, ptr %array_idx_val156, align 8
  %array_idx_val157 = getelementptr [3 x ptr], ptr %diakok148, i32 0, i32 2
  store ptr %array_temp_val_deref154, ptr %array_idx_val157, align 8
  %diakok158 = load [3 x ptr], ptr %diakok148, align 8
  %field_gep159 = getelementptr inbounds %osztaly, ptr %strct_init147, i32 0, i32 0
  store [3 x ptr] %diakok158, ptr %field_gep159, align 8
  %constructed_struct160 = load %osztaly, ptr %strct_init147, align 8
  store %osztaly %constructed_struct160, ptr %haram, align 8
  %input161 = alloca ptr, align 8
  store ptr @"Hello: %s\0A", ptr %input161, align 8
  %input162 = load ptr, ptr %input161, align 8
  %printf_idx_1_arg163 = alloca ptr, align 8
  %1 = alloca i32, align 4
  store i32 2, ptr %1, align 4
  %array_idx_val164 = load i32, ptr %1, align 4
  %array_idx_elem165 = getelementptr [3 x ptr], ptr %haram, i32 0, i32 %array_idx_val164
  %idx_array_val_deref166 = load ptr, ptr %array_idx_elem165, align 8
  store ptr %idx_array_val_deref166, ptr %printf_idx_1_arg163, align 8
  %printf_idx_1_arg167 = load ptr, ptr %printf_idx_1_arg163, align 8
  call void (ptr, ...) @printf(ptr %input162, ptr %printf_idx_1_arg167)
  br label %cond_branch_uncond

cond_branch_false:                                ; preds = %main_fn_entry
  br label %cond_branch_uncond

cond_branch_uncond:                               ; preds = %cond_branch_false, %cond_branch_true
  %ret_tmp_var = alloca i32, align 4
  store i32 0, ptr %ret_tmp_var, align 4
  %ret_tmp_var168 = load i32, ptr %ret_tmp_var, align 4
  ret i32 %ret_tmp_var168
}
