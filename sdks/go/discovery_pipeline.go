package vectorizer

// BroadDiscovery runs a multi-query broad search across all collections.
//
// POST /discovery/broad_discovery
func (c *Client) BroadDiscovery(req *BroadDiscoveryRequest) (*BroadDiscoveryResponse, error) {
	var result BroadDiscoveryResponse
	if err := c.request("POST", "/discovery/broad_discovery", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// SemanticFocus runs a focused semantic search within a single collection.
//
// POST /discovery/semantic_focus
func (c *Client) SemanticFocus(req *SemanticFocusRequest) (*SemanticFocusResponse, error) {
	var result SemanticFocusResponse
	if err := c.request("POST", "/discovery/semantic_focus", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// PromoteReadme promotes README-quality chunks to the top of a result set.
//
// POST /discovery/promote_readme
func (c *Client) PromoteReadme(req *PromoteReadmeRequest) (*PromoteReadmeResponse, error) {
	var result PromoteReadmeResponse
	if err := c.request("POST", "/discovery/promote_readme", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// CompressEvidence compresses a chunk set into a concise bullet list.
//
// POST /discovery/compress_evidence
func (c *Client) CompressEvidence(req *CompressEvidenceRequest) (*CompressEvidenceResponse, error) {
	var result CompressEvidenceResponse
	if err := c.request("POST", "/discovery/compress_evidence", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// BuildAnswerPlan organises bullets into a structured answer plan.
//
// POST /discovery/build_answer_plan
func (c *Client) BuildAnswerPlan(req *AnswerPlanRequest) (*AnswerPlan, error) {
	var result AnswerPlan
	if err := c.request("POST", "/discovery/build_answer_plan", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}

// RenderLlmPrompt renders an answer plan into a final LLM prompt string.
//
// POST /discovery/render_llm_prompt
func (c *Client) RenderLlmPrompt(req *RenderPromptRequest) (*LlmPrompt, error) {
	var result LlmPrompt
	if err := c.request("POST", "/discovery/render_llm_prompt", req, &result); err != nil {
		return nil, err
	}
	return &result, nil
}
