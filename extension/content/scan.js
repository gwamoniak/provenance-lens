// Page-scan content script (U7). Registered by the background PER GRANTED
// ORIGIN via scripting.registerContentScripts — it never appears in the
// manifest, so its reach is exactly the user's grants at any moment.
//
// U7a ships this file deliberately inert so the registration plumbing is
// real and testable; the scanning logic (IntersectionObserver + Mutation-
// Observer, min-size filter, shadow-DOM verdict pills) lands in U7b. The
// background's verify service is already live: send
// { type: "pl-verify", url } and receive the report entry
// { srcUrl, at, anchorsLoaded?, report? | error? }.
