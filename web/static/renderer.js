// web/static/renderer.js
// ISDF Production Renderer
// Thay thế hardcoded ND[]/ED[] bằng live /api/tree + /api/edges + SSE snapshot
// Drop-in: gọi initISDF(canvas) để khởi động

'use strict';

// ── Constants ──────────────────────────────────────────────────
const ISO = { TW: 38, TH: 19, HS: 30 };
const TAU = Math.PI * 2;
const CAT_HUE = { scripts:210, math:270, geo:150, emoji:35, olang:320, punct:180, numbers:60, musical:295, origin:0 };

// ── Camera state ───────────────────────────────────────────────
let W, H, ctx, canvas;
let camTheta = Math.PI/5, camPhi = Math.PI/8, camDist = 640;
let tTheta = camTheta, tPhi = camPhi, tDist = camDist;
let camPanX = 0, camPanY = 0;
let autoSpin = true;
let T = 0;                // animation time
let worldTime = 10;       // HomeOS world time [0..24]

// ── Scene data — populated from API ────────────────────────────
let NODES = {};           // id → node object
let EDGES = [];           // {fid, tid, op}
let SIGS  = [];           // active signal particles
let STATS = {qr:0, dn:0, nt:0};

// ── SDF opTable (mirrored from Go gene/sdf.go) ─────────────────
const 𝔻 = {
  '●': (px,py,pz, cx,cy,cz, r) => Math.hypot(px-cx,py-cy,pz-cz)-r,
  '∪': (d1,d2,k) => { if(k<1e-9) return Math.min(d1,d2); const h=Math.max(k-Math.abs(d1-d2),0)/k; return Math.min(d1,d2)-h*h*k*.25; },
  '∇': (px,py,pz, sdf, e) => {
    const nx=sdf(px+e,py,pz)-sdf(px-e,py,pz);
    const ny=sdf(px,py+e,pz)-sdf(px,py-e,pz);
    const nz=sdf(px,py,pz+e)-sdf(px,py,pz-e);
    const l=Math.hypot(nx,ny,nz)||1; return [nx/l,ny/l,nz/l];
  },
  '·': (a,b) => a[0]*b[0]+a[1]*b[1]+a[2]*b[2],
  '☀': t => {
    const a=(t-6)/24*TAU;
    const i=Math.max(0,Math.sin((t-6)/12*Math.PI));
    return {x:-Math.cos(a)*.6, y:Math.sin(a)*.5+.5, z:-.4, i, a:.25};
  },
  '👁': (ox,oy,oz, dx,dy,dz, sdf) => {
    let t=0;
    for(let i=0;i<256;i++){
      const d=sdf(ox+dx*t,oy+dy*t,oz+dz*t);
      if(d<.0005) return t;
      t+=d; if(t>500) return -1;
    } return -1;
  },
};

// ── toScreen — ISDF projection ─────────────────────────────────
function toScreen(wx,wy,wz){
  const ct=Math.cos(camTheta),st=Math.sin(camTheta);
  const rx= wx*ct+wz*st, rz=-wx*st+wz*ct;
  const cp=Math.cos(camPhi), sp=Math.sin(camPhi);
  const ry= wy*cp-rz*sp, rz2=wy*sp+rz*cp;
  const scale=camDist/(camDist+rz2*55)*(camDist/640);
  return { sx:W/2+camPanX+(rx-rz2)*ISO.TW*.5*scale,
           sy:H/2+camPanY+(rx+rz2)*ISO.TH*.5*scale-ry*ISO.HS*scale,
           depth:rz2, scale };
}

// ── Catmull-Rom spline eval ────────────────────────────────────
function catmullRom(p0,p1,p2,p3,t){
  const t2=t*t,t3=t2*t;
  return 0.5*((-p0+3*p1-3*p2+p3)*t3+(2*p0-5*p1+4*p2-p3)*t2+(-p0+p2)*t+2*p1);
}
function splineEval(kf,t){
  const n=kf.length; if(n===0) return {x:0,y:0,z:0};
  t=((t%1)+1)%1;
  const seg=t*n, i=Math.floor(seg), f=seg-i;
  const p=(j)=>kf[(j+n)%n];
  return {
    x:catmullRom(p(i-1).x,p(i).x,p(i+1).x,p(i+2).x,f),
    y:catmullRom(p(i-1).y,p(i).y,p(i+1).y,p(i+2).y,f),
    z:catmullRom(p(i-1).z,p(i).z,p(i+1).z,p(i+2).z,f),
  };
}
function makeOrbit(base,amp,phase){
  return [0,1,2,3].map(k=>{
    const a=phase+k/4*TAU;
    return {t:k/4, x:base.x+amp*Math.cos(a), y:base.y+amp*.4*Math.sin(a*1.3+.5), z:base.z+amp*Math.sin(a)};
  });
}

// ── Color helpers ──────────────────────────────────────────────
function cc(cat,a=1){ return `hsla(${CAT_HUE[cat]??200},65%,58%,${a})`; }

// ── Node radius by layer ───────────────────────────────────────
function nodeR(n){
  if(n.layer===0) return 1.5;
  if(n.layer===1) return 0.58+(n.deg||0)*.025;
  if(n.layer===2) return 0.36;
  return 0.26;
}

// ── LOD check ─────────────────────────────────────────────────
function needsFullRender(n, p){
  const sr = Math.max(0.5, nodeR(n)*p.scale*(camDist/120));
  return sr >= 4;  // < 4px screen → skip complex render
}

// ── Fetch data from Go API ──────────────────────────────────────
const API = window.HOMEOS_API || 'http://localhost:8080';
const SSE_URL = window.HOMEOS_SSE || 'http://localhost:8080/ws/sse';

async function fetchTree(){
  try {
    const r = await fetch(API+'/api/tree');
    const data = await r.json();
    data.forEach(n => {
      const id = n.id || n.name;
      if(!NODES[id]){
        const phase = id.split('').reduce((a,c,i)=>a+c.charCodeAt(0)*(0.37+i*.13),0);
        const amp   = n.layer===0?0:[0,.038,.024,.015][n.layer]||.015;
        const bx=0, by=0, bz=0; // base position — server may override
        NODES[id] = {
          id, name:n.name, glyph:n.glyph||'○', cat:n.cat||'origin',
          L:n.layer||0, deg:0, glow:0, sig:0,
          x:bx, y:by, z:bz,
          kf: makeOrbit({x:bx,y:by,z:bz}, amp, phase),
        };
      }
    });
    layoutNodes(); // assign 3D positions
  } catch(e){ console.warn('fetchTree failed, using defaults', e); }
}

async function fetchEdges(){
  try {
    const r = await fetch(API+'/api/edges');
    const data = await r.json();
    EDGES = data.map(e=>({ fid:e.from||e.from_id, tid:e.to||e.to_id, op:e.op||'∈' }));
    // Update deg counts
    EDGES.forEach(e=>{ if(NODES[e.fid]) NODES[e.fid].deg=(NODES[e.fid].deg||0)+1; });
  } catch(e){ console.warn('fetchEdges failed', e); }
}

// ── Layout — assign 3D positions in sphere shells ─────────────
function layoutNodes(){
  const byLayer = {};
  Object.values(NODES).forEach(n=>{ (byLayer[n.L]=byLayer[n.L]||[]).push(n); });
  const R = [0, 3.2, 6.5, 10.2, 14.5];
  Object.entries(byLayer).forEach(([l,ns])=>{
    const layer = parseInt(l);
    if(layer===0){ ns[0].x=ns[0].y=ns[0].z=0; ns[0].kf=makeOrbit({x:0,y:0,z:0},0,0); return; }
    const r = R[layer]||14;
    ns.forEach((n,i)=>{
      // Fibonacci sphere distribution
      const golden = Math.PI*(3-Math.sqrt(5));
      const y = 1-2*i/(ns.length-1||1);
      const rad = Math.sqrt(Math.max(0,1-y*y))*r;
      const theta = golden*i;
      n.x = rad*Math.cos(theta);
      n.y = y*r*.55;
      n.z = rad*Math.sin(theta);
      const amp = [0,.038,.024,.015][layer]||.015;
      const phase = n.id.split('').reduce((a,c,i)=>a+c.charCodeAt(0)*(0.37+i*.13),0);
      n.kf = makeOrbit({x:n.x,y:n.y,z:n.z}, amp, phase);
    });
  });
}

// ── SSE — receive WorldSnapshot from Go ──────────────────────
let sseSource = null;
function connectSSE(){
  if(sseSource) sseSource.close();
  sseSource = new EventSource(SSE_URL);
  sseSource.onmessage = evt => {
    try { updateWorld(JSON.parse(evt.data)); } catch{}
  };
  sseSource.onerror = () => {
    setTimeout(connectSSE, 3000);
  };
}

function updateWorld(snap){
  if(snap.time !== undefined) worldTime = snap.time;
  if(snap.nodes) snap.nodes.forEach(ns=>{
    if(NODES[ns.id]){
      if(ns.glow) NODES[ns.id].glow = ns.glow;
      if(ns.sig)  NODES[ns.id].sig  = ns.sig;
      if(ns.x !== undefined){ NODES[ns.id].x=ns.x; NODES[ns.id].y=ns.y; NODES[ns.id].z=ns.z; }
    }
  });
  if(snap.signals) snap.signals.forEach(ss=>{
    const existing = SIGS.find(s=>s.fid===ss.from&&s.tid===ss.to);
    if(existing) existing.p = ss.p;
    else SIGS.push({ fid:ss.from, tid:ss.to, p:ss.p||0, sp:.018,
      col:`hsla(${Math.random()*360|0},85%,65%,0.92)` });
  });
  if(snap.stats) STATS = snap.stats;
}

// ── Main render loop ───────────────────────────────────────────
function frame(){
  requestAnimationFrame(frame);
  T += .004;

  // Smooth camera
  camTheta += (tTheta-camTheta)*.062;
  camPhi   += (tPhi  -camPhi  )*.062;
  camDist  += (tDist -camDist )*.062;
  if(autoSpin) tTheta += .0004;

  const sun = 𝔻['☀'](worldTime);
  W = canvas.width = canvas.offsetWidth;
  H = canvas.height= canvas.offsetHeight;

  // Sky
  const dayF = Math.max(0,Math.sin((worldTime-6)/12*Math.PI));
  ctx.fillStyle = `rgb(${2+dayF*7|0},${3+dayF*9|0},${8+dayF*20|0})`;
  ctx.fillRect(0,0,W,H);

  // Sort nodes back→front
  const sorted = Object.values(NODES).map(n=>{
    const pos = splineEval(n.kf, T);
    n.x=pos.x; n.y=pos.y; n.z=pos.z;
    const p = toScreen(n.x,n.y,n.z);
    return {n, p};
  }).sort((a,b)=>a.p.depth-b.p.depth);

  // ── Draw silk edges ────────────────────────────────────────
  ctx.save();
  for(const e of EDGES){
    const fn=NODES[e.fid], tn=NODES[e.tid];
    if(!fn||!tn) continue;
    const pa=toScreen(fn.x,fn.y,fn.z);
    const pb=toScreen(tn.x,tn.y,tn.z);
    const ih = fn.glow>0||tn.glow>0;
    const op = e.op;
    const hue = CAT_HUE[fn.cat]??200;
    ctx.strokeStyle = ih?`hsla(${hue},80%,70%,.82)`:`hsla(${hue},55%,52%,.18)`;
    ctx.lineWidth = ih?1.4:op==='∈'?.55:.38;
    ctx.setLineDash(op==='∈'?[]:op==='≡'?[4,5]:op==='♫'?[2,4]:op==='∘'?[6,3]:[3,6]);
    const cpx=(pa.sx+pb.sx)/2+(pb.sy-pa.sy)*.09;
    const cpy=(pa.sy+pb.sy)/2-(pb.sx-pa.sx)*.09;
    ctx.beginPath(); ctx.moveTo(pa.sx,pa.sy);
    ctx.quadraticCurveTo(cpx,cpy,pb.sx,pb.sy);
    ctx.stroke();
  }
  ctx.setLineDash([]); ctx.restore();

  // ── Draw signal particles — ON the Bezier ─────────────────
  for(let i=SIGS.length-1;i>=0;i--){
    const s=SIGS[i];
    s.p=Math.min(1,s.p+s.sp);
    const fn=NODES[s.fid],tn=NODES[s.tid];
    if(!fn||!tn){SIGS.splice(i,1);continue;}
    const pa=toScreen(fn.x,fn.y,fn.z), pb=toScreen(tn.x,tn.y,tn.z);
    const cpx=(pa.sx+pb.sx)/2+(pb.sy-pa.sy)*.09;
    const cpy=(pa.sy+pb.sy)/2-(pb.sx-pa.sx)*.09;
    const t1=s.p, t0=1-t1;
    const bx=t0*t0*pa.sx+2*t0*t1*cpx+t1*t1*pb.sx;
    const by=t0*t0*pa.sy+2*t0*t1*cpy+t1*t1*pb.sy;
    const sc=pa.scale+(pb.scale-pa.scale)*s.p;
    const pr=Math.max(.5,3.2*sc);
    ctx.save();
    for(let tr=5;tr>=0;tr--){
      const tp=Math.max(0,s.p-tr*.022), tp0=1-tp;
      const trx=tp0*tp0*pa.sx+2*tp0*tp*cpx+tp*tp*pb.sx;
      const try_=tp0*tp0*pa.sy+2*tp0*tp*cpy+tp*tp*pb.sy;
      ctx.globalAlpha=(6-tr)*.028;
      ctx.beginPath(); ctx.arc(trx,try_,Math.max(.3,pr*(1-tr*.14)),0,TAU);
      ctx.fillStyle=s.col; ctx.fill();
    }
    ctx.globalAlpha=1;
    ctx.shadowColor=s.col; ctx.shadowBlur=14;
    ctx.beginPath(); ctx.arc(bx,by,pr,0,TAU);
    ctx.fillStyle=s.col; ctx.fill();
    ctx.restore();
    if(s.p>=1){if(NODES[s.tid])NODES[s.tid].glow=1;SIGS.splice(i,1);}
  }

  // ── Draw nodes (SDF spheres) ───────────────────────────────
  for(const {n,p} of sorted){
    const nr  = nodeR(n);
    const sr2 = Math.max(0.5, nr*p.scale*(camDist/120)*(1+n.glow*.52));

    // LOD check (ISDF-16)
    if(!needsFullRender(n,p) && n.L>2) {
      ctx.beginPath(); ctx.arc(p.sx,p.sy,Math.max(.5,sr2*.5),0,TAU);
      ctx.fillStyle=cc(n.cat,.4); ctx.fill();
      continue;
    }

    // SDF shading via sun
    const nx=p.sx-W/2, ny=p.sy-H/2;
    const viewOff=[nx/sr2*.3,ny/sr2*.3,.8];
    const lo=viewOff[0]*sr2*.05, li=viewOff[1]*sr2*.05;
    const sphereSDF=(x,y,z)=>𝔻['●'](x,y,z,0,0,0,1);
    const nm=𝔻['∇'](lo,li,.2,sphereSDF,.05);
    const shade=𝔻['·'](nm,[sun.x,sun.y,sun.z]);
    const c = {h:CAT_HUE[n.cat]??200, s:62, l:50};
    const litL=c.l*(sun.a+Math.max(0,shade)*sun.i);
    const litS=c.s;

    ctx.save();
    ctx.shadowColor=cc(n.cat,.5);
    ctx.shadowBlur=n.L===0?52:18+n.glow*28;

    // Rings (L0,L1)
    if(n.L<=1){
      [1.85,2.7,3.9].slice(0,n.L===0?3:2).forEach((m,mi)=>{
        ctx.beginPath(); ctx.arc(p.sx,p.sy,Math.max(.5,sr2*m),0,TAU);
        ctx.strokeStyle=cc(n.cat,.032+n.glow*.024+.012*Math.sin(T*1.1+mi));
        ctx.lineWidth=.55; ctx.stroke();
      });
    }

    // Sphere gradient
    const grd=ctx.createRadialGradient(p.sx-sr2*.28,p.sy-sr2*.32,0,p.sx,p.sy,Math.max(.5,sr2));
    grd.addColorStop(0,`hsla(${c.h},${litS}%,${Math.min(94,litL+36)}%,${.84+n.glow*.14})`);
    grd.addColorStop(.45,`hsla(${c.h},${litS}%,${litL}%,${.74+n.glow*.18})`);
    grd.addColorStop(1,`hsla(${c.h},${litS}%,${Math.max(4,litL-32)}%,.05)`);
    ctx.beginPath(); ctx.arc(p.sx,p.sy,Math.max(.5,sr2),0,TAU);
    ctx.fillStyle=grd; ctx.fill();

    // Specular (Phong)
    if(n.L<=2){
      const sR=Math.max(.5,sr2*.75);
      const sg=ctx.createRadialGradient(p.sx-sr2*.32,p.sy-sr2*.36,0,p.sx,p.sy,sR);
      sg.addColorStop(0,`rgba(255,255,255,${.16+shade*.14+n.glow*.08})`);
      sg.addColorStop(1,'rgba(255,255,255,0)');
      ctx.beginPath(); ctx.arc(p.sx,p.sy,sR,0,TAU);
      ctx.fillStyle=sg; ctx.fill();
    }

    // Label
    if(p.scale>.40){
      const fs=n.L===0?sr2*.88:n.L<=2?sr2*.75:sr2*1.02;
      ctx.font=`${Math.max(8,fs)}px monospace`;
      ctx.textAlign='center'; ctx.textBaseline='middle';
      ctx.fillStyle=`hsla(${c.h},40%,95%,${.7+n.glow*.28})`;
      ctx.fillText(n.glyph||n.name||'○',p.sx,p.sy);
    }
    ctx.restore();

    // Decay glow
    if(n.glow>.01) n.glow*=.975;
    else n.glow=0;
  }

  // ── HUD ───────────────────────────────────────────────────
  ctx.save();
  ctx.font='11px monospace';
  ctx.fillStyle='rgba(160,200,255,.55)';
  ctx.fillText(`QR:${STATS.qr} ΔΨ:${STATS.dn} N×T:${STATS.nt}`,12,18);
  ctx.fillText(`☀ ${worldTime.toFixed(1)}h  nodes:${Object.keys(NODES).length}  edges:${EDGES.length}`,12,32);
  ctx.restore();
}

// ── Mouse / touch controls ────────────────────────────────────
function attachControls(cvs){
  let drag=null, lastDist=0, dblTimer=0;
  cvs.addEventListener('mousedown',e=>{
    drag={type:e.button===2?'pan':'orbit',x:e.clientX,y:e.clientY,ct:camTheta,cp:camPhi,px:camPanX,py:camPanY};
    autoSpin=false;
  });
  window.addEventListener('mousemove',e=>{
    if(!drag) return;
    const dx=e.clientX-drag.x, dy=e.clientY-drag.y;
    if(drag.type==='orbit'){ tTheta=drag.ct-dx*.005; tPhi=Math.max(-.5,Math.min(1.2,drag.cp+dy*.005)); }
    else { camPanX=drag.px+dx; camPanY=drag.py+dy; }
  });
  window.addEventListener('mouseup',()=>drag=null);
  cvs.addEventListener('contextmenu',e=>e.preventDefault());
  cvs.addEventListener('wheel',e=>{tDist=Math.max(280,Math.min(1400,tDist+e.deltaY*.55));e.preventDefault();},{passive:false});
  cvs.addEventListener('dblclick',()=>{tTheta=Math.PI/5;tPhi=Math.PI/8;tDist=640;camPanX=camPanY=0;autoSpin=true;});
  cvs.addEventListener('touchstart',e=>{
    if(e.touches.length===2){
      const dx=e.touches[0].clientX-e.touches[1].clientX;
      const dy=e.touches[0].clientY-e.touches[1].clientY;
      lastDist=Math.hypot(dx,dy);
    }
  },{passive:true});
  cvs.addEventListener('touchmove',e=>{
    if(e.touches.length===2){
      const dx=e.touches[0].clientX-e.touches[1].clientX;
      const dy=e.touches[0].clientY-e.touches[1].clientY;
      const d1=Math.hypot(dx,dy);
      tDist=Math.max(280,Math.min(1400,tDist-(d1-lastDist)*1.5));
      lastDist=d1;
    }
  },{passive:true});
}

// ── Public API ─────────────────────────────────────────────────
async function initISDF(cvs){
  canvas = cvs;
  ctx    = cvs.getContext('2d');
  attachControls(cvs);
  // Fetch data
  await fetchTree();
  await fetchEdges();
  // Connect live SSE
  connectSSE();
  // Start render loop
  requestAnimationFrame(frame);
}

// Export
if(typeof module!=='undefined') module.exports={initISDF,updateWorld};
window.initISDF  = initISDF;
window.updateWorld = updateWorld;
