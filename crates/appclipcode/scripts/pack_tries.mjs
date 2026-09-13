import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import zlib from "node:zlib";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const dataDir = path.resolve(__dirname, "../data");

const files = [
  { name: "h", symbols: Array.from("-.0123456789abcdefghijklmnopqrstuvwxyz|") },
  { name: "cpq", symbols: Array.from("#%&+,-./0123456789:;=?ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz") },
  { name: "spq", symbols: Array.from("&+-./0123456789=?ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz|") }
];

class BitWriter {
  constructor() {
    this.bytes = [];
    this.currentByte = 0;
    this.bitsInCurrentByte = 0;
  }

  writeBit(bit) {
    this.currentByte = (this.currentByte << 1) | (bit ? 1 : 0);
    this.bitsInCurrentByte += 1;
    if (this.bitsInCurrentByte === 8) {
      this.bytes.push(this.currentByte);
      this.currentByte = 0;
      this.bitsInCurrentByte = 0;
    }
  }

  writeBits(value, count) {
    for (let i = count - 1; i >= 0; i -= 1) {
      this.writeBit(((value >> i) & 1) === 1);
    }
  }

  finish() {
    if (this.bitsInCurrentByte > 0) {
      this.currentByte <<= 8 - this.bitsInCurrentByte;
      this.bytes.push(this.currentByte);
    }
    return Buffer.from(this.bytes);
  }
}

function compareNodes(a, b) {
  if (a.freq !== b.freq) return a.freq - b.freq;
  if (a.leftmost < b.leftmost) return -1;
  if (a.leftmost > b.leftmost) return 1;
  return 0;
}

function buildTree(frequencies, symbols) {
  const leaves = frequencies
    .map((freq, symbolIndex) => ({
      freq,
      symbolIndex,
      leftmost: symbols[symbolIndex] ?? ""
    }))
    .filter(n => n.freq > 0);

  const nodes = leaves;
  while (nodes.length > 1) {
    nodes.sort(compareNodes);
    const left = nodes.shift();
    const right = nodes.shift();
    nodes.push({
      freq: left.freq + right.freq,
      symbolIndex: -1,
      left,
      right,
      leftmost: left.leftmost
    });
  }
  return nodes[0];
}

function writeTree(shapeWriter, leafWriter, node, symbolIndexBits) {
  if (!node.left && !node.right) {
    shapeWriter.writeBit(true);
    leafWriter.writeBits(node.symbolIndex, symbolIndexBits);
    return;
  }
  shapeWriter.writeBit(false);
  writeTree(shapeWriter, leafWriter, node.left, symbolIndexBits);
  writeTree(shapeWriter, leafWriter, node.right, symbolIndexBits);
}

for (const f of files) {
  const rawPath = path.join(dataDir, `${f.name}.data`);
  const data = fs.readFileSync(rawPath);
  const numSymbols = f.symbols.length;
  const nodeCount = 1 + numSymbols + numSymbols * numSymbols;
  const rowSize = numSymbols * 2;
  const symbolIndexBits = Math.ceil(Math.log2(numSymbols));

  const shapeWriter = new BitWriter();
  const leafWriter = new BitWriter();

  for (let i = 0; i < nodeCount; i++) {
    const freqs = [];
    for (let j = 0; j < numSymbols; j++) {
      freqs.push((data[i * rowSize + j * 2] << 8) | data[i * rowSize + j * 2 + 1]);
    }
    const tree = buildTree(freqs, f.symbols);
    writeTree(shapeWriter, leafWriter, tree, symbolIndexBits);
  }

  const packed = Buffer.concat([shapeWriter.finish(), leafWriter.finish()]);
  const compressed = zlib.deflateSync(packed, { level: 9 });
  const outPath = path.join(dataDir, `${f.name}.trie.zlib`);
  fs.writeFileSync(outPath, compressed);
  console.log(`Packed ${f.name}: raw ${data.length}B -> packed ${packed.length}B -> zlib ${compressed.length}B`);
}
