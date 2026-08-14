const fs = require('node:fs');
const path = require('node:path');

const output = path.join(__dirname, 'scanned-architectural-drawing.pdf');
const raster = path.join(__dirname, 'scanned-architectural-drawing-raster.jpg');
const width = 1200;
const height = 800;
const jpeg = fs.readFileSync(raster);
const pageContent = 'q 1200 0 0 800 0 0 cm /Im0 Do Q';

const objects = [
  '<< /Type /Catalog /Pages 2 0 R >>',
  '<< /Type /Pages /Kids [3 0 R] /Count 1 >>',
  '<< /Type /Page /Parent 2 0 R /MediaBox [0 0 1200 800] /Rotate 90 /Resources << /XObject << /Im0 5 0 R >> >> /Contents 4 0 R >>',
  `<< /Length ${Buffer.byteLength(pageContent)} >>\nstream\n${pageContent}\nendstream`,
  Buffer.concat([
    Buffer.from(`<< /Type /XObject /Subtype /Image /Width ${width} /Height ${height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length ${jpeg.length} >>\nstream\n`),
    jpeg,
    Buffer.from('\nendstream'),
  ]),
];
const chunks = [Buffer.from('%PDF-1.4\n%Fraia deterministic scanned fixture\n')];
const offsets = [0];
for (const [index, object] of objects.entries()) {
  offsets.push(chunks.reduce((sum, chunk) => sum + chunk.length, 0));
  chunks.push(Buffer.from(`${index + 1} 0 obj\n`));
  chunks.push(Buffer.isBuffer(object) ? object : Buffer.from(object));
  chunks.push(Buffer.from('\nendobj\n'));
}
const xref = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
chunks.push(Buffer.from(`xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`));
for (const offset of offsets.slice(1)) chunks.push(Buffer.from(`${String(offset).padStart(10, '0')} 00000 n \n`));
chunks.push(Buffer.from(`trailer\n<< /Size ${objects.length + 1} /Root 1 0 R /ID [<46726169614f4352><46726169614f4352>] >>\nstartxref\n${xref}\n%%EOF\n`));
fs.writeFileSync(output, Buffer.concat(chunks));
