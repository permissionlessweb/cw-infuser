const fs = require('fs');
const path = require('path');

function toPascalCase(str) {
  return str
    .split(/[-_]/)
    .map(word => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join('');
}

const rootDir = path.resolve(__dirname);
const contractsDir = path.join(rootDir, 'contracts');
const outputDir = path.join(rootDir, 'scripts', 'ts');
const outputFile = path.join(outputDir, 'contracts.generated.json');

console.log('🔍 Scanning for contract schemas under contracts/...');

const schemaFiles = [];
function scan(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      scan(fullPath);
    } else if (entry.name.endsWith('.json') && path.dirname(fullPath).endsWith('schema')) {
      schemaFiles.push(fullPath);
    }
  }
}
scan(contractsDir);

schemaFiles.sort();

const CONTRACTS = [];

for (const jsonPath of schemaFiles) {
  const outName = path.basename(jsonPath, '.json');
  const schemaDir = path.dirname(jsonPath);           // e.g. contracts/cw-infuser/schema
  const name = toPascalCase(outName);

  // Relative path from scripts/ts/ → e.g. '../../contracts/cw-infuser/schema'
  const relativeDir = path.relative(
    path.join(rootDir, 'scripts', 'ts'),
    schemaDir
  ).replace(/\\/g, '/');   // normalize slashes for TS/JSON

  CONTRACTS.push({
    name,
    dir: relativeDir,
    entryFiles: [
      `${name}.types.ts`,
      `${name}.client.ts`,
      `${name}.message-composer.ts`,
    ],
    globalName: name,
    outName,
  });
}

fs.mkdirSync(outputDir, { recursive: true });
fs.writeFileSync(outputFile, JSON.stringify(CONTRACTS, null, 2));

console.log(`✅ Generated ${CONTRACTS.length} contract entries → scripts/ts/contracts.generated.json`);
for (const c of CONTRACTS) {
  console.log(`   • ${c.name} (${c.outName})`);
}