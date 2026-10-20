const fs = require('fs');
const cp = require('child_process');
const path = require('path');

const root = path.resolve(__dirname, '..');
const frontendDir = path.join(root, 'frontend');
const distHtml = path.join(frontendDir, 'dist', 'index.html');

if (fs.existsSync(distHtml)) {
  console.log('[build-frontend] Frontend dist already built at', distHtml, '- skipping rebuild.');
  process.exit(0);
}

console.log('[build-frontend] Building frontend in:', frontendDir);
cp.execSync('npm run build', { cwd: frontendDir, stdio: 'inherit' });
