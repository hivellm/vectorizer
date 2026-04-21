# API Sandbox

The API Sandbox is an in-dashboard workbench for exercising the
Vectorizer REST API without writing any code. It lives at
**[http://localhost:15002/docs](http://localhost:15002/docs)** under the
"Try it in Sandbox" button on every endpoint.

This page documents the sandbox's day-to-day controls. For the full
endpoint catalog, see [API_REFERENCE.md](../api/API_REFERENCE.md).

---

## Opening the sandbox

1. Start the Vectorizer server (`vectorizer` or `./target/release/vectorizer`).
2. Navigate to the dashboard at `http://localhost:15002/`.
3. Open the **API Documentation** page from the sidebar (`/docs` route).
4. Expand an endpoint and click **Try it in Sandbox** — the modal opens
   pre-populated with the endpoint's example request body.

If you want to jump straight in from the terminal:

```bash
vectorizer docs --sandbox
```

---

## Request builder

Every sandbox invocation shares the same controls:

| Control              | Purpose                                                                 |
|----------------------|-------------------------------------------------------------------------|
| **Authentication**   | Check "Use API Key" to send `X-API-Key: <value>` with the request.      |
| **Path Parameters**  | Fields rendered for every `{param}` in the endpoint path.               |
| **Request Body**     | JSON editor (prefilled with an example) for `POST/PUT/PATCH` methods.   |
| **Send Request**     | Executes the request against the running server and shows the response. |
| **Save / Favorited** | Star button — pins the current request to the endpoint's favorites.    |

### Authentication

When **Use API Key** is toggled on, the sandbox sends the key as
`X-API-Key`. Without a key it hits the unauthenticated endpoints only —
admin and mutating endpoints will return `401 Unauthorized`. Create a
key on the **API Keys** page if you need one.

### Path parameters

The modal lists every `{name}` placeholder from the endpoint's path and
substitutes your input into the URL before sending. Leaving a field
empty sends the literal `:name` in its place so you can see the error.

### Request body

The body editor is a plain `<textarea>`. Text must be valid JSON unless
the endpoint accepts another content type (which is rare). Validation
happens server-side — the sandbox does not block malformed payloads, so
you can reproduce 4xx responses on purpose.

---

## Response panel

The response tab shows the returned HTTP status, round-trip time, and
body with syntax highlighting. Non-2xx responses are tagged in red so
failures are obvious at a glance. Switch tabs to view the same request
rendered as:

- **cURL** — drop-in command line for reproducing the call.
- **TypeScript** — `fetch()` snippet, async/await.
- **Python** — `requests` library call.
- **Rust** — `reqwest` async client.
- **Go** — `net/http` client.

Each snippet has a **Copy** button in the top-right corner.

---

## History and favorites

The sandbox remembers every successful or failing request **per endpoint**
in your browser's `localStorage`. Click the clock icon in the modal
header to expand the side panel; it shows up to 25 recent entries plus
your starred favorites.

- **Favorites** survive indefinitely and are keyed by
  `(method, path, body)`, so editing the body and re-starring creates a
  new favorite rather than overwriting the old one.
- **Recent** is capped at 25 entries per endpoint and prunes oldest-first.
  Click **Clear all** to wipe the history without touching favorites.
- Clicking a history or favorite entry loads its path parameters and
  body into the modal so you can re-run it or tweak it.
- Favorites and history live in
  `vectorizer.sandbox.favorites.v1` /
  `vectorizer.sandbox.history.v1` keys — clear them from DevTools if
  you need a full reset.

### Privacy note

Nothing in the sandbox is persisted server-side. History and favorites
never leave the browser that typed them. If you need cross-device
portability, copy the cURL or code snippet and save it in your own
notes.

---

## Troubleshooting

### "No API keys found."

You ticked **Use API Key** before creating one. Visit the **API Keys**
page in the dashboard, create a key with the scopes you need, then
paste it into the field.

### "Request failed" with no status

The fetch never reached the server — usually CORS, a crashed server, or
a typo in the path template. Open DevTools → Network to see the raw
request.

### "HELLO must succeed before any data-plane command"

This is an RPC-only error. The sandbox uses the REST endpoints, so if
you see it, a downstream SDK is talking to the binary port. Point the
sandbox at the REST host (default `http://localhost:15002`) instead of
`vectorizer://…`.

### Sandbox is ignoring my path parameter

Make sure the field name matches the `{placeholder}` in the template
*exactly* — it's case-sensitive. A missing field leaves `:name` in the
URL, which is visible in the rendered cURL snippet.

---

## See also

- [Setup Wizard](SETUP_WIZARD.md)
- [API Reference](../api/API_REFERENCE.md)
- [Authentication](../api/AUTHENTICATION.md)
