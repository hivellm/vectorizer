/**
 * Examples for VectorizerStore LangChain.js integration
 * 
 * This file demonstrates various usage patterns for the VectorizerStore
 * implementation with LangChain.js.
 */

import { VectorizerStore, VectorizerConfig, createVectorizerStore, VectorizerUtils } from './vectorizer-store';
import { Document } from '@langchain/core/documents';

/**
 * Basic usage example
 */
async function basicExample() {
  console.log('=== Basic VectorizerStore Example ===');
  
  // Create configuration
  const config: VectorizerConfig = {
    host: 'localhost',
    port: 15002,
    collectionName: 'example_documents',
    autoCreateCollection: true,
    batchSize: 100,
    similarityThreshold: 0.7
  };
  
  // Create store
  const store = new VectorizerStore(config);
  
  // Add some documents
  const texts = [
    'The quick brown fox jumps over the lazy dog',
    'Python is a great programming language',
    'Machine learning is transforming the world',
    'Vector databases are essential for AI applications'
  ];
  
  const metadatas = [
    { source: 'example1.txt', topic: 'animals' },
    { source: 'example2.txt', topic: 'programming' },
    { source: 'example3.txt', topic: 'technology' },
    { source: 'example4.txt', topic: 'databases' }
  ];
  
  try {
    // Add texts to store
    const vectorIds = await store.addTexts(texts, metadatas);
    console.log(`Added ${vectorIds.length} documents`);
    
    // Search for similar documents
    const query = 'programming languages';
    const results = await store.similaritySearch(query, 2);
    
    console.log(`\nSearch results for '${query}':`);
    results.forEach((doc, i) => {
      console.log(`${i + 1}. ${doc.pageContent}`);
      console.log(`   Metadata: ${JSON.stringify(doc.metadata)}`);
      console.log();
    });
    
    // Search with scores
    const resultsWithScores = await store.similaritySearchWithScore(query, 2);
    console.log(`\nSearch results with scores:`);
    resultsWithScores.forEach(([doc, score], i) => {
      console.log(`${i + 1}. Score: ${score.toFixed(3)} - ${doc.pageContent}`);
    });
    
  } catch (error) {
    console.error('Error in basic example:', error);
  }
}

/**
 * Document loading example
 */
async function documentLoadingExample() {
  console.log('\n=== Document Loading Example ===');
  
  // Create store
  const store = await createVectorizerStore(
    'localhost',
    15002,
    'file_documents'
  );
  
  // Sample documents
  const documents = [
    new Document({
      pageContent: 'Artificial intelligence is revolutionizing many industries.',
      metadata: { source: 'ai_doc.txt', category: 'technology' }
    }),
    new Document({
      pageContent: 'Natural language processing enables computers to understand human language.',
      metadata: { source: 'nlp_doc.txt', category: 'technology' }
    }),
    new Document({
      pageContent: 'Computer vision allows machines to interpret visual information.',
      metadata: { source: 'cv_doc.txt', category: 'technology' }
    })
  ];
  
  try {
    // Add documents to store
    const texts = documents.map(doc => doc.pageContent);
    const metadatas = documents.map(doc => doc.metadata);
    
    await store.addTexts(texts, metadatas);
    console.log(`Loaded ${documents.length} documents`);
    
    // Search
    const results = await store.similaritySearch('machine learning', 2);
    console.log(`\nSearch results for 'machine learning':`);
    results.forEach(doc => {
      console.log(`- ${doc.pageContent}`);
    });
    
  } catch (error) {
    console.error('Error in document loading example:', error);
  }
}

/**
 * Text splitting example
 */
async function textSplittingExample() {
  console.log('\n=== Text Splitting Example ===');
  
  // Create store
  const store = await createVectorizerStore(
    'localhost',
    15002,
    'split_documents'
  );
  
  // Long text to split
  const longText = `
    Artificial intelligence (AI) is intelligence demonstrated by machines, 
    in contrast to the natural intelligence displayed by humans and animals. 
    Leading AI textbooks define the field as the study of "intelligent agents": 
    any device that perceives its environment and takes actions that maximize 
    its chance of successfully achieving its goals. Colloquially, the term 
    "artificial intelligence" is often used to describe machines that mimic 
    "cognitive" functions that humans associate with the human mind, such as 
    "learning" and "problem solving".
  `;
  
  // Simple text splitting (in a real app, you'd use a proper text splitter)
  const chunkSize = 100;
  const overlap = 20;
  const chunks: string[] = [];
  
  for (let i = 0; i < longText.length; i += chunkSize - overlap) {
    const chunk = longText.slice(i, i + chunkSize);
    if (chunk.trim()) {
      chunks.push(chunk.trim());
    }
  }
  
  try {
    // Add chunks to store
    const metadatas = chunks.map((_, i) => ({ 
      chunkId: i, 
      source: 'ai_text.txt',
      totalChunks: chunks.length
    }));
    
    await store.addTexts(chunks, metadatas);
    console.log(`Split text into ${chunks.length} chunks`);
    
    // Search for specific information
    const results = await store.similaritySearch('machine learning', 3);
    console.log(`\nSearch results for 'machine learning':`);
    results.forEach((doc, i) => {
      console.log(`${i + 1}. ${doc.pageContent.substring(0, 100)}...`);
    });
    
  } catch (error) {
    console.error('Error in text splitting example:', error);
  }
}

/**
 * Metadata filtering example
 */
async function metadataFilteringExample() {
  console.log('\n=== Metadata Filtering Example ===');
  
  // Create store
  const store = await createVectorizerStore(
    'localhost',
    15002,
    'filtered_documents'
  );
  
  try {
    // Add documents with different metadata
    const texts = [
      'Python is a versatile programming language',
      'Java is widely used in enterprise applications',
      'JavaScript powers modern web applications',
      'C++ is used for system programming'
    ];
    
    const metadatas = [
      { language: 'python', type: 'programming', year: 2023 },
      { language: 'java', type: 'programming', year: 2023 },
      { language: 'javascript', type: 'web', year: 2023 },
      { language: 'cpp', type: 'system', year: 2022 }
    ];
    
    await store.addTexts(texts, metadatas);
    
    // Search without filter
    console.log('Search without filter:');
    const results = await store.similaritySearch('programming', 4);
    results.forEach(doc => {
      console.log(`- ${doc.pageContent}`);
    });
    
    // Search with metadata filter
    console.log('\nSearch with filter (type=programming):');
    const filterDict = { type: 'programming' };
    const filteredResults = await store.similaritySearch('programming', 4, filterDict);
    filteredResults.forEach(doc => {
      console.log(`- ${doc.pageContent}`);
    });
    
  } catch (error) {
    console.error('Error in metadata filtering example:', error);
  }
}

/**
 * Batch operations example
 */
async function batchOperationsExample() {
  console.log('\n=== Batch Operations Example ===');
  
  // Create store
  const store = await createVectorizerStore(
    'localhost',
    15002,
    'batch_documents'
  );
  
  try {
    // Large batch of documents
    const texts = Array.from({ length: 100 }, (_, i) => 
      `Document ${i}: This is sample content for document number ${i}`
    );
    const metadatas = Array.from({ length: 100 }, (_, i) => ({ 
      docId: i, 
      batch: 'example' 
    }));
    
    // Add in batch
    const vectorIds = await store.addTexts(texts, metadatas);
    console.log(`Added ${vectorIds.length} documents in batch`);
    
    // Search
    const results = await store.similaritySearch('sample content', 5);
    console.log(`Found ${results.length} similar documents`);
    
    // Delete some documents
    const idsToDelete = vectorIds.slice(0, 10); // Delete first 10
    const success = await store.delete(idsToDelete);
    console.log(`Deleted ${idsToDelete.length} documents: ${success}`);
    
  } catch (error) {
    console.error('Error in batch operations example:', error);
  }
}

/**
 * Configuration validation example
 */
async function configurationExample() {
  console.log('\n=== Configuration Example ===');
  
  try {
    // Validate configuration
    const config = VectorizerUtils.createDefaultConfig({
      host: 'localhost',
      port: 15002,
      collectionName: 'config_test',
      batchSize: 50,
      similarityThreshold: 0.8
    });
    
    VectorizerUtils.validateConfig(config);
    console.log('Configuration is valid:', config);
    
    // Check availability
    const isAvailable = await VectorizerUtils.checkAvailability(config);
    console.log(`Vectorizer is available: ${isAvailable}`);
    
  } catch (error) {
    console.error('Error in configuration example:', error);
  }
}

/**
 * Error handling example
 */
async function errorHandlingExample() {
  console.log('\n=== Error Handling Example ===');
  
  try {
    // Try to connect to non-existent server
    const store = await createVectorizerStore(
      'non-existent-host',
      15002,
      'error_test'
    );
    
    await store.addTexts(['test text'], [{ test: true }]);
    
  } catch (error) {
    console.log('Caught expected error:', error.message);
  }
}

/**
 * Run all examples
 */
async function runAllExamples() {
  console.log('VectorizerStore LangChain.js Integration Examples');
  console.log('='.repeat(50));
  
  try {
    await basicExample();
    await documentLoadingExample();
    await textSplittingExample();
    await metadataFilteringExample();
    await batchOperationsExample();
    await configurationExample();
    await errorHandlingExample();
    
    console.log('\n' + '='.repeat(50));
    console.log('All examples completed successfully!');
    
  } catch (error) {
    console.error('Error running examples:', error);
    console.log('Make sure Vectorizer is running on localhost:15002');
  }
}

// Export functions for individual testing
export {
  basicExample,
  documentLoadingExample,
  textSplittingExample,
  metadataFilteringExample,
  batchOperationsExample,
  configurationExample,
  errorHandlingExample,
  runAllExamples
};

// Run examples if this file is executed directly
if (require.main === module) {
  runAllExamples().catch(console.error);
}
