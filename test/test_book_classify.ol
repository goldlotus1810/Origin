let tbl = __file_read_bytes("json/udc_p_table.bin");

let books = ["data/cuon_theo_chieu_gio.txt", "data/ml_coban.txt", "data/book_Ban Thuc Su Co Tai! - Tina Seelig.txt"];
let names = ["Cuon Theo Chieu Gio", "ML Co Ban", "Ban Thuc Su Co Tai"];

emit "=== BOOK AUTO-CLASSIFICATION ===";
emit "";

let bi = 0;
while bi < 3 {
  let path = __array_get(books, bi);
  let name = __array_get(names, bi);
  let content = __file_read(path);
  let blen = __len(content);
  let t0 = __time();
  let pw = __text_to_pw(content, tbl);
  let lines = __line_offsets(content);
  let t1 = __time();
  let prlen = __array_len(pw);
  let nuniq = prlen / 2;
  let nlines = __array_len(lines) / 2;

  // Compose fingerprint
  let st = [__array_get(pw, 0)];
  let k = 2;
  while k < prlen {
    let p = __array_get(pw, k);
    let cur = __array_get(st, 0);
    let cs = (__floor(cur/4096))%16;
    let cr = (__floor(cur/256))%16;
    let cv = (__floor(cur/32))%8;
    let ca = (__floor(cur/4))%8;
    let ct = cur%4;
    let ns = (__floor(p/4096))%16;
    let nr = (__floor(p/256))%16;
    let nv = (__floor(p/32))%8;
    let na = (__floor(p/4))%8;
    let nt = p%4;
    let rs = (__floor((cs*2+ns)/3))%16;
    let rr = (__floor((cr*2+nr)/3))%16;
    let rv = (__floor((cv*2+nv)/3))%8;
    let ra = (__floor((ca*2+na)/3))%8;
    let rt = (__floor((ct*2+nt)/3))%4;
    let nw = (rs*4096)+(rr*256)+(rv*32)+(ra*4)+rt;
    let _ = __set_at(st, 0, nw);
    let k = k + 2;
  };
  let mol = __array_get(st, 0);
  let ms = (__floor(mol/4096))%16;
  let mr = (__floor(mol/256))%16;
  let mv = (__floor(mol/32))%8;
  let ma = (__floor(mol/4))%8;
  let mt = mol%4;

  // Auto-classify based on SRVAT
  let category = "unknown";
  // V >= 4 = positive/neutral, V < 3 = negative/sad
  // R >= 5 = complex relations, R < 4 = simple/technical
  // A >= 4 = high energy, A < 2 = calm
  if mr >= 5 { let category = "van-hoc"; };
  if mr <= 4 { let category = "ky-thuat"; };
  if mv >= 4 { let category = "tich-cuc"; };
  if mv <= 2 { let category = "tram-lang"; };

  // More specific: use V+R combined
  let genre = "general";
  if mr >= 6 { let genre = "tieu-thuyet"; };
  if mr <= 3 { let genre = "khoa-hoc"; };
  if mv <= 1 { let genre = "khoa-hoc"; };
  if mr >= 5 { if mv >= 3 { let genre = "tieu-thuyet"; }; };
  if mr >= 5 { if mv <= 2 { let genre = "self-help"; }; };
  if mr == 4 { if mv >= 3 { let genre = "xa-hoi"; }; };
  if mr == 4 { if mv <= 1 { let genre = "khoa-hoc"; }; };

  emit "--- " + name + " ---";
  emit "  Size: " + to_string(blen) + "B, " + to_string(nlines) + " lines, " + to_string(nuniq) + " unique pw";
  emit "  Mol=" + to_string(mol) + " S=" + to_string(ms) + " R=" + to_string(mr) + " V=" + to_string(mv) + " A=" + to_string(ma) + " T=" + to_string(mt);
  emit "  Category: " + category;
  emit "  Genre: " + genre;
  emit "  Time: " + to_string(t1-t0) + "ms";
  emit "";
  let bi = bi + 1;
};

emit "=== CLASSIFICATION DONE ===";
emit "PASS";
