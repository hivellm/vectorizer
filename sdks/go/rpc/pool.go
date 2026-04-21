package rpc

import (
	"context"
	"errors"
	"net"
	"strconv"
	"sync"
)

// PoolConfig configures a Pool. Address is required; the rest take
// safe defaults when zero-valued.
type PoolConfig struct {
	// Address is the host:port every connection in the pool dials.
	Address string
	// MaxConnections caps the total live + checked-out connections.
	// Calls block on Acquire when this many are checked out. Defaults
	// to 8 when zero.
	MaxConnections int
	// Hello is the HELLO payload sent on every newly-built connection.
	Hello HelloPayload
	// ConnectOptions are passed through to Connect.
	ConnectOptions ConnectOptions
}

// Pool is a bounded pool of RPC clients.
//
// Does NOT open any connections eagerly; the first Acquire dials the
// first connection. MaxConnections is enforced via a buffered
// channel so simultaneous Acquires beyond the cap block until a slot
// frees.
//
// Intentionally NOT a full retry/health-check pool — that complexity
// isn't needed for v1. A torn connection surfaces on the next Call as
// ErrConnectionClosed rather than being re-validated up-front.
type Pool struct {
	cfg PoolConfig

	idleMu sync.Mutex
	idle   []*Client

	// permits is a buffered channel acting as a semaphore. Acquire
	// reads a permit; Release writes one back. Capacity =
	// MaxConnections. Pre-filled at construction.
	permits chan struct{}
}

// NewPool builds a new Pool with the given config. Does not dial any
// connection; the first Acquire dials lazily.
func NewPool(cfg PoolConfig) *Pool {
	maxConns := cfg.MaxConnections
	if maxConns < 1 {
		maxConns = 8
	}
	cfg.MaxConnections = maxConns
	permits := make(chan struct{}, maxConns)
	for i := 0; i < maxConns; i++ {
		permits <- struct{}{}
	}
	return &Pool{cfg: cfg, permits: permits}
}

// PooledClient is a guard returned by Pool.Acquire. The wrapped
// Client must be returned via Release once the caller is done. The
// zero value is invalid; use Acquire to build one.
type PooledClient struct {
	pool   *Pool
	client *Client
	done   bool
}

// Client returns the underlying RPC client. Panics if Release was
// already called (this would otherwise return a Client that's been
// recycled into another caller's PooledClient).
func (p *PooledClient) Client() *Client {
	if p.done {
		panic("rpc: PooledClient already released")
	}
	return p.client
}

// Release returns the client to the pool. Subsequent calls are no-ops.
func (p *PooledClient) Release() {
	if p.done {
		return
	}
	p.done = true
	p.pool.returnClient(p.client)
}

// Acquire takes a client from the pool, blocking until a slot frees
// when the pool is at capacity. The caller MUST call PooledClient.Release
// when done; deferring it next to the Acquire is the idiomatic shape.
func (p *Pool) Acquire(ctx context.Context) (*PooledClient, error) {
	select {
	case <-p.permits:
	case <-ctx.Done():
		return nil, ctx.Err()
	}

	// Try the idle list first.
	p.idleMu.Lock()
	if n := len(p.idle); n > 0 {
		c := p.idle[n-1]
		p.idle = p.idle[:n-1]
		p.idleMu.Unlock()
		return &PooledClient{pool: p, client: c}, nil
	}
	p.idleMu.Unlock()

	// Miss — build a new connection.
	c, err := Connect(ctx, p.cfg.Address, p.cfg.ConnectOptions)
	if err != nil {
		// Failed to dial — return the permit so other Acquires don't
		// hang forever waiting for a slot that nobody holds.
		p.permits <- struct{}{}
		return nil, err
	}
	if _, err := c.Hello(ctx, p.cfg.Hello); err != nil {
		_ = c.Close()
		p.permits <- struct{}{}
		return nil, err
	}
	return &PooledClient{pool: p, client: c}, nil
}

// IdleCount returns the number of clients currently sitting in the
// idle list. Useful for diagnostics; production code should not
// branch on this.
func (p *Pool) IdleCount() int {
	p.idleMu.Lock()
	defer p.idleMu.Unlock()
	return len(p.idle)
}

// Close closes every idle client. In-flight clients held by callers
// are unaffected; they close on their own Release path or when the
// caller drops the reference.
func (p *Pool) Close() error {
	p.idleMu.Lock()
	idle := p.idle
	p.idle = nil
	p.idleMu.Unlock()
	var firstErr error
	for _, c := range idle {
		if err := c.Close(); err != nil && firstErr == nil {
			firstErr = err
		}
	}
	return firstErr
}

func (p *Pool) returnClient(c *Client) {
	p.idleMu.Lock()
	p.idle = append(p.idle, c)
	p.idleMu.Unlock()
	p.permits <- struct{}{}
}

// AddressFromHostPort is a small convenience for callers building a
// PoolConfig from separate host + port. Returns the host:port string,
// IPv6-bracketed when needed.
func AddressFromHostPort(host string, port int) string {
	return net.JoinHostPort(host, strconv.Itoa(port))
}

// Pool errors aren't separately defined — the underlying Client's
// errors (ErrConnectionClosed, ErrServer, ErrNotAuthenticated)
// propagate verbatim, plus ctx.Err() from Acquire when the context
// expires before a permit is free.
var _ = errors.New
