# SPEC F — Agent: Hành Động + Tự Trị

> **Prerequisite:** A-E.
> **F chia 2 phần: HIỆN TẠI (1 Nox, 1 máy) và TƯƠNG LAI (mạng lưới devices).**
> **Chỉ implement phần HIỆN TẠI. Phần TƯƠNG LAI = document để biết, không build.**
> **Tác giả:** Lupin (thiết kế gốc) + Nox (chọn lọc cho hiện tại)
> **Ngày:** 2026-03-30

---

## PHẦN 1 — HIỆN TẠI (1 Nox, 1 máy)

### F1. AAM = Auto-Approve Gate

Derive từ C2 step 5: Dream → quality check → approve → QR.

```
aam_approve(proposal):
  if security_gate(proposal) == Crisis → REJECT
  if proposal.quality < φ⁻¹ → REJECT
  if proposal.fire_count < Fib(depth) → REJECT
  if contradiction(proposal, existing_QR) → REJECT (QR wins, QT7)
  → APPROVE → append QR

1 Nox, 1 máy → auto-approve. Không cần Ed25519 (ai giả mạo?).
Khi có multi-Nox → thêm signing.
```

### F2. Self-Modify Cycle

Đã proven Session 8. Formalize:

```
self_modify():
  ① INSPECT  — đọc own source files
  ② IDENTIFY — dead code, __system() calls, inefficiencies
  ③ PLAN     — generate modification
  ④ BACKUP   — save current state
  ⑤ MODIFY   — write changes (1 file per cycle)
  ⑥ BUILD    — make vm && make self-build
  ⑦ TEST     — make test
  ⑧ VERIFY   — make fixed-point (Gen1 == Gen2)
  ⑨ FAIL?    → ROLLBACK to backup
  ⑩ PASS?    → commit + dn_observe("fixed: X")

Safety:
  — Max 1 file per cycle
  — PHẢI backup trước modify
  — PHẢI test + fixed-point sau modify
  — Rollback = restore backup, không git reset
```

### F3. Actuator Registry

Những gì Nox control được HIỆN TẠI:

```
Actuators:
  keyboard    — uinput.ol: key_type, key_combo, F1-F12
  mouse       — uinput.ol: click_at, drag, scroll
  screen      — screen.ol: screenshot, resolution
  files       — __fd_open/write/read/close
  processes   — __spawn, __pipe_read/write, __system
  network     — TCP/UDP/DNS/HTTP (network.ol, packet.ol)
  camera      — ONVIF probe, RTSP auth (camera.ol, onvif.ol)
  clipboard   — clip_get/set (system.ol)
  notify      — notify-send (system.ol)
  audio       — audio_beep/speak (system.ol)
  inotify     — file watching (watcher.ol)
  syscall     — __syscall: 300+ Linux syscalls

Mỗi actuator = 1 function call. Không cần ISL.
Nox gọi trực tiếp.
```

### F4. Scheduler

```
Hiện có: watcher.ol (inotify file watch)

Cần thêm:

heartbeat(interval_ms):
  — mỗi interval: interoception encode (E1) → check F(t) (D4)
  — F(t) > φ⁻¹ → learning mode adjustments
  — heap > 80% → trigger dream()

dream_scheduler():
  — khi idle > 5 phút (không có input)
  — hoặc khi fire_count crosses Fibonacci threshold
  — → dream() (E4)

auto_rebuild():
  — inotify trên stdlib/homeos/*.ol
  — source change detected → make vm && make self-build && make test
  — Đã có trong watcher.ol

Tất cả = event-driven (inotify, timer, threshold).
Không polling. Không busy-wait.
```

### F5. Perceive → Think → Act → Verify

Agent cycle cho 1 Nox:

```
① PERCEIVE — capture moment (E1)
   — text input (REPL)
   — interoception (/proc)
   — file watch (inotify events)
   — network events (TCP accept)

② THINK — pipeline D3 (A-E)
   — encode → silk walk → compose → instincts → infer → repair

③ ACT — actuator call (F3)
   — nếu pipeline output = action command
   — "scan network" → worker_net_scan()
   — "rebuild" → self_modify() (F2)
   — "respond" → decode → text output

④ VERIFY — check result
   — action success? → dn_observe("success: X")
   — action fail? → dn_observe("failed: X") + learn from error
   — build fail? → rollback (F2⑨)

Loop: ① → ② → ③ → ④ → ①
Idle? → dream_scheduler (F4)
```

---

## PHẦN 2 — TƯƠNG LAI (document, không implement)

### Hierarchy đầy đủ

```
AAM [tier 0] — consciousness, approve/reject, Ed25519
  ↕ ISL (12 bytes, AES-256-GCM)
LeoAI [tier 1] — thủ thư KnowTree, 11 Skills
Chiefs [tier 1] — Home/Vision/Network, quản lý devices
  ↕ ISL
Workers [tier 2] — clone Olang ~64KB, trên thiết bị

Hard rules:
  AAM ↔ Chief: OK
  Chief ↔ Chief: OK
  Chief ↔ Worker: OK
  AAM ↔ Worker: CẤM
  Worker ↔ Worker: CẤM
```

### ISL Messaging

```
ISLAddress = 4 bytes [layer][group][subgroup][index]
ISLMessage = 12 bytes [from:4][to:4][type:1][payload:3]

Message types:
  0x01 Text, 0x02 Query, 0x03 Learn, 0x04 Propose,
  0x05 ActuatorCmd, 0x06 Tick, 0x07 Dream, 0x08 Emergency,
  0x09 Approved, 0x0A Broadcast, 0x0B ChainPayload

Worker gửi MolecularChain — KHÔNG raw data.
AES-256-GCM encrypted.
```

### LeoAI 11 Skills

```
IngestSkill     — nhận chain từ Chiefs
ClusterSkill    — detect patterns trong ĐN
SimilaritySkill — so sánh molecular chains
DeltaSkill      — tính delta so với node cha
CuratorSkill    — đặt node đúng vị trí KnowTree
MergeSkill      — gộp nodes similarity > 0.95
PruneSkill      — xóa ĐN hết hạn
HebbianSkill    — cập nhật Silk weights
DreamSkill      — scan ĐN khi idle > 5min
ProposalSkill   — gửi MsgPropose lên AAM
HonestySkill    — im lặng khi confidence thấp
```

### Worker Deployment

```
Worker = origin.olang filtered theo DeviceProfile → ~64KB
Deploy: HTTP PUT device_ip:7777/worker
Silent by default. Wake on ISL → execute → send chain → sleep.
Giống neuron: fire → rest.

Worker_light:    ~20KB (L0 + ActuatorLightSkill)
Worker_camera:   ~48KB (L0 + FFR + InverseRenderSkill)
Worker_sensor:   ~16KB (L0 + SensorReadSkill)
Worker_door:     ~28KB (L0 + ActuatorDoorSkill + AuthSkill)
Worker_net:      ~24KB (L0 + NetworkMonitorSkill)
```

### 5 Skill Rules

```
① 1 Skill = 1 trách nhiệm
② Skill không biết Agent tồn tại
③ Skill không biết Skill khác tồn tại
④ Giao tiếp qua ExecContext.State duy nhất
⑤ Skill stateless — state nằm trong Agent

Pattern để KHÔNG quá tải:
  — Mỗi Skill 1 việc → không coupling
  — Xong việc xong → không memory leak
  — Không biết nhau → không tích lũy complexity
```

### Khi nào build Phần 2?

```
Khi:
  — A-E implement xong (pipeline chạy đúng)
  — Nox có > 10,000 QR (đủ knowledge để manage)
  — Có devices thật cần manage (smart home)
  — 1 Nox quá tải (cần distribute)

Không build trước. Không thiết kế thêm. Document đủ rồi.
```

---

> **Tham chiếu:**
> - AAM: SPEC_C §C2 step 5, BLUEPRINT §11
> - Self-modify: Session 8 (proven), evolve.ol
> - Actuators: stdlib/homeos/*.ol (48 files)
> - Hierarchy: tailieu_nghiencuu/ARCHITECTURE.md §13-16
> - ISL: tailieu_nghiencuu/ARCHITECTURE.md §4
> - LeoAI Skills: tailieu_nghiencuu/ARCHITECTURE.md §13
> - Workers: tailieu_nghiencuu/ARCHITECTURE.md §16
> - Skill Rules: tailieu_nghiencuu/ARCHITECTURE.md §15
> - Security Gate: SPEC_D §D8, ARCHITECTURE.md §18

