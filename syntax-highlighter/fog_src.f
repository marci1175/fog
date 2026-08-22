#-> 
asd
#->
@nofree
function main(): int {
    array<int, 4> marci = {1, 2, 3, 4};
    ptr egy_ptr = ref marci[0];
    int egy = deref egy_ptr;
    return egy;
}