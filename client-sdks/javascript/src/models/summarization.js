/**
 * Summarization models for the Hive Vectorizer JavaScript SDK.
 * 
 * This module contains all the data models used for text and context summarization.
 */

export class SummarizeTextRequest {
  constructor(data) {
    this.text = data.text;
    this.method = data.method || 'extractive';
    this.max_length = data.max_length;
    this.compression_ratio = data.compression_ratio;
    this.language = data.language;
    this.metadata = data.metadata;
  }

  toJSON() {
    return {
      text: this.text,
      method: this.method,
      max_length: this.max_length,
      compression_ratio: this.compression_ratio,
      language: this.language,
      metadata: this.metadata
    };
  }
}

export class SummarizeTextResponse {
  constructor(data) {
    this.summary_id = data.summary_id;
    this.original_text = data.original_text;
    this.summary = data.summary;
    this.method = data.method;
    this.original_length = data.original_length;
    this.summary_length = data.summary_length;
    this.compression_ratio = data.compression_ratio;
    this.language = data.language;
    this.status = data.status;
    this.message = data.message;
    this.metadata = data.metadata;
  }
}

export class SummarizeContextRequest {
  constructor(data) {
    this.context = data.context;
    this.method = data.method || 'extractive';
    this.max_length = data.max_length;
    this.compression_ratio = data.compression_ratio;
    this.language = data.language;
    this.metadata = data.metadata;
  }

  toJSON() {
    return {
      context: this.context,
      method: this.method,
      max_length: this.max_length,
      compression_ratio: this.compression_ratio,
      language: this.language,
      metadata: this.metadata
    };
  }
}

export class SummarizeContextResponse {
  constructor(data) {
    this.summary_id = data.summary_id;
    this.original_context = data.original_context;
    this.summary = data.summary;
    this.method = data.method;
    this.original_length = data.original_length;
    this.summary_length = data.summary_length;
    this.compression_ratio = data.compression_ratio;
    this.language = data.language;
    this.status = data.status;
    this.message = data.message;
    this.metadata = data.metadata;
  }
}

export class GetSummaryResponse {
  constructor(data) {
    this.summary_id = data.summary_id;
    this.original_text = data.original_text;
    this.summary = data.summary;
    this.method = data.method;
    this.original_length = data.original_length;
    this.summary_length = data.summary_length;
    this.compression_ratio = data.compression_ratio;
    this.language = data.language;
    this.created_at = data.created_at;
    this.metadata = data.metadata;
    this.status = data.status;
  }
}

export class SummaryInfo {
  constructor(data) {
    this.summary_id = data.summary_id;
    this.method = data.method;
    this.language = data.language;
    this.original_length = data.original_length;
    this.summary_length = data.summary_length;
    this.compression_ratio = data.compression_ratio;
    this.created_at = data.created_at;
    this.metadata = data.metadata;
  }
}

export class ListSummariesResponse {
  constructor(data) {
    this.summaries = data.summaries.map(s => new SummaryInfo(s));
    this.total_count = data.total_count;
    this.status = data.status;
  }
}
