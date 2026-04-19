package rpc

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

// Errors that the Client can return. Use errors.Is to discriminate.
var (
	// ErrServer wraps the message returned in a server-side
	// Result::Err. The wrapped string is the human-readable message;
	// v1 of the protocol does not carry a structured error code.
	ErrServer = errors.New("server error")

	// ErrConnectionClosed indicates the reader exited before the
	// response arrived. The connection is unusable; build a new one.
	ErrConnectionClosed = errors.New("connection closed before response")

	// ErrNotAuthenticated indicates a data-plane command was issued
	// before HELLO succeeded. The local gate fires first to save a
	// network round-trip; the server would also reject this.
	ErrNotAuthenticated = errors.New(
		"HELLO must succeed before any data-plane command can be issued",
	)
)

// HelloPayload is sent as the FIRST frame on a connection.
//
// At least one of Token / APIKey should be populated when the server
// has auth enabled. When the server runs in single-user mode
// (auth.enabled: false), credentials are accepted-but-ignored and
// the connection runs as the implicit local admin.
type HelloPayload struct {
	ClientName string
	Token      string
	APIKey     string
	// Version is the wire-spec protocol version. Defaults to 1 when zero.
	Version int64
}

// HelloResponse is the decoded HELLO success payload from the server.
type HelloResponse struct {
	ServerVersion   string
	ProtocolVersion int64
	Authenticated   bool
	Admin           bool
	Capabilities    []string
}

// authExempt is the set of commands that bypass the local auth gate
// per wire spec § 4.
var authExempt = map[string]struct{}{
	"HELLO": {},
	"PING":  {},
}

// pendingCall is one in-flight call's mailbox. The reader goroutine
// fulfils ch when the matching response arrives.
type pendingCall struct {
	ch  chan Response
	ctx context.Context
}

// Client is one TCP connection to a Vectorizer RPC server.
//
// Use Connect (raw host:port) or ConnectURL (vectorizer:// URL).
// Always call Hello before any data-plane method; otherwise Call
// returns ErrNotAuthenticated.
//
// Client is safe for concurrent use: multiple goroutines may call
// methods concurrently. Writes serialise on writeMu; responses are
// demultiplexed by Request.ID into per-call mailbox channels.
type Client struct {
	conn          net.Conn
	writeMu       sync.Mutex
	pending       sync.Map // map[uint32]*pendingCall
	nextID        atomic.Uint32
	authenticated atomic.Bool
	closeOnce     sync.Once
	closed        atomic.Bool
	readerDone    chan struct{}
}

// ConnectOptions configures the dialler.
type ConnectOptions struct {
	// Timeout caps the connect phase. Defaults to 10s when zero.
	Timeout time.Duration
}

// Connect opens a TCP connection to address (host:port). Does NOT
// send HELLO — callers MUST call Hello before any data-plane command,
// or the server will reject it.
func Connect(ctx context.Context, address string, opts ConnectOptions) (*Client, error) {
	if opts.Timeout == 0 {
		opts.Timeout = 10 * time.Second
	}
	host, port, err := splitHostPort(address)
	if err != nil {
		return nil, err
	}
	dialer := net.Dialer{Timeout: opts.Timeout}
	conn, err := dialer.DialContext(ctx, "tcp", net.JoinHostPort(host, strconv.Itoa(port)))
	if err != nil {
		return nil, fmt.Errorf("rpc dial %s: %w", address, err)
	}
	if tcp, ok := conn.(*net.TCPConn); ok {
		// Disable Nagle: every RPC frame is a complete request,
		// latency matters more than packing several into one segment.
		_ = tcp.SetNoDelay(true)
	}
	c := &Client{
		conn:       conn,
		readerDone: make(chan struct{}),
	}
	c.nextID.Store(1)
	go c.readLoop()
	return c, nil
}

// ConnectURL parses a vectorizer://host[:port] URL and dials it.
//
// REST URLs (http(s)://) are rejected with a clear error pointing
// the caller at the HTTP client.
func ConnectURL(ctx context.Context, url string, opts ConnectOptions) (*Client, error) {
	ep, err := ParseEndpoint(url)
	if err != nil {
		return nil, err
	}
	switch ep.Kind {
	case EndpointRPC:
		return Connect(ctx, net.JoinHostPort(ep.Host, strconv.Itoa(ep.Port)), opts)
	case EndpointREST:
		return nil, fmt.Errorf(
			"%w: rpc.Client cannot dial REST URL '%s'; "+
				"use the HTTP client instead, or pass a 'vectorizer://' URL",
			ErrServer, ep.URL,
		)
	}
	return nil, fmt.Errorf("unrecognised endpoint shape: %v", ep)
}

// Hello issues the HELLO handshake. Must be the first call on a fresh
// connection. Returns the server's capability list and auth flags.
func (c *Client) Hello(ctx context.Context, payload HelloPayload) (HelloResponse, error) {
	if payload.Version == 0 {
		payload.Version = 1
	}
	value := helloPayloadToValue(payload)
	result, err := c.rawCall(ctx, "HELLO", []VectorizerValue{value})
	if err != nil {
		return HelloResponse{}, err
	}
	parsed := parseHelloResponse(result)
	if parsed.Authenticated {
		c.authenticated.Store(true)
	}
	return parsed, nil
}

// Ping is the health-check command. Auth-exempt per wire spec § 4 —
// works pre-HELLO.
func (c *Client) Ping(ctx context.Context) (string, error) {
	result, err := c.rawCall(ctx, "PING", nil)
	if err != nil {
		return "", err
	}
	s, ok := result.AsStr()
	if !ok {
		return "", fmt.Errorf("%w: PING returned non-string payload", ErrServer)
	}
	return s, nil
}

// Call dispatches a generic command. Most callers should reach for a
// typed wrapper from commands.go instead.
//
// Enforces the local auth gate: data-plane commands return
// ErrNotAuthenticated before sending if HELLO hasn't succeeded.
func (c *Client) Call(ctx context.Context, command string, args []VectorizerValue) (VectorizerValue, error) {
	if _, exempt := authExempt[command]; !exempt && !c.authenticated.Load() {
		return VectorizerValue{}, ErrNotAuthenticated
	}
	return c.rawCall(ctx, command, args)
}

// IsAuthenticated returns true once HELLO has succeeded on this
// connection.
func (c *Client) IsAuthenticated() bool { return c.authenticated.Load() }

// Close shuts the connection down. In-flight calls receive
// ErrConnectionClosed.
func (c *Client) Close() error {
	var closeErr error
	c.closeOnce.Do(func() {
		c.closed.Store(true)
		closeErr = c.conn.Close()
	})
	// Drain pending — every waiter gets ErrConnectionClosed via the
	// reader's defer block. Wait for it briefly so callers that
	// Close() then immediately reuse memory don't race.
	select {
	case <-c.readerDone:
	case <-time.After(100 * time.Millisecond):
	}
	return closeErr
}

// rawCall skips the local auth check — used by Hello/Ping so the
// auth gate doesn't block the auth handshake itself.
func (c *Client) rawCall(ctx context.Context, command string, args []VectorizerValue) (VectorizerValue, error) {
	if c.closed.Load() {
		return VectorizerValue{}, ErrConnectionClosed
	}
	id := c.allocID()
	req := Request{ID: id, Command: command, Args: args}
	frame, err := EncodeFrame(req.ToMsgpack())
	if err != nil {
		return VectorizerValue{}, err
	}

	pending := &pendingCall{ch: make(chan Response, 1), ctx: ctx}
	c.pending.Store(id, pending)
	defer c.pending.Delete(id)

	c.writeMu.Lock()
	_, writeErr := c.conn.Write(frame)
	c.writeMu.Unlock()
	if writeErr != nil {
		return VectorizerValue{}, fmt.Errorf("%w: send failed: %v", ErrConnectionClosed, writeErr)
	}

	select {
	case resp := <-pending.ch:
		if resp.Result.IsOk {
			return resp.Result.Value, nil
		}
		return VectorizerValue{}, fmt.Errorf("%w: %s", ErrServer, resp.Result.Message)
	case <-ctx.Done():
		return VectorizerValue{}, ctx.Err()
	case <-c.readerDone:
		return VectorizerValue{}, ErrConnectionClosed
	}
}

func (c *Client) allocID() uint32 {
	for {
		id := c.nextID.Add(1)
		// 0 is reserved (uninitialised); wrap past zero on overflow.
		if id != 0 {
			return id
		}
	}
}

// readLoop pulls frames from the wire forever, dispatches them by id
// to per-call mailboxes, and closes down on EOF or any error.
func (c *Client) readLoop() {
	defer func() {
		close(c.readerDone)
		// Wake every pending call so they fail fast instead of
		// hanging on the channel forever.
		c.pending.Range(func(_, v any) bool {
			pending := v.(*pendingCall)
			select {
			case pending.ch <- ResponseErr(0, "connection closed"):
			default:
			}
			return true
		})
	}()

	for {
		raw, err := ReadFrame(c.conn)
		if err != nil {
			// Clean EOF or torn connection — both unrecoverable for
			// this Client. Pending callers see ErrConnectionClosed.
			if err == io.EOF || isUseOfClosed(err) {
				return
			}
			return
		}
		resp, err := ResponseFromMsgpack(raw)
		if err != nil {
			// Skip malformed frames — log channel would be too noisy
			// for a library. Pending callers will eventually time out
			// or get ErrConnectionClosed when the socket dies.
			continue
		}
		if v, ok := c.pending.LoadAndDelete(resp.ID); ok {
			pending := v.(*pendingCall)
			select {
			case pending.ch <- resp:
			default:
			}
		}
	}
}

func isUseOfClosed(err error) bool {
	return err != nil && (errors.Is(err, net.ErrClosed) ||
		strings.Contains(err.Error(), "use of closed network connection"))
}

// ── helpers ────────────────────────────────────────────────────────

func helloPayloadToValue(p HelloPayload) VectorizerValue {
	pairs := []MapPair{
		{Key: StrValue("version"), Value: IntValue(p.Version)},
	}
	if p.Token != "" {
		pairs = append(pairs, MapPair{Key: StrValue("token"), Value: StrValue(p.Token)})
	}
	if p.APIKey != "" {
		pairs = append(pairs, MapPair{Key: StrValue("api_key"), Value: StrValue(p.APIKey)})
	}
	if p.ClientName != "" {
		pairs = append(pairs, MapPair{Key: StrValue("client_name"), Value: StrValue(p.ClientName)})
	}
	return MapValue(pairs)
}

func parseHelloResponse(value VectorizerValue) HelloResponse {
	out := HelloResponse{}
	if v, ok := value.MapGet("server_version"); ok {
		if s, ok := v.AsStr(); ok {
			out.ServerVersion = s
		}
	}
	if v, ok := value.MapGet("protocol_version"); ok {
		if i, ok := v.AsInt(); ok {
			out.ProtocolVersion = i
		}
	}
	if v, ok := value.MapGet("authenticated"); ok {
		if b, ok := v.AsBool(); ok {
			out.Authenticated = b
		}
	}
	if v, ok := value.MapGet("admin"); ok {
		if b, ok := v.AsBool(); ok {
			out.Admin = b
		}
	}
	if v, ok := value.MapGet("capabilities"); ok {
		if arr, ok := v.AsArray(); ok {
			out.Capabilities = make([]string, 0, len(arr))
			for _, item := range arr {
				if s, ok := item.AsStr(); ok {
					out.Capabilities = append(out.Capabilities, s)
				}
			}
		}
	}
	return out
}

// splitHostPort splits a host:port string. IPv6 literals
// ("[::1]:1234") are handled specially so the colons inside the
// brackets aren't treated as port separators.
func splitHostPort(address string) (string, int, error) {
	if strings.HasPrefix(address, "[") {
		close := strings.Index(address, "]")
		if close < 0 {
			return "", 0, fmt.Errorf("unterminated IPv6 literal in address: %s", address)
		}
		host := address[1:close]
		rest := address[close+1:]
		if !strings.HasPrefix(rest, ":") {
			return "", 0, fmt.Errorf("expected ':<port>' after IPv6 literal in address: %s", address)
		}
		port, err := strconv.Atoi(rest[1:])
		if err != nil {
			return "", 0, fmt.Errorf("invalid port in address %s: %w", address, err)
		}
		return host, port, nil
	}
	host, portStr, err := net.SplitHostPort(address)
	if err != nil {
		return "", 0, fmt.Errorf("invalid address %s: %w", address, err)
	}
	port, err := strconv.Atoi(portStr)
	if err != nil {
		return "", 0, fmt.Errorf("invalid port in address %s: %w", address, err)
	}
	return host, port, nil
}
