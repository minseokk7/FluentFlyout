import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const roots = ['src', 'src-tauri'];
const suspicious = [/�/, /占/, /媛/, /諛/, /誘몃/, /\?ㅼ/, /\?쒖/];
const files = [];

function walk(dir) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (!['target', 'node_modules', 'dist'].includes(entry.name)) walk(path);
      continue;
    }
    if (/\.(svelte|ts|rs|md|json|toml)$/.test(entry.name) && statSync(path).size < 1_000_000) files.push(path);
  }
}

for (const root of roots) {
  try { walk(root); } catch { /* 없는 폴더는 검사하지 않습니다. */ }
}

const bad = [];
for (const file of files) {
  const text = readFileSync(file, 'utf8');
  for (const pattern of suspicious) {
    if (pattern.test(text)) bad.push(`${file}: ${pattern}`);
  }
}

if (bad.length > 0) {
  console.error('한국어 인코딩이 의심되는 문자열을 발견했습니다.');
  console.error(bad.join('\n'));
  process.exit(1);
}

console.log('한국어 인코딩 검사 통과');
