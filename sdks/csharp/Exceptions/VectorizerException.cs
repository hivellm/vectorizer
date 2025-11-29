namespace Vectorizer.Exceptions;

/// <summary>
/// Exception thrown by Vectorizer API operations
/// </summary>
public class VectorizerException : Exception
{
    public string ErrorType { get; }
    public int StatusCode { get; }
    public Dictionary<string, object>? Details { get; }

    public VectorizerException(
        string errorType,
        string message,
        int statusCode,
        Dictionary<string, object>? details = null)
        : base(message)
    {
        ErrorType = errorType;
        StatusCode = statusCode;
        Details = details;
    }

    public bool IsNotFound => StatusCode == 404;
    public bool IsUnauthorized => StatusCode == 401;
    public bool IsValidationError => StatusCode == 400;
}

