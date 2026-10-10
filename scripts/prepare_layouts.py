#!/usr/bin/env python3
"""Mirror wasm/app.js readLayout so both frontends use the same extracted scenes."""
import json
import re
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def read_layout(text):
    nodes = []
    for match in re.finditer(r'#frames: \[', text):
        prefix = text[:match.start()]
        header = prefix[prefix.rfind('#nodeID:'):]
        ident = re.search(r'#nodeID: "([^"]*)"', header)
        end = text.find('], #children:', match.start())
        block = text[match.start():(end if end > match.start() else text.find(']',match.start()))+1]
        frames = []
        for frame in re.findall(r'\[#[^\]]+\]', block):
            member = re.search(r'#member: "([^"]+)"',frame)
            cast = re.search(r'#cast: "([^"]+)"',frame)
            rect = re.search(r'#rect: rect\(([^)]+)\)',frame)
            z = re.search(r'#z: (-?\d+)',frame)
            anim = re.search(r'#animID: "([^"]*)"',frame)
            if member and cast and rect:
                frames.append(dict(key=cast[1]+'/'+member[1],rect=[float(x) for x in rect[1].split(',')],z=int(z[1]) if z else 0,anim=anim[1] if anim else ''))
        if frames:nodes.append(dict(id=ident[1] if ident else '',frames=frames,**frames[0]))
    return sorted(nodes,key=lambda n:n['z'])
if __name__ == '__main__':
    result={name:read_layout((ROOT/'assets/data/scene_layouts'/file).read_text()) for name,file in [('title','title_screen_layout.txt'),('career','career_setup_layout.txt')]}
    assert result['title'] and result['career']
    (ROOT/'packaging/menu_layouts.json').write_text(json.dumps(result,separators=(',',':'))+'\n')
