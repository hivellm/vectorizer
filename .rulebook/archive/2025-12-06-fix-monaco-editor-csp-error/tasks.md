## 1. Update Content Security Policy

- [x] 1.1 Locate CSP configuration in src/server/mod.rs (security_headers_middleware function)
- [x] 1.2 Add `https://cdn.jsdelivr.net` to script-src directive
- [x] 1.3 Add `worker-src` directive if needed for Monaco workers
- [x] 1.4 Test CSP header is correctly set

## 2. Verify Monaco Editor Loading

- [ ] 2.1 Test dashboard loads without CSP errors
- [ ] 2.2 Verify Monaco Editor initializes correctly
- [ ] 2.3 Check console for any remaining CSP violations
- [ ] 2.4 Test editor functionality (syntax highlighting, editing, etc.)

## 3. Testing

- [ ] 3.1 Test in development mode
- [ ] 3.2 Test in production build
- [ ] 3.3 Verify no CSP errors in browser console
- [ ] 3.4 Test with different file types (JSON, Markdown, etc.)
- [ ] 3.5 Verify editor features work (syntax highlighting, auto-complete, etc.)

## 4. Documentation (Optional)

- [ ] 4.1 Document CSP configuration if needed
- [ ] 4.2 Add note about Monaco Editor CDN dependency if relevant
