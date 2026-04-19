package rpc

import (
	"context"
	"errors"
	"net"
	"strings"
	"sync"
	"testing"
	"time"
)

// End-to-end integration tests for Client.
//
// Spins up an in-test server on 127.0.0.1:0 that speaks the
// VectorizerRPC wire format using the SDK's own codec + types
// (because the production server isn't a Go dependency), and drives
// it from Client to prove:
//
//   - HELLO handshake produces the expected HelloResponse shape.
//   - PING works pre-HELLO (auth-exempt per wire spec § 4).
//   - A data-plane call before HELLO returns ErrNotAuthenticated
//     from the local gate.
//   - Concurrent calls on the same connection get correctly
//     demultiplexed by Request.ID.
//   - The typed wrappers (ListCollections, GetCollectionInfo,
//     SearchBasic) round-trip through the codec.
//   - ConnectURL accepts the canonical vectorizer:// form and
//     rejects REST URLs with a clear error.

func buildHelloResponse(rid uint32) Response {
	return ResponseOk(rid, MapValue([]MapPair{
		{Key: StrValue("server_version"), Value: StrValue("test-fixture/0.0.0")},
		{Key: StrValue("protocol_version"), Value: IntValue(1)},
		{Key: StrValue("authenticated"), Value: BoolValue(true)},
		{Key: StrValue("admin"), Value: BoolValue(true)},
		{Key: StrValue("capabilities"), Value: ArrayValue([]VectorizerValue{
			StrValue("PING"),
			StrValue("collections.list"),
			StrValue("collections.get_info"),
			StrValue("vectors.get"),
			StrValue("search.basic"),
		})},
	}))
}

func buildCollectionInfoResponse(rid uint32, name string) Response {
	return ResponseOk(rid, MapValue([]MapPair{
		{Key: StrValue("name"), Value: StrValue(name)},
		{Key: StrValue("vector_count"), Value: IntValue(42)},
		{Key: StrValue("document_count"), Value: IntValue(10)},
		{Key: StrValue("dimension"), Value: IntValue(384)},
		{Key: StrValue("metric"), Value: StrValue("Cosine")},
		{Key: StrValue("created_at"), Value: StrValue("2026-04-19T00:00:00Z")},
		{Key: StrValue("updated_at"), Value: StrValue("2026-04-19T00:00:00Z")},
	}))
}

func buildSearchBasicResponse(rid uint32) Response {
	return ResponseOk(rid, ArrayValue([]VectorizerValue{
		MapValue([]MapPair{
			{Key: StrValue("id"), Value: StrValue("vec-0")},
			{Key: StrValue("score"), Value: FloatValue(0.95)},
			{Key: StrValue("payload"), Value: StrValue(`{"title":"hit one"}`)},
		}),
		MapValue([]MapPair{
			{Key: StrValue("id"), Value: StrValue("vec-1")},
			{Key: StrValue("score"), Value: FloatValue(0.81)},
		}),
	}))
}

func dispatch(req Request, state *struct {
	authenticated bool
	mu            sync.Mutex
}) Response {
	cmd := req.Command
	if cmd == "HELLO" {
		state.mu.Lock()
		state.authenticated = true
		state.mu.Unlock()
		return buildHelloResponse(req.ID)
	}
	if cmd == "PING" {
		return ResponseOk(req.ID, StrValue("PONG"))
	}
	state.mu.Lock()
	authed := state.authenticated
	state.mu.Unlock()
	if !authed {
		return ResponseErr(req.ID, "authentication required: send HELLO first ("+cmd+")")
	}
	switch cmd {
	case "collections.list":
		return ResponseOk(req.ID, ArrayValue([]VectorizerValue{
			StrValue("alpha-docs"),
			StrValue("beta-source"),
		}))
	case "collections.get_info":
		name := "unknown"
		if len(req.Args) > 0 {
			if s, ok := req.Args[0].AsStr(); ok {
				name = s
			}
		}
		return buildCollectionInfoResponse(req.ID, name)
	case "search.basic":
		return buildSearchBasicResponse(req.ID)
	}
	return ResponseErr(req.ID, "unknown command '"+cmd+"'")
}

type fakeServer struct {
	listener net.Listener
	addr     string
	wg       sync.WaitGroup
}

func spawnFakeServer(t *testing.T) *fakeServer {
	t.Helper()
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	srv := &fakeServer{listener: listener, addr: listener.Addr().String()}
	srv.wg.Add(1)
	go srv.acceptLoop()
	return srv
}

func (s *fakeServer) acceptLoop() {
	defer s.wg.Done()
	for {
		conn, err := s.listener.Accept()
		if err != nil {
			return
		}
		go s.handleConn(conn)
	}
}

func (s *fakeServer) handleConn(conn net.Conn) {
	defer conn.Close()
	state := &struct {
		authenticated bool
		mu            sync.Mutex
	}{}
	for {
		raw, err := ReadFrame(conn)
		if err != nil {
			return
		}
		arr, ok := raw.([]any)
		if !ok || len(arr) != 3 {
			return
		}
		id, _ := coerceInt(arr[0])
		cmd, _ := arr[1].(string)
		argsRaw, _ := arr[2].([]any)
		args := make([]VectorizerValue, len(argsRaw))
		for i, a := range argsRaw {
			v, err := ValueFromMsgpack(a)
			if err != nil {
				return
			}
			args[i] = v
		}
		req := Request{ID: uint32(id), Command: cmd, Args: args}
		resp := dispatch(req, state)
		frame, err := EncodeFrame(resp.ToMsgpack())
		if err != nil {
			return
		}
		if _, err := conn.Write(frame); err != nil {
			return
		}
	}
}

func (s *fakeServer) close() {
	_ = s.listener.Close()
	s.wg.Wait()
}

func TestIntegration_HelloPingTypedCommands(t *testing.T) {
	srv := spawnFakeServer(t)
	defer srv.close()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client, err := Connect(ctx, srv.addr, ConnectOptions{})
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer client.Close()

	pong, err := client.Ping(ctx)
	if err != nil {
		t.Fatalf("Ping: %v", err)
	}
	if pong != "PONG" {
		t.Fatalf("Ping returned %q", pong)
	}

	hello, err := client.Hello(ctx, HelloPayload{ClientName: "rpc-integration-test"})
	if err != nil {
		t.Fatalf("Hello: %v", err)
	}
	if !hello.Authenticated || !hello.Admin {
		t.Fatalf("hello flags: %+v", hello)
	}
	if hello.ProtocolVersion != 1 || hello.ServerVersion != "test-fixture/0.0.0" {
		t.Fatalf("hello version mismatch: %+v", hello)
	}
	foundCol := false
	for _, c := range hello.Capabilities {
		if c == "collections.list" {
			foundCol = true
			break
		}
	}
	if !foundCol {
		t.Fatalf("capabilities missing collections.list: %v", hello.Capabilities)
	}

	cols, err := client.ListCollections(ctx)
	if err != nil {
		t.Fatalf("ListCollections: %v", err)
	}
	if len(cols) != 2 || cols[0] != "alpha-docs" || cols[1] != "beta-source" {
		t.Fatalf("ListCollections returned %v", cols)
	}

	info, err := client.GetCollectionInfo(ctx, "alpha-docs")
	if err != nil {
		t.Fatalf("GetCollectionInfo: %v", err)
	}
	if info.Name != "alpha-docs" || info.VectorCount != 42 || info.Dimension != 384 || info.Metric != "Cosine" {
		t.Fatalf("info mismatch: %+v", info)
	}

	hits, err := client.SearchBasic(ctx, "alpha-docs", "anything", 10)
	if err != nil {
		t.Fatalf("SearchBasic: %v", err)
	}
	if len(hits) != 2 {
		t.Fatalf("SearchBasic returned %d hits", len(hits))
	}
	if hits[0].ID != "vec-0" {
		t.Fatalf("hit[0].ID = %q", hits[0].ID)
	}
	if d := hits[0].Score - 0.95; d > 1e-9 || d < -1e-9 {
		t.Fatalf("hit[0].Score = %f", hits[0].Score)
	}
	if hits[0].Payload == nil || *hits[0].Payload != `{"title":"hit one"}` {
		t.Fatalf("hit[0].Payload = %v", hits[0].Payload)
	}
	if hits[1].ID != "vec-1" || hits[1].Payload != nil {
		t.Fatalf("hit[1] mismatch: %+v", hits[1])
	}
}

func TestIntegration_DataPlaneBeforeHelloRejectedLocally(t *testing.T) {
	srv := spawnFakeServer(t)
	defer srv.close()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client, err := Connect(ctx, srv.addr, ConnectOptions{})
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer client.Close()

	_, err = client.ListCollections(ctx)
	if !errors.Is(err, ErrNotAuthenticated) {
		t.Fatalf("expected ErrNotAuthenticated, got %v", err)
	}
}

func TestIntegration_ConcurrentCallsDemultiplexedByID(t *testing.T) {
	srv := spawnFakeServer(t)
	defer srv.close()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client, err := Connect(ctx, srv.addr, ConnectOptions{})
	if err != nil {
		t.Fatalf("Connect: %v", err)
	}
	defer client.Close()

	if _, err := client.Hello(ctx, HelloPayload{ClientName: "concurrent-test"}); err != nil {
		t.Fatalf("Hello: %v", err)
	}

	const N = 16
	var wg sync.WaitGroup
	results := make([][]string, N)
	errs := make([]error, N)
	wg.Add(N)
	for i := 0; i < N; i++ {
		i := i
		go func() {
			defer wg.Done()
			results[i], errs[i] = client.ListCollections(ctx)
		}()
	}
	wg.Wait()
	for i, e := range errs {
		if e != nil {
			t.Fatalf("goroutine %d: %v", i, e)
		}
		if len(results[i]) != 2 || results[i][0] != "alpha-docs" {
			t.Fatalf("goroutine %d: got %v", i, results[i])
		}
	}
}

func TestIntegration_ConnectURLAcceptsVectorizerScheme(t *testing.T) {
	srv := spawnFakeServer(t)
	defer srv.close()

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	client, err := ConnectURL(ctx, "vectorizer://"+srv.addr, ConnectOptions{})
	if err != nil {
		t.Fatalf("ConnectURL: %v", err)
	}
	defer client.Close()

	pong, err := client.Ping(ctx)
	if err != nil || pong != "PONG" {
		t.Fatalf("Ping after ConnectURL: pong=%q err=%v", pong, err)
	}
}

func TestIntegration_ConnectURLRejectsHTTPScheme(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()

	_, err := ConnectURL(ctx, "http://localhost:15002", ConnectOptions{})
	if err == nil {
		t.Fatal("expected error for http:// URL, got nil")
	}
	if !errors.Is(err, ErrServer) {
		t.Fatalf("expected ErrServer, got %v", err)
	}
	msg := err.Error()
	if !strings.Contains(msg, "REST URL") {
		t.Fatalf("error should mention REST URL: %v", err)
	}
	if !strings.Contains(msg, "HTTP client") {
		t.Fatalf("error should point at the HTTP client: %v", err)
	}
}
