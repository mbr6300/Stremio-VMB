/**
 * Rendert Text mit **text** als fett formatiert.
 */
export default function FormatBoldText({ text }: { text: string }) {
  const parts: (string | { bold: string })[] = [];
  let remaining = text;
  while (remaining.length > 0) {
    const start = remaining.indexOf("**");
    if (start === -1) {
      parts.push(remaining);
      break;
    }
    const before = remaining.slice(0, start);
    if (before) parts.push(before);
    const afterStart = remaining.slice(start + 2);
    const end = afterStart.indexOf("**");
    if (end === -1) {
      parts.push("**" + afterStart);
      break;
    }
    parts.push({ bold: afterStart.slice(0, end) });
    remaining = afterStart.slice(end + 2);
  }
  return (
    <>
      {parts.map((p, i) =>
        typeof p === "string" ? (
          <span key={i}>{p}</span>
        ) : (
          <strong key={i}>{p.bold}</strong>
        )
      )}
    </>
  );
}
