// Shared presentation of signed claims. All callers render values as text.
export function credentialLines(report) {
  const c = report.verdict === "verified" ? report.credentials : null;
  if (!c) return [];
  const lines = [
    ["credential claims", ""], ["issuer", c.issuer],
    ["claim generator", c.claim_generator], ["signed at", c.signing_time],
    ["declared source type", c.digital_source_type], ["note", c.source_type_note],
  ];
  if (c.declared_watermark) {
    const w = c.declared_watermark;
    lines.push(["note", w.note]);
    for (const action of w.action_labels) lines.push(["watermark action", action]);
    for (const algorithm of w.algorithms)
      lines.push(["signed soft-binding algorithm (not necessarily the watermark)", algorithm]);
    if (!w.algorithms.length) lines.push(["watermark scheme", "not established by the available signed context"]);
  }
  return lines.filter(([, value]) => value !== undefined && value !== null);
}

export function appendLines(list, lines) {
  for (const [label, value] of lines) {
    const item = document.createElement("li");
    item.textContent = `${label}: ${value}`;
    list.appendChild(item);
  }
}
