# Claude marks: what Provenance Lens can check

Capability snapshot: 2026-09-17, N3 candidate. This documents local implementation and fixtures, not new vendor access. The official article [How Claude marks AI-generated content](https://support.claude.com/en/articles/16266773-how-claude-marks-ai-generated-content) was inspected in August 2026; its current availability claims have not been refreshed here.

Claude's exported-file Content Credentials and its text watermark are different mechanisms. A signed statement may describe processing, including proofreading, translation, or conversion; it does not establish original authorship of every part. Missing credentials or watermark signals never establish human authorship.

| Evidence or check | CLI | Standalone WASM | Chrome / Firefox |
|---|---|---|---|
| JPEG/PNG C2PA parsing and generic signed/unsigned/tampered fixtures | Tested | Tested | Same bundled engine; browser harness tested |
| SVG C2PA with explicit MIME/filename hint | Tested | Tested with MIME | Response MIME or local-file hint; no arbitrary XML sniffing |
| Trusted signed watermark declaration | Descriptive claims | Same JSON | Same descriptive claims; not pixel detection |
| SD DWT-DCT pixel watermark | Native detector | Not compiled | Not compiled |
| IMATAG bzh pixel watermark | Requires the exact calibrated local model | Not compiled | Not compiled |
| Genuine Anthropic-signed positive specimen | Not demonstrated | Not demonstrated | Not demonstrated |
| Claude text watermark / actual SynthID decoder | Not implemented | Not implemented | Not implemented |
| PDF/audio/video/other exported formats | No project acceptance corpus or support claim | Same limitation | Same limitation |

JPEG/PNG/SVG generic positive fixtures are generated locally with a test-only certificate. They prove the generic path, not Anthropic interoperability. The six retained Claude flower SVG specimens, including three with visible AI-generated labels and ordinary metadata, have no C2PA manifest and produce C2PA `no_signal`. Editable labels are not signatures. Locally signing them or converting them to PNG cannot create vendor evidence.

Verified means the provenance chain validated to the supplied anchors, not that the image is truthful or human-made. A cryptographically valid signature from an untrusted issuer currently maps to Tampered with an explanatory detail; this is the project's conservative trust policy, not proof that the pixels were edited. Reader acceptance of a MIME type alone is not a successful signed-format or vendor test.

For an explicit watermark action, the exact note is: “the verified credential records an invisible watermark; the watermark was not independently detected”. Signed soft-binding algorithms are shown separately: a fingerprint and watermark action can coexist without proving that the algorithm names the watermark scheme. A soft binding alone never creates this declaration, and a declaration never becomes a Layer-2 hit.

C4's useful cross-surface documentation is delivered here. Its conditional text-detector integration and release acceptance remain open. C0-positive requires a genuine, permission-to-retain Anthropic-signed specimen and recorded trust outcome. C2/C3 and W3-D require an implementable vendor detector, calibration, and any separately approved privacy exception. No issuer-name heuristic, writing-style classifier, account, vendor API, or upload was added.
