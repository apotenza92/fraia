#!/usr/bin/env python3
"""Build a self-contained browser viewer for the Fraia markdown knowledge wiki."""
from __future__ import annotations

import json
import re
import shutil
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
KNOWLEDGE = ROOT / "docs" / "knowledge"
OUT = KNOWLEDGE / "viewer.html"


def front_matter(text: str) -> tuple[dict[str, str], str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end == -1:
        return {}, text
    meta: dict[str, str] = {}
    for line in text[4:end].splitlines():
        if line and not line.startswith(" ") and ":" in line:
            k, v = line.split(":", 1)
            meta[k.strip()] = v.strip().strip('"')
    return meta, text[end + 5 :]


def title_from(text: str, rel: str) -> str:
    meta, body = front_matter(text)
    if meta.get("title"):
        return meta["title"]
    for line in body.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return rel


def kind_for(rel: str) -> str:
    if rel.startswith("raw/"):
        return "raw"
    if rel.startswith("proposals/"):
        return "proposal"
    if rel.startswith("wiki/"):
        return "wiki"
    return "meta"


def collect_pages() -> list[dict[str, object]]:
    pages = []
    for path in sorted(KNOWLEDGE.rglob("*.md")):
        rel = path.relative_to(KNOWLEDGE).as_posix()
        text = path.read_text()
        meta, _ = front_matter(text)
        pages.append({"path": rel, "title": title_from(text, rel), "kind": kind_for(rel), "meta": meta, "text": text})
    return pages


def build_html(pages: list[dict[str, object]]) -> str:
    generated = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    payload = json.dumps({"generated": generated, "pages": pages}, ensure_ascii=False).replace("<", "\\u003c")
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/>
<title>Fraia Knowledge Wiki</title>
<style>
:root {{ color-scheme: light dark; --bg:#eef3f8; --panel:#f8fafc; --text:#172033; --muted:#60708a; --border:#d8e0ea; --accent:#2563eb; --code:#e7edf5; --badge:#dbeafe; }}
@media (prefers-color-scheme: dark) {{ :root {{ --bg:#07111f; --panel:#0d1726; --text:#e5edf7; --muted:#94a3b8; --border:#223044; --accent:#60a5fa; --code:#111d2e; --badge:#172554; }} }}
* {{ box-sizing:border-box; }} body {{ margin:0; background:var(--bg); color:var(--text); font:14px/1.55 Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif; }}
.app {{ display:grid; grid-template-columns:380px minmax(0,1fr); height:100vh; }} aside {{ border-right:1px solid var(--border); background:var(--panel); display:flex; flex-direction:column; min-height:0; }}
header {{ padding:22px 20px 14px; border-bottom:1px solid var(--border); }} h1 {{ margin:0; font-size:18px; }} .brand {{ margin:0; padding:0; border:0; background:transparent; color:var(--text); font:inherit; font-size:18px; font-weight:700; cursor:pointer; }} .brand:hover {{ color:var(--accent); }} .home-row {{ display:flex; align-items:center; justify-content:space-between; gap:10px; }} .home-pill {{ border:1px solid var(--border); border-radius:999px; color:var(--muted); background:transparent; font-size:11px; padding:3px 8px; cursor:pointer; }} .home-pill:hover {{ color:var(--accent); border-color:var(--accent); }} .sub {{ color:var(--muted); font-size:12px; margin-top:4px; }} .search {{ padding:14px; }}
input {{ width:100%; border:1px solid var(--border); background:transparent; color:var(--text); border-radius:10px; padding:10px 12px; font:inherit; }} nav {{ overflow:auto; padding:0 10px 14px; }}
.group {{ color:var(--muted); text-transform:uppercase; letter-spacing:.08em; font-size:11px; margin:14px 10px 6px; }} button.page {{ width:100%; border:0; background:transparent; color:var(--text); text-align:left; padding:8px 10px; border-radius:10px; cursor:pointer; }}
button.page:hover,button.page.active {{ background:color-mix(in srgb,var(--accent) 12%,transparent); }} .title {{ display:block; font-weight:650; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }} .path {{ display:block; color:var(--muted); font-size:11px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }}
.badges {{ display:flex; gap:4px; flex-wrap:wrap; margin-top:4px; }} .badge {{ font-size:10px; color:var(--muted); background:var(--badge); border:1px solid var(--border); border-radius:999px; padding:1px 6px; }}
main {{ min-width:0; overflow:auto; padding:36px min(8vw,90px); }} article {{ max-width:960px; margin:0 auto; }} article h1 {{ font-size:34px; margin:0 0 18px; }} article h2 {{ margin:34px 0 10px; padding-top:10px; border-top:1px solid var(--border); font-size:21px; }} article h3 {{ margin:24px 0 8px; font-size:16px; }} article a {{ color:var(--accent); }} article code {{ background:var(--code); padding:2px 5px; border-radius:5px; }} pre,.frontmatter {{ background:var(--code); border:1px solid var(--border); border-radius:12px; padding:12px; overflow:auto; }} .meta-strip {{ display:flex; flex-wrap:wrap; gap:6px; margin:0 0 18px; }} ul {{ padding-left:24px; }} blockquote {{ border-left:3px solid var(--accent); margin:12px 0; padding:8px 14px; color:var(--muted); }}
</style></head><body><div class="app"><aside><header><div class="home-row"><h1><button id="homeTitle" class="brand" title="Go to wiki home">Fraia Knowledge Wiki</button></h1><button id="homeButton" class="home-pill" title="Go to wiki home">Home</button></div><div class="sub">Generated {generated}. Markdown is source of truth.</div></header><div class="search"><input id="search" placeholder="Search topics…"/></div><nav id="nav"></nav></aside><main><article id="content"></article></main></div>
<script id="payload" type="application/json">{payload}</script>
<script>
const data = JSON.parse(document.getElementById('payload').textContent); const pages = data.pages; const nav = document.getElementById('nav'); const content = document.getElementById('content'); const search = document.getElementById('search'); const homePath = (pages.find(p=>p.path==='index.md')||pages[0]).path;
let active = location.hash ? decodeURIComponent(location.hash.slice(1)) : homePath;
function esc(s) {{ return String(s||'').replace(/[&<>]/g,c=>({{'&':'&amp;','<':'&lt;','>':'&gt;'}}[c])); }}
function pageForLink(from,target) {{ if (!target || /^(https?:|mailto:|#)/.test(target)) return null; const base=from.split('/').slice(0,-1); for (const part of target.split('#')[0].split('/')) {{ if (!part||part==='.') continue; if (part==='..') base.pop(); else base.push(part); }} return base.join('/'); }}
function inline(s,from) {{ return esc(s).replace(/`([^`]+)`/g,'<code>$1</code>').replace(/\\*\\*([^*]+)\\*\\*/g,'<strong>$1</strong>').replace(/\\[([^\\]]+)\\]\\(([^)]+)\\)/g,(m,t,u)=>{{ const p=pageForLink(from,u); return p && pages.find(x=>x.path===p) ? `<a href="#${{encodeURIComponent(p)}}">${{t}}</a>` : `<a href="${{u}}">${{t}}</a>`; }}); }}
function splitFm(md) {{ if (!md.startsWith('---\\n')) return ['',md]; const end=md.indexOf('\\n---\\n',4); return end<0?['',md]:[md.slice(4,end),md.slice(end+5)]; }}
function renderMd(page) {{ let [fm,md]=splitFm(page.text); const out=[]; const m=page.meta||{{}}; out.push('<div class="meta-strip">'+['kind:'+page.kind,m.status&&'status:'+m.status,m.trust_level&&'trust:'+m.trust_level,m.source_count&&'sources:'+m.source_count,m.last_compiled&&'compiled:'+m.last_compiled].filter(Boolean).map(x=>'<span class="badge">'+esc(x)+'</span>').join('')+'</div>'); if(fm) out.push('<details><summary>Page metadata</summary><div class="frontmatter">'+esc(fm)+'</div></details>'); let inList=false,inCode=false; for(const raw of md.split(/\\r?\\n/)) {{ const line=raw.trimEnd(); if(line.startsWith('```')) {{ out.push(inCode?'</code></pre>':'<pre><code>'); inCode=!inCode; continue; }} if(inCode) {{ out.push(esc(raw)+'\\n'); continue; }} if(!line.trim()) {{ if(inList) {{ out.push('</ul>'); inList=false; }} continue; }} if(line.startsWith('# ')) {{ if(inList){{out.push('</ul>');inList=false;}} out.push('<h1>'+inline(line.slice(2),page.path)+'</h1>'); continue; }} if(line.startsWith('## ')) {{ if(inList){{out.push('</ul>');inList=false;}} out.push('<h2>'+inline(line.slice(3),page.path)+'</h2>'); continue; }} if(line.startsWith('### ')) {{ if(inList){{out.push('</ul>');inList=false;}} out.push('<h3>'+inline(line.slice(4),page.path)+'</h3>'); continue; }} if(line.startsWith('- ')) {{ if(!inList){{out.push('<ul>');inList=true;}} out.push('<li>'+inline(line.slice(2),page.path)+'</li>'); continue; }} if(line.startsWith('|')) {{ out.push('<pre>'+esc(line)+'</pre>'); continue; }} if(inList){{out.push('</ul>');inList=false;}} out.push('<p>'+inline(line,page.path)+'</p>'); }} if(inList) out.push('</ul>'); if(inCode) out.push('</code></pre>'); return out.join('\\n'); }}
function groups(items) {{ const order=['wiki','meta','proposal','raw']; return order.map(k=>[k,items.filter(p=>p.kind===k)]).filter(x=>x[1].length); }}
function depth(p) {{ return Math.max(0,p.path.split('/').length-1); }}
function navButton(p) {{ const b=[p.meta?.status,p.meta?.trust_level,p.meta?.source_count&&'S'+p.meta.source_count].filter(Boolean).map(x=>`<span class="badge">${{esc(x)}}</span>`).join(''); return `<button class="page ${{p.path===active?'active':''}}" style="padding-left:${{10+depth(p)*14}}px" data-path="${{esc(p.path)}}"><span class="title">${{esc(p.title)}}</span><span class="path">${{esc(p.path)}}</span><span class="badges">${{b}}</span></button>`; }}
function renderNav() {{ const q=search.value.toLowerCase(); const filtered=pages.filter(p=>!q||(p.title+' '+p.path+' '+p.text).toLowerCase().includes(q)); nav.innerHTML=groups(filtered).map(([k,items])=>'<div class="group">'+k+'</div>'+items.map(navButton).join('')).join(''); nav.querySelectorAll('button.page').forEach(btn=>btn.addEventListener('click',()=>{{ active=btn.dataset.path; location.hash=encodeURIComponent(active); render(); }})); }}
function render() {{ const page=pages.find(p=>p.path===active)||pages[0]; active=page.path; document.title=page.title+' · Fraia Knowledge Wiki'; content.innerHTML=renderMd(page); renderNav(); }}
function goHome() {{ active=homePath; location.hash=encodeURIComponent(homePath); render(); }}
document.getElementById('homeTitle').addEventListener('click',goHome); document.getElementById('homeButton').addEventListener('click',goHome); search.addEventListener('input',renderNav); window.addEventListener('hashchange',()=>{{active=decodeURIComponent(location.hash.slice(1)); render();}}); render();
</script></body></html>
"""


def validate(html: str) -> None:
    payload = re.search(r'<script id="payload" type="application/json">(.*?)</script>', html, re.S)
    if not payload:
        raise RuntimeError("viewer payload script missing")
    json.loads(payload.group(1))
    scripts = re.findall(r"<script(?: [^>]*)?>(.*?)</script>", html, re.S)
    if shutil.which("node") and len(scripts) >= 2:
        tmp = Path("/tmp/fraia-knowledge-viewer.js")
        tmp.write_text(scripts[-1])
        subprocess.run(["node", "--check", str(tmp)], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)


def main() -> int:
    pages = collect_pages()
    html = build_html(pages)
    validate(html)
    OUT.write_text(html)
    print(f"Wrote {OUT.relative_to(ROOT)} with {len(pages)} pages")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
