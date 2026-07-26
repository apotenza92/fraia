import type { ReactNode } from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

type MessageBlock =
  | { kind: 'heading'; level: 2 | 3; text: string }
  | { kind: 'paragraph'; lines: string[] }
  | { kind: 'bulletList'; items: string[] }
  | { kind: 'orderedList'; items: string[] }
  | { kind: 'table'; rows: string[][] };

function splitTableRow(line: string) {
  return line
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map((cell) => cell.trim());
}

function isTableDivider(line: string) {
  return /^\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?$/.test(line);
}

function messageBlocks(text: string): MessageBlock[] {
  const blocks: MessageBlock[] = [];
  let paragraph: string[] = [];
  let bulletItems: string[] = [];
  let orderedItems: string[] = [];
  let tableRows: string[][] = [];
  function flushParagraph() {
    if (!paragraph.length) return;
    blocks.push({ kind: 'paragraph', lines: paragraph });
    paragraph = [];
  }
  function flushBulletList() {
    if (!bulletItems.length) return;
    blocks.push({ kind: 'bulletList', items: bulletItems });
    bulletItems = [];
  }
  function flushOrderedList() {
    if (!orderedItems.length) return;
    blocks.push({ kind: 'orderedList', items: orderedItems });
    orderedItems = [];
  }
  function flushTable() {
    if (!tableRows.length) return;
    blocks.push({ kind: 'table', rows: tableRows });
    tableRows = [];
  }
  function flushAll() {
    flushParagraph();
    flushBulletList();
    flushOrderedList();
    flushTable();
  }

  for (const rawLine of text.trim().split('\n')) {
    const line = rawLine.trim();
    const bullet = line.match(/^[-*]\s+(.+)$/);
    const ordered = line.match(/^\d+[.)]\s+(.+)$/);
    const heading = line.match(/^(#{2,3})\s+(.+)$/);
    const boldHeading = line.match(/^\*\*(.+?)\*\*:?$/);
    if (!line) {
      flushAll();
      continue;
    }
    if (heading || boldHeading) {
      flushAll();
      blocks.push({
        kind: 'heading',
        level: heading?.[1].length === 3 ? 3 : 2,
        text: (heading?.[2] ?? boldHeading?.[1] ?? '').trim(),
      });
      continue;
    }
    if (line.includes('|') && !isTableDivider(line)) {
      flushParagraph();
      flushBulletList();
      flushOrderedList();
      tableRows.push(splitTableRow(line));
      continue;
    }
    if (isTableDivider(line)) continue;
    if (bullet) {
      flushParagraph();
      flushOrderedList();
      flushTable();
      bulletItems.push(bullet[1]);
      continue;
    }
    if (ordered) {
      flushParagraph();
      flushBulletList();
      flushTable();
      orderedItems.push(ordered[1]);
      continue;
    }
    flushBulletList();
    flushOrderedList();
    flushTable();
    paragraph.push(line);
  }

  flushAll();
  return blocks;
}

function inlineMarkdown(text: string): ReactNode[] {
  const nodes: ReactNode[] = [];
  const pattern = /(`[^`]+`|\*\*[^*]+\*\*)/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(text))) {
    if (match.index > lastIndex) nodes.push(text.slice(lastIndex, match.index));
    const token = match[0];
    if (token.startsWith('`')) {
      nodes.push(
        <code key={`${match.index}-code`} className="rounded bg-muted px-1 py-0.5 font-mono text-[0.9em]">
          {token.slice(1, -1)}
        </code>,
      );
    } else {
      nodes.push(<strong key={`${match.index}-bold`}>{token.slice(2, -2)}</strong>);
    }
    lastIndex = pattern.lastIndex;
  }
  if (lastIndex < text.length) nodes.push(text.slice(lastIndex));
  return nodes;
}

export function ChatMessageText({ text }: { text: string }) {
  return (
    <div className="flex flex-col gap-2">
      {messageBlocks(text).map((block, index) => {
        if (block.kind === 'heading') {
          const HeadingTag = block.level === 3 ? 'h3' : 'h2';
          return <HeadingTag key={index} className="font-semibold">{inlineMarkdown(block.text)}</HeadingTag>;
        }
        if (block.kind === 'bulletList') {
          return (
            <ul key={index} className="list-disc space-y-1 pl-5 text-sm">
              {block.items.map((item, itemIndex) => <li key={itemIndex}>{inlineMarkdown(item)}</li>)}
            </ul>
          );
        }
        if (block.kind === 'orderedList') {
          return (
            <ol key={index} className="list-decimal space-y-1 pl-5 text-sm">
              {block.items.map((item, itemIndex) => <li key={itemIndex}>{inlineMarkdown(item)}</li>)}
            </ol>
          );
        }
        if (block.kind === 'table') {
          const [head, ...body] = block.rows;
          return (
            <div key={index} className="overflow-auto rounded-md border">
              <Table>
                {head && (
                  <TableHeader>
                    <TableRow>
                      {head.map((cell, cellIndex) => (
                        <TableHead key={cellIndex}>
                          {inlineMarkdown(cell)}
                        </TableHead>
                      ))}
                    </TableRow>
                  </TableHeader>
                )}
                <TableBody>
                  {body.map((row, rowIndex) => (
                    <TableRow key={rowIndex}>
                      {row.map((cell, cellIndex) => (
                        <TableCell key={cellIndex}>
                          {inlineMarkdown(cell)}
                        </TableCell>
                      ))}
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          );
        }
        return (
          <p key={index} className="text-sm">
            {block.lines.map((line, lineIndex) => (
              <span key={`${line}-${lineIndex}`}>
                {lineIndex > 0 ? <br /> : null}
                {inlineMarkdown(line)}
              </span>
            ))}
          </p>
        );
      })}
    </div>
  );
}
