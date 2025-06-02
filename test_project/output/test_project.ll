; ModuleID = 'main'
source_filename = "main"

%asd = type { float }

declare i32 @puts(ptr)

define float @test() {
main_fn_entry:
  ret float 0x402475C280000000
}

define i32 @main() {
main_fn_entry:
  %a = alloca %asd, align 8
  %strct_init = alloca %asd, align 8
  %field_gep = getelementptr inbounds %asd, ptr %strct_init, i32 0, i32 0
  store float 0x4037333340000000, ptr %field_gep, align 4
  %constructed_struct = load %asd, ptr %strct_init, align 4
  store %asd %constructed_struct, ptr %a, align 4
  %deref_strct_val = load float, ptr %a, align 4
  %cmp = fcmp oeq float %deref_strct_val, 0x4037333340000000
  ret i32 1
}
