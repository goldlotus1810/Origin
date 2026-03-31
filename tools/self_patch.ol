// Olang patches itself: reverse search optimization
// Finds var_load_hash linear scan → patches to reverse search

let vm = __file_read_bytes("origin_new.olang");
let vm_len = __bytes_len(vm);
emit "VM: " + to_string(vm_len) + " bytes";

// Find pattern: var_load_hash forward search
// .vlh_search: cmp (%rdi), %rax = 48 39 07
// .vlh_next:   add $24, %rdi    = 48 83 C7 18
// Together: 48 39 07 ... 48 83 C7 18
// The "add $24, %rdi" is unique to var_load/store linear scan

let found = [];
let i = 4096;
while i < vm_len - 8 {
    let b0 = __bytes_get(vm, i);
    let b1 = __bytes_get(vm, i + 1);
    let b2 = __bytes_get(vm, i + 2);
    let b3 = __bytes_get(vm, i + 3);
    // add $24, %rdi = 48 83 C7 18
    let match = 0;
    if b0 == 0x48 { if b1 == 0x83 { if b2 == 0xC7 { if b3 == 0x18 { let match = 1; }; }; }; };
    if match == 1 { push(found, i); emit "add $24,%rdi at " + to_string(i); };
    let i = i + 1;
};
emit "Found " + to_string(len(found)) + " occurrences of add $24,%rdi";

// Also find sub $24, %rdi = 48 83 EF 18 (should be 0 — reverse not present yet)
let rev = [];
let j = 4096;
while j < vm_len - 8 {
    let b0 = __bytes_get(vm, j);
    let b1 = __bytes_get(vm, j + 1);
    let b2 = __bytes_get(vm, j + 2);
    let b3 = __bytes_get(vm, j + 3);
    let match = 0;
    if b0 == 0x48 { if b1 == 0x83 { if b2 == 0xEF { if b3 == 0x18 { let match = 1; }; }; }; };
    if match == 1 { push(rev, j); emit "sub $24,%rdi at " + to_string(j); };
    let j = j + 1;
};
emit "Found " + to_string(len(rev)) + " occurrences of sub $24,%rdi";
emit "DONE";
