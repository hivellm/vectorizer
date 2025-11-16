package vectorizer

import "fmt"

// ErrorResponse represents an error response from the API
type ErrorResponse struct {
	ErrorType string                 `json:"error_type"`
	Message   string                 `json:"message"`
	Details   map[string]interface{} `json:"details,omitempty"`
	Status    int                    `json:"status_code,omitempty"`
}

// VectorizerError represents a Vectorizer API error
type VectorizerError struct {
	Type    string
	Message string
	Status  int
	Details map[string]interface{}
}

func (e *VectorizerError) Error() string {
	if e.Details != nil && len(e.Details) > 0 {
		return fmt.Sprintf("%s: %s (status: %d, details: %v)", e.Type, e.Message, e.Status, e.Details)
	}
	return fmt.Sprintf("%s: %s (status: %d)", e.Type, e.Message, e.Status)
}

// IsNotFound checks if the error is a not found error
func (e *VectorizerError) IsNotFound() bool {
	return e.Status == 404
}

// IsUnauthorized checks if the error is an unauthorized error
func (e *VectorizerError) IsUnauthorized() bool {
	return e.Status == 401
}

// IsValidationError checks if the error is a validation error
func (e *VectorizerError) IsValidationError() bool {
	return e.Status == 400
}

