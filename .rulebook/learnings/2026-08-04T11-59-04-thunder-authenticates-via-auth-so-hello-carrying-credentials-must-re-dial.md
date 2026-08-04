# Thunder authenticates via AUTH, so HELLO-carrying-credentials must re-dial
**Source**: manual
**Date**: 2026-08-04
**Related Task**: phase1_replace-vectorizer-protocol-with-thunder
**Tags**: rpc, thunder, auth, handshake, sdk
Vectorizer's pre-Thunder wire authenticated inside the HELLO command: the server's handle_hello validated the token and flipped per-connection state. Thunder's AuthCommand handshake owns that state instead — it is set by the AUTH frame the client library sends at connect time, and the server's Dispatch::authenticate hook validates it.

Consequence found while migrating: after the server moved to Thunder, handle_hello still validated credentials and still reported authenticated:true in its reply, but the SESSION stayed gated, so the next data-plane command returned NOAUTH. Every SDK's `hello(payload_with_token)` was therefore silently broken against an auth-enabled server.

Fix applied in all five SDKs: hello() re-dials with the credentials in ClientConfig when the payload carries a token or api_key, then issues HELLO on the fresh connection for the capability list. Call sites are unchanged, so this is invisible to users.

Second-order effect worth remembering: `is_authenticated()` now reports the session handshake, so it is FALSE against an open (auth-disabled) server — it authenticates nobody because it gates nothing. Two C# pool tests asserted the old meaning and had to be re-pointed at a working PING instead.