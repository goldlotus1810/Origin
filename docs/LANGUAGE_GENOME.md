# LANGUAGE GENOME — Bản Đồ Gene Của Mọi Ngôn Ngữ Lập Trình

> **Mục đích:** Hiểu mọi ngôn ngữ làm được gì, TẠI SAO, và cần gì ở tầng ASM.
> **Không so sánh với Olang.** Chỉ phân tích sự thật.
> **Cuối cùng:** tổng hợp silk → ASM requirements → Olang gaps.

---

# PHẦN 1: MỌI NGÔN NGỮ VÀ THẾ MẠNH

---

## 1.1 Systems Programming — Kiểm soát phần cứng

### C (1972, Dennis Ritchie)
```
Thế mạnh:
  - Truy cập memory trực tiếp (pointer arithmetic)
  - Gọi syscall trực tiếp (Unix được viết bằng C)
  - Biên dịch thành native code — nhanh nhất
  - Chạy mọi nơi có CPU (từ microcontroller 8-bit đến supercomputer)
  - Footprint cực nhỏ (embedded systems, kernel)
  
Tại sao:
  C = abstraction layer MỎNG NHẤT trên ASM.
  int *p = (int*)0x1000; *p = 42;  → MOV [0x1000], 42
  Gần như 1:1 với machine code.
```

### C++ (1979, Bjarne Stroustrup)
```
Thế mạnh:
  - Mọi thứ C có + OOP + templates + RAII
  - Game engines (Unreal, Unity core), browsers (Chrome), OS (Windows)
  - Zero-cost abstractions — class/template compile thành code tối ưu
  - Deterministic destruction (destructor gọi khi object ra khỏi scope)

Tại sao:
  C++ = C + compile-time code generation.
  Template = macro thông minh → compiler sinh code tối ưu cho mỗi type.
  RAII = compiler tự chèn destructor call → memory safety không cần GC.
```

### Rust (2010, Graydon Hoare / Mozilla)
```
Thế mạnh:
  - Memory safety KHÔNG CẦN GC (ownership + borrow checker)
  - Concurrency safety tại compile time (no data race)
  - Performance ngang C/C++
  - Modern tooling (cargo, crates.io)
  - Dùng cho: Firefox, Linux kernel modules, crypto, blockchain

Tại sao:
  Rust = C performance + compile-time proofs.
  Compiler CHỨNG MINH code không có memory bug TRƯỚC KHI chạy.
  Borrow checker = hệ thống type tracking ai đang "sở hữu" memory.
```

### Zig (2015, Andrew Kelley)
```
Thế mạnh:
  - C interop hoàn hảo (import .h files trực tiếp)
  - Comptime — chạy code tại compile time
  - Không có hidden control flow (no exceptions, no operator overloading)
  - Cross-compile tích hợp (build cho mọi platform từ 1 máy)
  - Nhỏ, đơn giản, self-contained

Tại sao:
  Zig = C modernized. Bỏ mọi thứ "ẩn" trong C (undefined behavior, macros).
  Comptime = giống template C++ nhưng dùng CÙNG syntax với runtime code.
```

---

## 1.2 Application Programming — Xây ứng dụng

### Go (2009, Thompson/Pike/Griesemer — Google)
```
Thế mạnh:
  - Compile CỰC NHANH (toàn bộ Go compiler < 10 giây)
  - Goroutines — concurrent nhẹ (1 triệu goroutines trên 1 máy)
  - GC nhanh (< 1ms pause)
  - Static binary (1 file, deploy anywhere)
  - Networking built-in (net/http trong stdlib)
  - Docker, Kubernetes, Terraform đều viết bằng Go

Tại sao:
  Go = C + goroutines + GC + fast compile.
  Goroutine = user-space thread (~2KB stack, OS thread = ~1MB).
  Go scheduler multiplexes goroutines trên OS threads → concurrent không cần async/await.
  Compile nhanh vì: no templates, no macros, dependency graph đơn giản.
```

### Java (1995, James Gosling — Sun)
```
Thế mạnh:
  - "Write once, run anywhere" (JVM = máy ảo chạy mọi OS)
  - Enterprise ecosystem khổng lồ (Spring, Hibernate, Maven)
  - GC trưởng thành nhất (30 năm R&D, ZGC < 1ms)
  - Strongly typed — IDE support cực mạnh
  - Android apps (Kotlin tương thích JVM)

Tại sao:
  JVM = CPU ảo. Java bytecode = ASM ảo.
  JIT compiler tối ưu bytecode → native code TẠI RUNTIME.
  → Có thể nhanh hơn C trong một số case (JIT biết runtime behavior).
```

### C# (2000, Anders Hejlsberg — Microsoft)
```
Thế mạnh:
  - .NET ecosystem (Windows, Azure, Unity game engine)
  - LINQ — query data như SQL trong code
  - Async/await native (inspired Go, JavaScript, Python)
  - Cross-platform via .NET Core

Tại sao:
  CLR (Common Language Runtime) = JVM version Microsoft.
  Cùng pattern: bytecode → JIT → native code.
```

### Swift (2014, Chris Lattner — Apple)
```
Thế mạnh:
  - iOS/macOS native (AppStore = Swift)
  - ARC (Automatic Reference Counting) — deterministic memory
  - Protocol-oriented programming
  - Interop với Objective-C

Tại sao:
  ARC = compiler chèn retain/release tự động.
  Không cần GC → deterministic → tốt cho mobile (battery, latency).
```

### Kotlin (2011, JetBrains)
```
Thế mạnh:
  - Java nhưng gọn hơn 40% LOC
  - Null safety tại compile time
  - Coroutines (concurrent nhẹ)
  - Android official language
  - Multiplatform (JVM, JS, Native)

Tại sao:
  Chạy trên JVM → mọi Java library hoạt động.
  Compiler thông minh hơn Java → null checks, type inference.
```

---

## 1.3 Scripting & Dynamic — Viết nhanh, chạy linh hoạt

### Python (1991, Guido van Rossum)
```
Thế mạnh:
  - Đọc như tiếng Anh (cú pháp đơn giản nhất)
  - 400,000+ packages (PyPI)
  - AI/ML standard (PyTorch, TensorFlow, NumPy, Pandas)
  - Scripting, automation, data science, web
  - REPL — thử code ngay

Tại sao:
  CPython = interpreter viết bằng C. Mỗi Python object = C struct trên heap.
  Dynamic typing = mỗi object MANG TYPE theo nó (runtime check).
  GIL (Global Interpreter Lock) = 1 thread Python chạy 1 lúc → đơn giản nhưng chậm.
  Nhanh nhờ C extensions (NumPy = C/Fortran, PyTorch = C++/CUDA).
```

### JavaScript (1995, Brendan Eich — Netscape)
```
Thế mạnh:
  - TRÌNH DUYỆT. Mọi browser chạy JS. Đây là độc quyền tuyệt đối.
  - V8 engine (JIT → nhanh gần native)
  - Node.js → backend
  - npm = package registry lớn nhất thế giới (2M+ packages)
  - Async event loop — concurrent không cần threads

Tại sao:
  V8 JIT: JS code → Hidden Classes → Inline Caches → machine code.
  Event loop = 1 thread + callback queue → non-blocking I/O.
  Prototype-based OOP = object tạo object, không cần class.
```

### TypeScript (2012, Anders Hejlsberg — Microsoft)
```
Thế mạnh:
  - JavaScript + types tại compile time
  - IDE intelligence (autocomplete, refactor)
  - Gradual typing — dần dần thêm types

Tại sao:
  TypeScript compiler → JavaScript. Types BỊ XÓA khi compile.
  Types chỉ tồn tại lúc development → giúp developer, không chậm runtime.
```

### Ruby (1995, Yukihiro Matsumoto)
```
Thế mạnh:
  - "Developer happiness" — cú pháp đẹp, expressiveness cao
  - Rails framework → web development nhanh nhất
  - Metaprogramming cực mạnh (define_method, method_missing)

Tại sao:
  Mọi thứ là object (kể cả số, true, false).
  Open classes — sửa class ĐÃ TỒN TẠI tại runtime.
```

### PHP (1994, Rasmus Lerdorf)
```
Thế mạnh:
  - 77% websites dùng PHP (WordPress, Facebook ban đầu)
  - Deploy đơn giản (copy .php file lên server)
  - Shared hosting rẻ

Tại sao:
  PHP = template engine thành ngôn ngữ.
  Apache/nginx thấy .php → gọi PHP interpreter → output HTML.
```

### Lua (1993, PUC-Rio Brazil)
```
Thế mạnh:
  - Embed vào C/C++ cực dễ (300KB runtime)
  - Game scripting (WoW, Roblox, LÖVE)
  - LuaJIT = JIT compiler nhanh nhất cho dynamic language

Tại sao:
  Lua VM = 35 opcodes, ~15,000 LOC C. Nhỏ → embed được mọi nơi.
  LuaJIT trace compiler: detect hot loops → compile thành machine code.
```

### Perl (1987, Larry Wall)
```
Thế mạnh:
  - Regex engine mạnh nhất
  - Text processing, bioinformatics
  - "Swiss army knife" cho sysadmin

Tại sao:
  Perl regex engine = NFA + backtracking + lookahead/behind.
  Tích hợp regex VÀO CÚ PHÁP (if $str =~ /pattern/).
```

---

## 1.4 Functional Programming — Toán học thuần

### Haskell (1990, academic committee)
```
Thế mạnh:
  - Pure functions — không side effects
  - Type system mạnh nhất (monads, type classes, GADTs)
  - Lazy evaluation — chỉ tính khi cần
  - Compiler chứng minh code đúng

Tại sao:
  Lazy evaluation: thunks (chưa tính) thay values.
  Monads: wrap side effects trong type system → compiler track I/O.
  GHC optimizer: deforestation, fusion → loại bỏ intermediate data structures.
```

### OCaml (1996, INRIA France)
```
Thế mạnh:
  - ML type inference — không cần khai báo type, compiler suy ra
  - Pattern matching cực mạnh
  - Nhanh (gần C)
  - Compiler phát triển (Rust, Flow, Reason đều ảnh hưởng bởi OCaml)

Tại sao:
  Hindley-Milner type inference: compiler suy type từ cách dùng.
  Algebraic Data Types: sum types + product types = model mọi cấu trúc.
```

### Erlang/Elixir (1986, Joe Armstrong — Ericsson)
```
Thế mạnh:
  - Fault tolerance ("let it crash" philosophy)
  - Hot code reload — update code KHÔNG cần restart
  - 1 triệu lightweight processes trên 1 máy
  - Telecom grade (99.9999999% uptime)

Tại sao:
  BEAM VM: mỗi process = isolated heap, crash 1 process không ảnh hưởng khác.
  Preemptive scheduling: VM cắt process sau N reductions → fair scheduling.
  Hot reload: load module mới → process mới dùng code mới, process cũ finish code cũ.
```

### Lisp/Clojure (1958/2007)
```
Thế mạnh:
  - Code IS data (homoiconicity) — program = list, list = program
  - Macro system mạnh nhất — code sinh code
  - REPL-driven development
  - AI research gốc (McCarthy, MIT)

Tại sao:
  S-expressions: (fn arg1 arg2) = cả cú pháp LẪN data structure.
  Macros: nhận AST → trả AST → compiler dùng → code sinh code tại compile time.
```

---

## 1.5 Scientific & Mathematical — Tính toán khoa học

### Julia (2012, MIT)
```
Thế mạnh:
  - Nhanh như C, dễ như Python
  - Multiple dispatch — function chọn implementation theo TẤT CẢ argument types
  - Built-in matrix, complex numbers, big integers
  - Scientific computing, differential equations

Tại sao:
  JIT via LLVM: Julia code → LLVM IR → machine code.
  Multiple dispatch: compiler specialize function cho mỗi tổ hợp types.
  → Không cần template (C++) hay generics (Rust) — dispatch tại runtime nhưng JIT tối ưu.
```

### R (1993, Ihaka/Gentleman)
```
Thế mạnh:
  - Thống kê và visualization tốt nhất
  - 20,000+ packages (CRAN)
  - Data frames, ggplot2

Tại sao:
  R = domain-specific cho thống kê.
  Vectorized operations: mean(x) chạy trên C loop, không phải R loop.
```

### MATLAB (1984, MathWorks)
```
Thế mạnh:
  - Matrix operations native (tên = MATrix LABoratory)
  - Simulink — visual simulation
  - Engineering standard (aerospace, automotive)

Tại sao:
  LAPACK/BLAS backend: matrix ops → optimized Fortran → hardware SIMD.
```

### Fortran (1957, IBM)
```
Thế mạnh:
  - Numerical computing nhanh nhất (compiler tối ưu array loops)
  - Vẫn dùng trong physics simulation, weather forecasting
  - 67 năm tuổi, vẫn sống

Tại sao:
  No pointer aliasing (by default) → compiler CAN vectorize aggressively.
  Column-major arrays → cache-friendly cho matrix operations.
```

---

## 1.6 Web & Network — Internet

### HTML/CSS (1993/1996)
```
Thế mạnh:
  - Không phải ngôn ngữ lập trình — là markup + styling
  - MỌI trang web trên thế giới
  - Declarative — nói CÁI GÌ, không nói LÀM SAO

Tại sao:
  Browser DOM engine: parse HTML → build tree → CSS layout → paint pixels.
  Không cần logic. Chỉ cần cấu trúc + trình bày.
```

### SQL (1970, IBM)
```
Thế mạnh:
  - Query data từ database
  - Declarative — nói CẦN GÌ, database tự tìm cách
  - Mọi database hiểu SQL (MySQL, PostgreSQL, SQLite, Oracle)

Tại sao:
  Query optimizer: SQL → execution plan → index scan / table scan / join.
  Declarative = database có quyền CHỌN cách tối ưu nhất.
```

---

## 1.7 Low-Level — Sát phần cứng

### Assembly (ASM)
```
Thế mạnh:
  - 1:1 với machine code
  - Kiểm soát TUYỆT ĐỐI: mỗi register, mỗi byte, mỗi cycle
  - Bootloaders, OS kernels, firmware, crypto primitives

Tại sao:
  ASM = human-readable machine code. Không abstraction.
  MOV RAX, 42 → CPU đặt 42 vào register RAX. Hết. Không gì ẩn.
```

### WebAssembly (WASM, 2017)
```
Thế mạnh:
  - Chạy trong browser gần native speed
  - Compile từ C/C++/Rust/Go → WASM → browser chạy
  - Sandbox an toàn (không truy cập filesystem, network trực tiếp)
  - Portable — chạy mọi browser, mọi OS

Tại sao:
  WASM = ASM cho browser. Stack machine, typed, validated trước khi chạy.
  Browser JIT WASM → machine code. Nhanh hơn JavaScript nhiều lần.
```

---

## 1.8 Emerging & Specialized

### Mojo (2023, Chris Lattner)
```
Thế mạnh:
  - Python syntax + C performance
  - AI/ML native (thay thế Python + C extensions)
  - SIMD, GPU programming tích hợp

Tại sao:
  MLIR backend (tạo bởi Lattner) → compile Python-like code thành GPU code.
```

### Solidity (2015, Ethereum)
```
Thế mạnh:
  - Smart contracts trên blockchain
  - Immutable code — deploy rồi không sửa được

Tại sao:
  EVM (Ethereum VM) = stack machine chạy trên mọi node.
  Gas system = mỗi opcode có cost → prevent infinite loops.
```

### Prolog (1972)
```
Thế mạnh:
  - Logic programming — khai báo FACTS + RULES, engine tìm SOLUTION
  - AI gốc (expert systems, natural language)

Tại sao:
  Unification + backtracking: engine thử mọi tổ hợp cho đến khi match.
```

---

---

# PHẦN 2: PHÂN NHÓM THEO NỀN TẢNG HOẠT ĐỘNG

---

## Nhóm A: Native Machine Code — Chạy trực tiếp trên CPU

```
Ngôn ngữ:  C, C++, Rust, Zig, Go, Swift, Fortran, ASM
Output:    Machine code (ELF/Mach-O/PE executable)
Runtime:   Không (hoặc minimal — Go có GC runtime)

TẠI SAO CHÚNG LÀM ĐƯỢC:
  Compiler translate source → machine instructions.
  CPU đọc instructions trực tiếp. Không cần interpreter.
  
  Source → Compiler → Machine Code → CPU Execute
  
SYSCALLS chúng dùng (Linux x86_64):
  Memory:    mmap(9), munmap(11), brk(12)
  File I/O:  open(2), read(0), write(1), close(3)
  Network:   socket(41), connect(42), bind(49), listen(50), accept(43)
  Process:   fork(57), execve(59), wait4(61), exit(60)
  Thread:    clone(56), futex(202)
  Signal:    rt_sigaction(13), rt_sigprocmask(14)
  Time:      clock_gettime(228), nanosleep(35)
  Misc:      ioctl(16), mprotect(10), getpid(39)

→ ~30 syscalls = nền tảng cho MỌI THỨ máy tính làm được.
```

## Nhóm B: Virtual Machine — Chạy trên VM, VM chạy trên CPU

```
Ngôn ngữ:  Java, Kotlin, Scala, C#, F#
VM:        JVM (.class bytecode), CLR (.NET IL bytecode)
Runtime:   GC + JIT compiler + class loader

TẠI SAO CHÚNG LÀM ĐƯỢC:
  Source → Compiler → Bytecode → VM interpret/JIT → Machine Code → CPU
  
  VM = CPU ảo. Bytecode = ASM ảo.
  JIT: detect hot code → compile thành native → cache.
  
  "Write once, run anywhere" vì: VM abstract phần cứng.
  Cùng bytecode chạy trên x86, ARM, RISC-V → VM lo phần translate.

THÊM gì so với Nhóm A:
  - Bytecode format (class file, IL format)
  - Bytecode verifier (type-safe trước khi chạy)
  - Class loader (load code dynamically)
  - GC (heap management tự động)
  - JIT compiler (bytecode → native, tại runtime)
  - Reflection (code inspect chính nó tại runtime)
```

## Nhóm C: Interpreter — Đọc source, chạy từng dòng

```
Ngôn ngữ:  Python, Ruby, PHP, Perl, Lua
Runtime:   Interpreter (viết bằng C) + GC + stdlib

TẠI SAO CHÚNG LÀM ĐƯỢC:
  Source → Interpreter READ → Interpreter EXECUTE → syscalls
  
  Interpreter = chương trình C đọc source code và thực hiện.
  CPython: parse .py → bytecode → ceval.c loop → mỗi opcode = 1 C function.
  
  Chậm hơn Nhóm A/B vì: mỗi operation = nhiều C instructions.
  Nhưng DỄ hơn vì: dynamic typing, REPL, no compile step.

THÊM gì so với Nhóm A:
  - Parser/tokenizer (đọc source tại runtime)
  - Dynamic type system (mỗi value mang type tag)
  - GC (reference counting + cycle detection — CPython)
  - Eval (chạy string như code)
  - Module system (import tại runtime)
  - C FFI (gọi C functions từ Python — ctypes, cffi)
```

## Nhóm D: Browser — Chạy trong trình duyệt

```
Ngôn ngữ:  JavaScript, TypeScript, WASM
Runtime:   Browser engine (V8, SpiderMonkey, JavaScriptCore)

TẠI SAO CHÚNG LÀM ĐƯỢC:
  JS Source → Parser → AST → JIT → Machine Code (trong browser sandbox)
  
  WASM Binary → Validate → JIT → Machine Code (trong browser sandbox)
  
  Browser = OS thu nhỏ:
    DOM = filesystem (tree of nodes)
    fetch() = networking
    Canvas/WebGL = graphics
    Web Audio = audio
    localStorage = persistence
    WebSocket = real-time communication
    WebRTC = peer-to-peer
    
KHÔNG CÓ syscalls trực tiếp. Browser API thay thế:
  File:    fetch() thay sys_read
  Network: XMLHttpRequest / fetch() thay sys_socket
  Thread:  Web Workers thay sys_clone
  Timer:   setTimeout thay sys_nanosleep
  Storage: localStorage/IndexedDB thay sys_write
```

## Nhóm E: Specialized Runtime — Chạy trong môi trường đặc biệt

```
Ngôn ngữ:  SQL, HTML/CSS, Solidity, Prolog, R, MATLAB
Runtime:   Database engine, Browser DOM, EVM, Prolog engine

TẠI SAO CHÚNG LÀM ĐƯỢC:
  Không phải general-purpose. Engine HIỂU domain.
  
  SQL: database optimizer biết cách tìm data nhanh nhất
  HTML: browser layout engine biết cách render
  Solidity: EVM biết cách đảm bảo consensus
  Prolog: unification engine biết cách tìm solution
  
THÊM gì: domain-specific engine, KHÔNG thể replace bằng syscalls đơn thuần.
```

---

---

# PHẦN 3: SILK — KẾT NỐI MỌI KHẢ NĂNG

---

## 3.1 Tất cả chức năng, gom lại

```
COMPUTE (tính toán):
  ├── Integer arithmetic ← ALU instructions (ADD, SUB, MUL, DIV)
  ├── Float arithmetic ← FPU/SSE/AVX (ADDSD, MULSD, SQRTSD)
  ├── Bit operations ← AND, OR, XOR, SHL, SHR
  ├── SIMD/Vector ← SSE, AVX2, AVX-512 (process 4-16 values cùng lúc)
  └── GPU compute ← PCIe → GPU → CUDA/OpenCL kernels

MEMORY (bộ nhớ):
  ├── Stack ← RSP register, PUSH/POP
  ├── Heap ← mmap syscall (9)
  ├── GC ← mark-sweep / reference counting / generational
  ├── Arena ← bump allocator (mmap + pointer increment)
  └── Memory-mapped files ← mmap with file descriptor

STORAGE (lưu trữ):
  ├── File I/O ← open(2), read(0), write(1), close(3)
  ├── Directory ← mkdir(83), getdents(78)
  ├── Database ← file I/O + B-tree index + query optimizer
  └── Key-value ← file I/O + hash table

NETWORK (mạng):
  ├── TCP ← socket(41), connect(42), send(44), recv(45)
  ├── UDP ← socket(41), sendto(44), recvfrom(45)
  ├── HTTP ← TCP + text protocol ("GET / HTTP/1.1\r\n...")
  ├── HTTPS ← TCP + TLS handshake + AES encryption
  ├── WebSocket ← HTTP upgrade → binary frames
  └── DNS ← UDP port 53

CONCURRENCY (đồng thời):
  ├── Threads ← clone(56) syscall
  ├── Mutex ← futex(202) syscall
  ├── Async I/O ← epoll_create(213), epoll_ctl(233), epoll_wait(232)
  ├── Coroutines ← user-space stack switching (no syscall)
  └── Goroutines ← user-space scheduler + epoll

PROCESS (tiến trình):
  ├── Spawn ← fork(57) + execve(59)
  ├── Pipe ← pipe(22)
  ├── Signal ← rt_sigaction(13)
  └── IPC ← shared memory: mmap(9) + MAP_SHARED

TEXT (xử lý văn bản):
  ├── UTF-8 decode ← byte pattern matching (pure computation)
  ├── Regex ← NFA/DFA engine (pure computation)
  ├── String ops ← memcpy, memcmp, strlen (pure computation)
  └── JSON/XML parse ← state machine (pure computation)

CRYPTO (mật mã):
  ├── Hash ← SHA-256 (pure computation, hoặc CPU SHA extensions)
  ├── Symmetric ← AES-256 (pure computation, hoặc AES-NI instructions)
  ├── Asymmetric ← Ed25519 (pure computation, big integer math)
  └── TLS ← hash + AES + key exchange + certificates

GRAPHICS (đồ họa):
  ├── Terminal ← write(1) + ANSI escape codes
  ├── Framebuffer ← mmap(/dev/fb0) + pixel writes
  ├── GPU 2D/3D ← ioctl → GPU driver → shaders
  └── Browser ← Canvas API / WebGL (qua JS bridge)

AUDIO:
  ├── Play ← write to /dev/snd/* hoặc ALSA ioctl
  ├── Record ← read from /dev/snd/*
  └── Process ← pure computation (FFT, filters)

INPUT:
  ├── Keyboard ← read(0) from stdin, hoặc /dev/input/*
  ├── Mouse ← /dev/input/* event
  ├── Touch ← /dev/input/* event
  └── Sensor ← /dev/i2c-*, /dev/spi*
```

## 3.2 Silk connections — Mọi thứ nối với nhau thế nào

```
                         ┌─────────────┐
                         │   SYSCALLS   │
                         │  (~30 calls) │
                         └──────┬──────┘
                                │
        ┌───────────┬───────────┼───────────┬───────────┐
        │           │           │           │           │
   ┌────▼───┐  ┌────▼───┐  ┌───▼────┐  ┌───▼───┐  ┌───▼────┐
   │ MEMORY │  │ FILE   │  │NETWORK │  │PROCESS│  │ TIME   │
   │ mmap   │  │ open   │  │socket  │  │ fork  │  │ clock  │
   │ munmap │  │ read   │  │connect │  │execve │  │ sleep  │
   │ mprotect│ │ write  │  │send    │  │wait   │  │        │
   └────┬───┘  │ close  │  │recv    │  │       │  └───┬────┘
        │      └────┬───┘  │bind    │  └───┬───┘      │
        │           │      │listen  │      │           │
        │           │      │accept  │      │           │
        │           │      │epoll   │      │           │
        │           │      └───┬────┘      │           │
        │           │          │           │           │
        ▼           ▼          ▼           ▼           ▼
   ┌─────────────────────────────────────────────────────┐
   │              PURE COMPUTATION                        │
   │  (không cần syscall — chỉ cần CPU instructions)     │
   │                                                      │
   │  Integer math, float math, string ops, regex,       │
   │  JSON parse, hash, encrypt, compress, sort,         │
   │  graph algorithms, tree operations, encode/decode   │
   │                                                      │
   │  → ĐÂY LÀ NƠI OLANG MẠNH NHẤT (157K P_weights,   │
   │    5D encode, chain compose, mol distance)           │
   └──────────────────────────────────────────────────────┘

MỌI ngôn ngữ = PURE COMPUTATION + SYSCALL WRAPPERS.
Khác nhau ở:
  - Bao nhiêu syscalls được wrap
  - Wrap ĐẸP hay XẤU (API design)
  - Computation nhanh hay chậm (compiler quality)
  - Thêm gì ở giữa (GC, JIT, type system)
```

---

---

# PHẦN 4: ASM CẦN GÌ CHO MỖI KHẢ NĂNG

---

## 4.1 Bảng syscalls → khả năng

```
#    SYSCALL         ASM CODE                      UNLOCK
─────────────────────────────────────────────────────────────
0    sys_read        mov $0, %rax; syscall         File read, stdin
1    sys_write       mov $1, %rax; syscall         File write, stdout
2    sys_open        mov $2, %rax; syscall         Open files
3    sys_close       mov $3, %rax; syscall         Close files
9    sys_mmap        mov $9, %rax; syscall         Memory allocation
11   sys_munmap      mov $11, %rax; syscall        Memory free
35   sys_nanosleep   mov $35, %rax; syscall        Timer, sleep
41   sys_socket      mov $41, %rax; syscall        ★ TCP/UDP networking
42   sys_connect     mov $42, %rax; syscall        ★ Connect to server
43   sys_accept      mov $43, %rax; syscall        ★ Accept connection
44   sys_sendto      mov $44, %rax; syscall        ★ Send data
45   sys_recvfrom    mov $45, %rax; syscall        ★ Receive data
49   sys_bind        mov $49, %rax; syscall        ★ Bind port (server)
50   sys_listen      mov $50, %rax; syscall        ★ Listen (server)
56   sys_clone       mov $56, %rax; syscall        ★★ Threads
57   sys_fork        mov $57, %rax; syscall        Spawn process
59   sys_execve      mov $59, %rax; syscall        Run program
60   sys_exit        mov $60, %rax; syscall        Exit
78   sys_getdents    mov $78, %rax; syscall        List directory
202  sys_futex       mov $202, %rax; syscall       ★★ Thread sync
213  sys_epoll_create mov $213, %rax; syscall      ★★ Async I/O
232  sys_epoll_wait  mov $232, %rax; syscall       ★★ Async I/O
233  sys_epoll_ctl   mov $233, %rax; syscall       ★★ Async I/O
228  sys_clock_gettime mov $228, %rax; syscall     Precise time

★   = networking (HTTP, WebSocket, crawler)
★★  = concurrency (goroutines, async, parallel)
```

## 4.2 Mỗi syscall = bao nhiêu dòng ASM?

```
Mỗi syscall wrapper = ~10-15 dòng ASM:

.sys_socket:
    # Pop arguments from VM stack
    sub $16, %r14                  # pop protocol
    mov (%r14), %rdx
    sub $16, %r14                  # pop type
    mov (%r14), %rsi
    sub $16, %r14                  # pop domain
    mov (%r14), %rdi
    # Syscall
    mov $41, %rax
    syscall
    # Push result to VM stack
    mov %rax, (%r14)
    movq $-1, 8(%r14)             # F64 marker
    add $16, %r14
    jmp .next_op

Tổng cho TOÀN BỘ networking: ~100 dòng ASM (7 syscalls × ~15 dòng)
Tổng cho TOÀN BỘ threading:  ~60 dòng ASM (3 syscalls × ~20 dòng)
Tổng cho TOÀN BỘ async I/O:  ~50 dòng ASM (3 syscalls × ~17 dòng)
──────────────────────────────────
TỔNG: ~210 dòng ASM = UNLOCK networking + threading + async
```

## 4.3 Pure computation — KHÔNG cần syscall mới

```
Những thứ Olang KHÔNG CẦN thêm ASM để làm:

  ✅ JSON parse      → string ops (đã có)
  ✅ XML parse       → string ops (đã có)
  ✅ Regex           → state machine bằng Olang
  ✅ HTTP protocol   → string formatting + socket syscalls
  ✅ TLS/HTTPS       → SHA-256 (đã có) + AES (cần thêm) + big int
  ✅ Database        → file I/O (đã có) + B-tree bằng Olang
  ✅ Compression     → thuật toán bằng Olang (zlib, LZ4)
  ✅ Sort/Search     → thuật toán bằng Olang (đã có quicksort)
  ✅ Graph/Tree      → thuật toán bằng Olang (KnowTree đã có)
  ✅ Encode/Decode   → 157,386 P_weights (đã có)
  
  ❌ AES-256-GCM     → cần ~200 dòng ASM (hoặc dùng AES-NI CPU instruction)
  ❌ SIMD math       → cần SSE/AVX instructions (~100 dòng ASM)
  ❌ GPU access      → cần ioctl syscall (~20 dòng ASM)
```

---

---

# PHẦN 5: TỔNG HỢP — CÁI GÌ ĐƯA VÀO OLANG

---

## 5.1 Olang VM hiện có

```
SYSCALLS ĐÃ CÓ:
  sys_read (0)       ✅
  sys_write (1)      ✅
  sys_open (2)       ✅ (qua __file_read, __file_write)
  sys_close (3)      ✅
  sys_mmap (9)       ✅ (heap allocation)
  sys_exit (60)      ✅
  sys_nanosleep (35) ✅ (__sleep)

PURE COMPUTATION ĐÃ CÓ:
  Integer math       ✅ (add, sub, mul, div, mod)
  Float math         ✅ (f64: add, sub, mul, div, sqrt, floor, ceil)
  String ops         ✅ (len, char_at, substr, trim, concat, compare)
  UTF-8              ✅ (__utf8_cp, __utf8_len)
  Array ops          ✅ (push, len, set_at, index)
  Dict ops           ✅ (create, field access)
  Hash               ✅ (SHA-256 FIPS, FNV-1a)
  Closures           ✅ (first-class functions)
  P_weight encode    ✅ (157,386 entries)
```

## 5.2 Cần thêm — theo thứ tự ưu tiên

```
TIER 1: NETWORKING (HTTP, crawler) — ~100 dòng ASM
  Thêm 7 syscall wrappers:
    __socket(domain, type, protocol) → fd
    __connect(fd, addr, port) → 0/-1
    __bind(fd, addr, port) → 0/-1
    __listen(fd, backlog) → 0/-1
    __accept(fd) → new_fd
    __send(fd, data) → bytes_sent
    __recv(fd, max_len) → data
  
  Viết bằng Olang (sau khi có syscalls):
    http_get(url) → text           ~80 LOC Olang
    http_post(url, body) → text    ~40 LOC Olang
    json_parse(text) → dict        ~100 LOC Olang
    wiki_fetch(title) → text       ~30 LOC Olang

TIER 2: LARGER HEAP — ~5 dòng ASM
  Sửa heap size: 64MB → 512MB (sửa 1 hằng số trong vm_x86_64.S)
  → Unlock: đọc sách, đọc Wikipedia, KnowTree hàng nghìn facts

TIER 3: FILE SYSTEM — ~30 dòng ASM
  Thêm 3 syscalls:
    __mkdir(path) → 0/-1
    __readdir(path) → [names]
    __stat(path) → {size, mtime}
  
  Viết bằng Olang:
    list_files(dir) → [paths]      ~20 LOC Olang
    walk_dir(dir) → [all_paths]    ~30 LOC Olang

TIER 4: CONCURRENCY — ~60 dòng ASM
  Thêm 3 syscalls:
    __thread_create(fn) → tid
    __thread_join(tid) → result
    __atomic_cas(addr, old, new) → bool
    
  Viết bằng Olang:
    parallel_map(fn, items) → results   ~50 LOC Olang
    channel_send/recv                    ~40 LOC Olang

TIER 5: ASYNC I/O — ~50 dòng ASM
  Thêm 3 syscalls:
    __epoll_create() → epfd
    __epoll_add(epfd, fd, events) → 0/-1
    __epoll_wait(epfd, timeout) → [{fd, events}]
  
  Viết bằng Olang:
    event_loop + non-blocking I/O       ~100 LOC Olang
    → Concurrent networking không cần threads

TIER 6: CRYPTO — ~200 dòng ASM
  Thêm:
    __aes_encrypt(key, data) → encrypted
    __aes_decrypt(key, data) → decrypted
    __ed25519_sign(key, msg) → signature
    __ed25519_verify(pub, msg, sig) → bool
  
  → Unlock: HTTPS, TLS, signed records, secure storage

TIER 7: GRAPHICS — ~20 dòng ASM + ~200 LOC Olang
  Thêm:
    __ioctl(fd, cmd, arg) → 0/-1  (generic ioctl)
  
  Viết bằng Olang:
    Terminal UI (ANSI escape codes)     ~100 LOC Olang (đã có phần)
    Framebuffer rendering               ~100 LOC Olang
```

## 5.3 Tổng kết: ASM cần thêm

```
Hiện có:     ~6,000 dòng ASM

Cần thêm:
  Networking:  ~100 dòng   (7 syscall wrappers)
  Heap:        ~5 dòng     (sửa 1 hằng số)
  Filesystem:  ~30 dòng    (3 syscall wrappers)
  Threading:   ~60 dòng    (3 syscall wrappers)
  Async I/O:   ~50 dòng    (3 syscall wrappers)
  Crypto:      ~200 dòng   (AES, Ed25519 primitives)
  Graphics:    ~20 dòng    (ioctl wrapper)
  ─────────────────────
  TỔNG:        ~465 dòng ASM

Sau đó MỌI THỨ KHÁC = Olang:
  HTTP client:     ~80 LOC Olang
  HTTP server:     ~120 LOC Olang
  JSON:            ~100 LOC Olang
  Crawler:         ~50 LOC Olang
  Database:        ~200 LOC Olang
  Regex:           ~150 LOC Olang
  Event loop:      ~100 LOC Olang
  Thread pool:     ~80 LOC Olang
  TLS client:      ~300 LOC Olang
  Terminal UI:     ~100 LOC Olang
  ─────────────────────
  TỔNG:            ~1,280 LOC Olang

6,000 (hiện có) + 465 (thêm) = 6,465 dòng ASM
→ UNLOCK mọi khả năng của mọi ngôn ngữ lập trình.

Phần còn lại = Olang thuần.
```

---

---

# PHẦN 6: NGUYÊN LÝ CƠ BẢN

```
MỌI ngôn ngữ lập trình = 2 thứ:

  1. SYSCALLS      — nói chuyện với phần cứng (qua OS kernel)
  2. COMPUTATION   — tính toán trong memory

  C      = 30 syscalls + minimal abstraction
  Python = 30 syscalls + interpreter + GC + dynamic types + 400K packages  
  Java   = 30 syscalls + VM + JIT + GC + type system + ecosystem
  Go     = 30 syscalls + goroutine scheduler + GC + fast compiler
  Rust   = 30 syscalls + borrow checker + zero-cost abstractions
  JS     = browser APIs (= syscalls qua browser) + event loop + JIT

  CÙNG SYSCALLS. Khác cách BỌC.

Olang hiện tại = 7 syscalls + VM + compiler + 157K encode table
Olang + 465 dòng ASM = 30 syscalls + VM + compiler + 157K encode table
  → CÓ THỂ làm mọi thứ mọi ngôn ngữ khác làm
  → CỘNG THÊM: 5D encode, mol distance, KnowTree — cái KHÔNG AI CÓ
```

---

*Không so sánh. Không bán mơ. Chỉ sự thật.*
*30 syscalls = nền tảng mọi phần mềm trên thế giới.*
*Olang có 7. Cần thêm 23. Đó là 465 dòng ASM.*
