package rpc

// Wire-shape tests for phase16 typed wrappers.
//
// Each test spins up the in-process fake server from rpc_integration_test.go,
// extends its dispatch table by injecting the relevant command handler, and
// verifies that the typed wrapper correctly decodes the wire-encoded response.
//
// Coverage: one test per domain group as required (10 tests total).

import (
	"context"
	"net"
	"sync"
	"testing"
	"time"
)

// ── helper: fake server with extensible dispatch ──────────────────

type extDispatch func(req Request) (Response, bool)

type extFakeServer struct {
	listener net.Listener
	addr     string
	wg       sync.WaitGroup
	mu       sync.Mutex
	handlers []extDispatch
}

func spawnExtFakeServer(t *testing.T) *extFakeServer {
	t.Helper()
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen: %v", err)
	}
	s := &extFakeServer{listener: l, addr: l.Addr().String()}
	s.wg.Add(1)
	go s.acceptLoop()
	return s
}

func (s *extFakeServer) register(fn extDispatch) {
	s.mu.Lock()
	s.handlers = append(s.handlers, fn)
	s.mu.Unlock()
}

func (s *extFakeServer) close() {
	_ = s.listener.Close()
	s.wg.Wait()
}

func (s *extFakeServer) acceptLoop() {
	defer s.wg.Done()
	for {
		conn, err := s.listener.Accept()
		if err != nil {
			return
		}
		go s.handleConn(conn)
	}
}

func (s *extFakeServer) handleConn(conn net.Conn) {
	defer conn.Close()
	authenticated := false
	for {
		raw, err := ReadFrame(conn)
		if err != nil {
			return
		}
		arr, ok := raw.([]any)
		if !ok || len(arr) != 3 {
			return
		}
		idRaw, _ := coerceInt(arr[0])
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
		rid := uint32(idRaw)
		req := Request{ID: rid, Command: cmd, Args: args}

		// HELLO / PING are always handled.
		if cmd == "HELLO" {
			authenticated = true
			resp := buildHelloResponse(rid)
			frame, _ := EncodeFrame(resp.ToMsgpack())
			conn.Write(frame) //nolint:errcheck
			continue
		}
		if cmd == "PING" {
			resp := ResponseOk(rid, StrValue("PONG"))
			frame, _ := EncodeFrame(resp.ToMsgpack())
			conn.Write(frame) //nolint:errcheck
			continue
		}
		if !authenticated {
			resp := ResponseErr(rid, "authentication required")
			frame, _ := EncodeFrame(resp.ToMsgpack())
			conn.Write(frame) //nolint:errcheck
			continue
		}
		// Try registered handlers in order.
		s.mu.Lock()
		handlers := make([]extDispatch, len(s.handlers))
		copy(handlers, s.handlers)
		s.mu.Unlock()
		handled := false
		for _, h := range handlers {
			if resp, ok := h(req); ok {
				frame, _ := EncodeFrame(resp.ToMsgpack())
				conn.Write(frame) //nolint:errcheck
				handled = true
				break
			}
		}
		if !handled {
			resp := ResponseErr(rid, "unknown command '"+cmd+"'")
			frame, _ := EncodeFrame(resp.ToMsgpack())
			conn.Write(frame) //nolint:errcheck
		}
	}
}

func dialAndHello(t *testing.T, addr string) (*Client, context.CancelFunc) {
	t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	c, err := Connect(ctx, addr, ConnectOptions{})
	if err != nil {
		cancel()
		t.Fatalf("Connect: %v", err)
	}
	if _, err := c.Hello(ctx, HelloPayload{ClientName: "phase16-test"}); err != nil {
		cancel()
		c.Close()
		t.Fatalf("Hello: %v", err)
	}
	return c, cancel
}

// ── Test 1: Collections domain ────────────────────────────────────

func TestPhase16_Collections_CreateAndDelete(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "collections.create":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("name"), Value: StrValue("my-col")},
				{Key: StrValue("dimension"), Value: IntValue(128)},
				{Key: StrValue("metric"), Value: StrValue("Cosine")},
				{Key: StrValue("success"), Value: BoolValue(true)},
			})), true
		case "collections.delete":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("success"), Value: BoolValue(true)},
			})), true
		case "collections.force_save":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("success"), Value: BoolValue(true)},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	res, err := c.CreateCollectionRpc(ctx, "my-col", MapValue([]MapPair{
		{Key: StrValue("dimension"), Value: IntValue(128)},
	}))
	if err != nil {
		t.Fatalf("CreateCollectionRpc: %v", err)
	}
	if res.Name != "my-col" || res.Dimension != 128 || res.Metric != "Cosine" || !res.Success {
		t.Fatalf("unexpected CreateCollectionResult: %+v", res)
	}

	ok, err := c.DeleteCollectionRpc(ctx, "my-col")
	if err != nil || !ok {
		t.Fatalf("DeleteCollectionRpc: ok=%v err=%v", ok, err)
	}

	ok, err = c.ForceSaveCollection(ctx, "my-col")
	if err != nil || !ok {
		t.Fatalf("ForceSaveCollection: ok=%v err=%v", ok, err)
	}
}

// ── Test 2: Vectors insert / update / delete ──────────────────────

func TestPhase16_Vectors_WriteOperations(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "vectors.insert", "vectors.insert_text", "vectors.update":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("id"), Value: StrValue("v-42")},
				{Key: StrValue("success"), Value: BoolValue(true)},
			})), true
		case "vectors.delete":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("success"), Value: BoolValue(true)},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	ins, err := c.InsertVectorRpc(ctx, "col", "", []float64{0.1, 0.2, 0.3}, NullValue())
	if err != nil || ins.ID != "v-42" || !ins.Success {
		t.Fatalf("InsertVectorRpc: %+v err=%v", ins, err)
	}

	txt, err := c.InsertTextVectorRpc(ctx, "col", "", "hello world", NullValue())
	if err != nil || txt.ID != "v-42" || !txt.Success {
		t.Fatalf("InsertTextVectorRpc: %+v err=%v", txt, err)
	}

	upd, err := c.UpdateVectorRpc(ctx, "col", "v-42", []float64{0.4, 0.5, 0.6}, NullValue())
	if err != nil || upd.ID != "v-42" || !upd.Success {
		t.Fatalf("UpdateVectorRpc: %+v err=%v", upd, err)
	}

	ok, err := c.DeleteVectorRpc(ctx, "col", "v-42")
	if err != nil || !ok {
		t.Fatalf("DeleteVectorRpc: ok=%v err=%v", ok, err)
	}
}

// ── Test 3: Vectors batch operations ─────────────────────────────

func TestPhase16_Vectors_BatchOperations(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "vectors.batch_insert", "vectors.batch_insert_texts":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("inserted"), Value: IntValue(2)},
				{Key: StrValue("failed"), Value: IntValue(0)},
				{Key: StrValue("results"), Value: ArrayValue([]VectorizerValue{
					MapValue([]MapPair{
						{Key: StrValue("index"), Value: IntValue(0)},
						{Key: StrValue("id"), Value: StrValue("v-0")},
						{Key: StrValue("status"), Value: StrValue("ok")},
					}),
					MapValue([]MapPair{
						{Key: StrValue("index"), Value: IntValue(1)},
						{Key: StrValue("id"), Value: StrValue("v-1")},
						{Key: StrValue("status"), Value: StrValue("ok")},
					}),
				})},
			})), true
		case "vectors.batch_delete":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("deleted"), Value: IntValue(2)},
				{Key: StrValue("failed"), Value: IntValue(0)},
				{Key: StrValue("results"), Value: ArrayValue(nil)},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	items := []VectorizerValue{
		MapValue([]MapPair{{Key: StrValue("data"), Value: ArrayValue([]VectorizerValue{FloatValue(0.1)})}}),
		MapValue([]MapPair{{Key: StrValue("data"), Value: ArrayValue([]VectorizerValue{FloatValue(0.2)})}}),
	}

	br, err := c.BatchInsertVectors(ctx, "col", items)
	if err != nil || br.Inserted != 2 || br.Failed != 0 || len(br.Results) != 2 {
		t.Fatalf("BatchInsertVectors: %+v err=%v", br, err)
	}
	if br.Results[0].ID == nil || *br.Results[0].ID != "v-0" {
		t.Fatalf("batch result[0].ID = %v", br.Results[0].ID)
	}

	dr, err := c.BatchDeleteVectors(ctx, "col", []string{"v-0", "v-1"})
	if err != nil || dr.Deleted != 2 {
		t.Fatalf("BatchDeleteVectors: %+v err=%v", dr, err)
	}
}

// ── Test 4: Search domain ─────────────────────────────────────────

func TestPhase16_Search_ByTextAndExplain(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "search.by_text":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("results"), Value: ArrayValue([]VectorizerValue{
					MapValue([]MapPair{
						{Key: StrValue("id"), Value: StrValue("r-1")},
						{Key: StrValue("score"), Value: FloatValue(0.88)},
					}),
				})},
			})), true
		case "search.explain":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("hits"), Value: ArrayValue([]VectorizerValue{
					MapValue([]MapPair{
						{Key: StrValue("id"), Value: StrValue("h-0")},
						{Key: StrValue("score"), Value: FloatValue(0.77)},
					}),
				})},
				{Key: StrValue("collection"), Value: StrValue("col")},
				{Key: StrValue("k"), Value: IntValue(5)},
				{Key: StrValue("trace"), Value: MapValue([]MapPair{
					{Key: StrValue("visited_nodes"), Value: IntValue(120)},
					{Key: StrValue("ef_search"), Value: IntValue(64)},
					{Key: StrValue("hnsw_search_ms"), Value: FloatValue(1.23)},
					{Key: StrValue("total_ms"), Value: FloatValue(2.50)},
				})},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	hits, err := c.SearchByText(ctx, "col", "query", 5)
	if err != nil || len(hits) != 1 || hits[0].ID != "r-1" {
		t.Fatalf("SearchByText: %v err=%v", hits, err)
	}

	explain, err := c.SearchExplain(ctx, "col", MapValue(nil))
	if err != nil {
		t.Fatalf("SearchExplain: %v", err)
	}
	if len(explain.Hits) != 1 || explain.Hits[0].ID != "h-0" {
		t.Fatalf("explain.Hits = %v", explain.Hits)
	}
	if explain.Trace.VisitedNodes != 120 || explain.Trace.EfSearch != 64 {
		t.Fatalf("explain.Trace = %+v", explain.Trace)
	}
}

// ── Test 5: Discovery domain ──────────────────────────────────────

func TestPhase16_Discovery_DiscoverAndCompress(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "discovery.discover":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("answer_prompt"), Value: StrValue("Here is the plan")},
				{Key: StrValue("sections"), Value: IntValue(3)},
				{Key: StrValue("bullets"), Value: IntValue(9)},
				{Key: StrValue("chunks"), Value: IntValue(27)},
			})), true
		case "discovery.compress_evidence":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("bullets"), Value: ArrayValue([]VectorizerValue{
					MapValue([]MapPair{
						{Key: StrValue("text"), Value: StrValue("bullet one")},
						{Key: StrValue("source_id"), Value: StrValue("doc-1")},
						{Key: StrValue("score"), Value: FloatValue(0.9)},
					}),
				})},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	dr, err := c.Discover(ctx, MapValue([]MapPair{{Key: StrValue("query"), Value: StrValue("q")}}))
	if err != nil || dr.AnswerPrompt != "Here is the plan" || dr.Sections != 3 {
		t.Fatalf("Discover: %+v err=%v", dr, err)
	}

	bullets, err := c.CompressEvidence(ctx, MapValue([]MapPair{{Key: StrValue("chunks"), Value: ArrayValue(nil)}}))
	if err != nil || len(bullets) != 1 || bullets[0].Text != "bullet one" {
		t.Fatalf("CompressEvidence: %v err=%v", bullets, err)
	}
}

// ── Test 6: File ops domain ───────────────────────────────────────

func TestPhase16_File_ContentAndList(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "file.content":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("content"), Value: StrValue("package main\n")},
				{Key: StrValue("size_bytes"), Value: IntValue(14)},
			})), true
		case "file.list":
			return ResponseOk(req.ID, ArrayValue([]VectorizerValue{
				StrValue("main.go"),
				StrValue("README.md"),
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	content, err := c.FileContent(ctx, MapValue([]MapPair{
		{Key: StrValue("collection"), Value: StrValue("col")},
		{Key: StrValue("file_path"), Value: StrValue("main.go")},
	}))
	if err != nil {
		t.Fatalf("FileContent: %v", err)
	}
	if s, ok := content.MapGet("content"); !ok {
		t.Fatal("FileContent: missing content key")
	} else if v, _ := s.AsStr(); v != "package main\n" {
		t.Fatalf("FileContent.content = %q", v)
	}

	list, err := c.FileList(ctx, MapValue([]MapPair{{Key: StrValue("collection"), Value: StrValue("col")}}))
	if err != nil {
		t.Fatalf("FileList: %v", err)
	}
	arr, ok := list.AsArray()
	if !ok || len(arr) != 2 {
		t.Fatalf("FileList: expected 2-element Array, got %v", list)
	}
}

// ── Test 7: Graph domain ──────────────────────────────────────────

func TestPhase16_Graph_DiscoverEdgesAndStatus(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "graph.discover_edges":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("success"), Value: BoolValue(true)},
				{Key: StrValue("total_nodes"), Value: IntValue(500)},
				{Key: StrValue("nodes_processed"), Value: IntValue(500)},
				{Key: StrValue("nodes_with_edges"), Value: IntValue(490)},
				{Key: StrValue("total_edges_created"), Value: IntValue(2400)},
			})), true
		case "graph.discovery_status":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("total_nodes"), Value: IntValue(500)},
				{Key: StrValue("nodes_with_edges"), Value: IntValue(490)},
				{Key: StrValue("total_edges"), Value: IntValue(2400)},
				{Key: StrValue("progress_percentage"), Value: FloatValue(98.0)},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	de, err := c.GraphDiscoverEdges(ctx, "col", MapValue(nil))
	if err != nil || !de.Success || de.TotalNodes != 500 || de.TotalEdgesCreated != 2400 {
		t.Fatalf("GraphDiscoverEdges: %+v err=%v", de, err)
	}

	ds, err := c.GraphDiscoveryStatus(ctx, "col")
	if err != nil || ds.TotalNodes != 500 || ds.ProgressPercentage != 98.0 {
		t.Fatalf("GraphDiscoveryStatus: %+v err=%v", ds, err)
	}
}

// ── Test 8: Admin domain ──────────────────────────────────────────

func TestPhase16_Admin_StatsAndStatus(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "admin.stats":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("collections_count"), Value: IntValue(12)},
				{Key: StrValue("total_vectors"), Value: IntValue(48000)},
				{Key: StrValue("version"), Value: StrValue("3.8.0")},
			})), true
		case "admin.status":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("ready"), Value: BoolValue(true)},
				{Key: StrValue("collections_count"), Value: IntValue(12)},
				{Key: StrValue("version"), Value: StrValue("3.8.0")},
			})), true
		case "admin.slow_queries_config":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("threshold_ms"), Value: IntValue(200)},
				{Key: StrValue("capacity"), Value: IntValue(100)},
				{Key: StrValue("status"), Value: StrValue("ok")},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	stats, err := c.AdminStats(ctx)
	if err != nil || stats.CollectionsCount != 12 || stats.TotalVectors != 48000 || stats.Version != "3.8.0" {
		t.Fatalf("AdminStats: %+v err=%v", stats, err)
	}

	status, err := c.AdminStatus(ctx)
	if err != nil || !status.Ready || status.CollectionsCount != 12 {
		t.Fatalf("AdminStatus: %+v err=%v", status, err)
	}

	cfg, err := c.AdminSlowQueriesConfig(ctx, MapValue([]MapPair{
		{Key: StrValue("threshold_ms"), Value: IntValue(200)},
	}))
	if err != nil || cfg.ThresholdMs != 200 || cfg.Capacity != 100 {
		t.Fatalf("AdminSlowQueriesConfig: %+v err=%v", cfg, err)
	}
}

// ── Test 9: Auth domain ───────────────────────────────────────────

func TestPhase16_Auth_MeAndRotateKey(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "auth.me":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("username"), Value: StrValue("alice")},
				{Key: StrValue("authenticated"), Value: BoolValue(true)},
			})), true
		case "auth.api_keys_rotate":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("old_key_id"), Value: StrValue("old-id-123")},
				{Key: StrValue("new_key_id"), Value: StrValue("new-id-456")},
				{Key: StrValue("new_token"), Value: StrValue("tok-abc")},
			})), true
		case "auth.api_keys_revoke":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("success"), Value: BoolValue(true)},
			})), true
		case "auth.validate_password":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("valid"), Value: BoolValue(true)},
				{Key: StrValue("errors"), Value: ArrayValue(nil)},
			})), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	me, err := c.AuthMe(ctx)
	if err != nil || me.Username != "alice" || !me.Authenticated {
		t.Fatalf("AuthMe: %+v err=%v", me, err)
	}

	rot, err := c.RotateApiKeyRpc(ctx, "old-id-123")
	if err != nil || rot.OldKeyID != "old-id-123" || rot.NewKeyID != "new-id-456" || rot.NewToken != "tok-abc" {
		t.Fatalf("RotateApiKeyRpc: %+v err=%v", rot, err)
	}
	if rot.GraceUntil != nil {
		t.Fatalf("GraceUntil should be nil, got %v", rot.GraceUntil)
	}

	ok, err := c.AuthApiKeysRevoke(ctx, "key-id")
	if err != nil || !ok {
		t.Fatalf("AuthApiKeysRevoke: ok=%v err=%v", ok, err)
	}

	vp, err := c.AuthValidatePassword(ctx, "hunter2")
	if err != nil || !vp.Valid || len(vp.Errors) != 0 {
		t.Fatalf("AuthValidatePassword: %+v err=%v", vp, err)
	}
}

// ── Test 10: Replication + Cluster domain ─────────────────────────

func TestPhase16_ReplicationAndCluster(t *testing.T) {
	srv := spawnExtFakeServer(t)
	defer srv.close()

	srv.register(func(req Request) (Response, bool) {
		switch req.Command {
		case "replication.configure":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("success"), Value: BoolValue(true)},
				{Key: StrValue("role"), Value: StrValue("master")},
				{Key: StrValue("message"), Value: StrValue("role updated")},
			})), true
		case "cluster.rebalance_status":
			return ResponseOk(req.ID, MapValue([]MapPair{
				{Key: StrValue("status"), Value: StrValue("idle")},
			})), true
		case "replication.status", "replication.stats", "replication.replicas_list":
			return ResponseOk(req.ID, MapValue(nil)), true
		}
		return Response{}, false
	})

	c, cancel := dialAndHello(t, srv.addr)
	defer cancel()
	defer c.Close()

	ctx := context.Background()

	rcfg, err := c.ReplicationConfigure(ctx, MapValue([]MapPair{
		{Key: StrValue("role"), Value: StrValue("master")},
	}))
	if err != nil || !rcfg.Success || rcfg.Role != "master" {
		t.Fatalf("ReplicationConfigure: %+v err=%v", rcfg, err)
	}

	rs, err := c.ClusterRebalanceStatus(ctx)
	if err != nil {
		t.Fatalf("ClusterRebalanceStatus: %v", err)
	}
	if rs.Status == nil || *rs.Status != "idle" {
		t.Fatalf("ClusterRebalanceStatus.Status = %v", rs.Status)
	}
	if rs.Message != nil {
		t.Fatalf("ClusterRebalanceStatus.Message should be nil, got %v", rs.Message)
	}

	_, err = c.ReplicationStatus(ctx)
	if err != nil {
		t.Fatalf("ReplicationStatus: %v", err)
	}
}
