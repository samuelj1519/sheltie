import json,re,hashlib,subprocess
from pathlib import Path
root=Path('/private/tmp/sheltie-completion-20261003')
rows=[]
for v in ['0140','0145']:
 p=root/f'runner-pipe-{v}.txt';s=p.read_text();m=re.search(r'Summary \[[^\]]+\] ([0-9]+) tests run: ([0-9]+) passed(?: \(([^)]+)\))?, ([0-9]+) skipped',s)
 assert m and m.group(1)==m.group(2)=='4096' and m.group(4)=='0'
 rows.append({'version':'0.9.140' if v=='0140' else '0.9.145','run_id':re.search(r'Nextest run ID ([\w-]+)',s).group(1),'tests':4096,'passed':4096,'skipped':0,'leaky':int(re.search(r'(\d+) leaky',m.group(3) or '').group(1)) if 'leaky' in (m.group(3) or '') else 0,'raw_file':str(p),'raw_sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
x={'fixture':'runner-pipe-probe','same_source_sha256':hashlib.sha256((root/'runner-pipe-probe/src/lib.rs').read_bytes()).hexdigest(),'threads':64,'comparison':rows,'conclusion':'The installed older runner reproduces capture-handle leak reports with ordinary completed test processes; required version has zero reports for same fixture. Official release fixes this class. Historical individual Sheltie LEAK causality is not retrospectively proven.'}
(root/'runner-pipe-comparison.json').write_text(json.dumps(x,indent=2)+'\n');print(json.dumps(rows))
