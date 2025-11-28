import { IExecuteFunctions } from 'n8n-core';
import {
	INodeExecutionData,
	INodeType,
	INodeTypeDescription,
	NodeOperationError,
} from 'n8n-workflow';

import axios, { AxiosRequestConfig } from 'axios';

export class Vectorizer implements INodeType {
	description: INodeTypeDescription = {
		displayName: 'Vectorizer',
		name: 'vectorizer',
		icon: 'file:vectorizer.svg',
		group: ['transform'],
		version: 1,
		subtitle: '={{$parameter["operation"] + ": " + $parameter["resource"]}}',
		description: 'Interact with Vectorizer vector database',
		defaults: {
			name: 'Vectorizer',
		},
		inputs: ['main'],
		outputs: ['main'],
		credentials: [
			{
				name: 'vectorizerApi',
				required: true,
			},
		],
		properties: [
			{
				displayName: 'Resource',
				name: 'resource',
				type: 'options',
				noDataExpression: true,
				options: [
					{
						name: 'Collection',
						value: 'collection',
					},
					{
						name: 'Vector',
						value: 'vector',
					},
					{
						name: 'Search',
						value: 'search',
					},
				],
				default: 'collection',
			},
			// Collection Operations
			{
				displayName: 'Operation',
				name: 'operation',
				type: 'options',
				noDataExpression: true,
				displayOptions: {
					show: {
						resource: ['collection'],
					},
				},
				options: [
					{
						name: 'Create',
						value: 'create',
						description: 'Create a new collection',
						action: 'Create a collection',
					},
					{
						name: 'Delete',
						value: 'delete',
						description: 'Delete a collection',
						action: 'Delete a collection',
					},
					{
						name: 'Get',
						value: 'get',
						description: 'Get collection information',
						action: 'Get a collection',
					},
					{
						name: 'List',
						value: 'list',
						description: 'List all collections',
						action: 'List collections',
					},
				],
				default: 'list',
			},
			// Vector Operations
			{
				displayName: 'Operation',
				name: 'operation',
				type: 'options',
				noDataExpression: true,
				displayOptions: {
					show: {
						resource: ['vector'],
					},
				},
				options: [
					{
						name: 'Insert',
						value: 'insert',
						description: 'Insert a vector',
						action: 'Insert a vector',
					},
					{
						name: 'Batch Insert',
						value: 'batchInsert',
						description: 'Insert multiple vectors',
						action: 'Batch insert vectors',
					},
					{
						name: 'Delete',
						value: 'delete',
						description: 'Delete a vector',
						action: 'Delete a vector',
					},
					{
						name: 'Get',
						value: 'get',
						description: 'Get a vector by ID',
						action: 'Get a vector',
					},
				],
				default: 'insert',
			},
			// Search Operations
			{
				displayName: 'Operation',
				name: 'operation',
				type: 'options',
				noDataExpression: true,
				displayOptions: {
					show: {
						resource: ['search'],
					},
				},
				options: [
					{
						name: 'Vector Search',
						value: 'vectorSearch',
						description: 'Search by vector',
						action: 'Vector search',
					},
					{
						name: 'Semantic Search',
						value: 'semanticSearch',
						description: 'Search by text query',
						action: 'Semantic search',
					},
					{
						name: 'Hybrid Search',
						value: 'hybridSearch',
						description: 'Combine vector and keyword search',
						action: 'Hybrid search',
					},
				],
				default: 'semanticSearch',
			},
			// Collection Fields
			{
				displayName: 'Collection Name',
				name: 'collectionName',
				type: 'string',
				required: true,
				displayOptions: {
					show: {
						resource: ['collection'],
						operation: ['create', 'delete', 'get'],
					},
				},
				default: '',
				description: 'Name of the collection',
			},
			{
				displayName: 'Dimension',
				name: 'dimension',
				type: 'number',
				required: true,
				displayOptions: {
					show: {
						resource: ['collection'],
						operation: ['create'],
					},
				},
				default: 384,
				description: 'Vector dimension size',
			},
			{
				displayName: 'Distance Metric',
				name: 'metric',
				type: 'options',
				displayOptions: {
					show: {
						resource: ['collection'],
						operation: ['create'],
					},
				},
				options: [
					{
						name: 'Cosine',
						value: 'cosine',
					},
					{
						name: 'Euclidean',
						value: 'euclidean',
					},
					{
						name: 'Dot Product',
						value: 'dot',
					},
				],
				default: 'cosine',
				description: 'Distance metric for similarity calculation',
			},
			// Vector Fields
			{
				displayName: 'Collection Name',
				name: 'collectionName',
				type: 'string',
				required: true,
				displayOptions: {
					show: {
						resource: ['vector', 'search'],
					},
				},
				default: '',
				description: 'Name of the collection',
			},
			{
				displayName: 'Vector ID',
				name: 'vectorId',
				type: 'string',
				required: true,
				displayOptions: {
					show: {
						resource: ['vector'],
						operation: ['insert', 'delete', 'get'],
					},
				},
				default: '',
				description: 'Unique identifier for the vector',
			},
			{
				displayName: 'Vector Data',
				name: 'vectorData',
				type: 'string',
				required: true,
				displayOptions: {
					show: {
						resource: ['vector'],
						operation: ['insert'],
					},
				},
				default: '',
				placeholder: '[0.1, 0.2, 0.3, ...]',
				description: 'Vector values as JSON array',
			},
			{
				displayName: 'Vectors',
				name: 'vectors',
				type: 'string',
				required: true,
				displayOptions: {
					show: {
						resource: ['vector'],
						operation: ['batchInsert'],
					},
				},
				default: '',
				placeholder: '[{"id": "vec1", "vector": [0.1, 0.2]}, ...]',
				description: 'Array of vectors to insert as JSON',
			},
			{
				displayName: 'Payload',
				name: 'payload',
				type: 'string',
				displayOptions: {
					show: {
						resource: ['vector'],
						operation: ['insert', 'batchInsert'],
					},
				},
				default: '{}',
				description: 'Metadata for the vector as JSON object',
			},
			// Search Fields
			{
				displayName: 'Query',
				name: 'query',
				type: 'string',
				required: true,
				displayOptions: {
					show: {
						resource: ['search'],
						operation: ['semanticSearch', 'hybridSearch'],
					},
				},
				default: '',
				description: 'Search query text',
			},
			{
				displayName: 'Query Vector',
				name: 'queryVector',
				type: 'string',
				required: true,
				displayOptions: {
					show: {
						resource: ['search'],
						operation: ['vectorSearch'],
					},
				},
				default: '',
				placeholder: '[0.1, 0.2, 0.3, ...]',
				description: 'Query vector as JSON array',
			},
			{
				displayName: 'Limit',
				name: 'limit',
				type: 'number',
				displayOptions: {
					show: {
						resource: ['search'],
					},
				},
				default: 10,
				description: 'Maximum number of results to return',
			},
			{
				displayName: 'Score Threshold',
				name: 'scoreThreshold',
				type: 'number',
				displayOptions: {
					show: {
						resource: ['search'],
					},
				},
				default: 0,
				description: 'Minimum similarity score (0-1)',
			},
		],
	};

	async execute(this: IExecuteFunctions): Promise<INodeExecutionData[][]> {
		const items = this.getInputData();
		const returnData: INodeExecutionData[] = [];
		const credentials = await this.getCredentials('vectorizerApi');
		const baseURL = credentials.host as string;
		const apiKey = credentials.apiKey as string;

		for (let i = 0; i < items.length; i++) {
			try {
				const resource = this.getNodeParameter('resource', i) as string;
				const operation = this.getNodeParameter('operation', i) as string;

				const config: AxiosRequestConfig = {
					baseURL,
					headers: {
						'Content-Type': 'application/json',
						...(apiKey && { 'X-API-Key': apiKey }),
					},
				};

				let responseData: any;

				if (resource === 'collection') {
					if (operation === 'create') {
						const name = this.getNodeParameter('collectionName', i) as string;
						const dimension = this.getNodeParameter('dimension', i) as number;
						const metric = this.getNodeParameter('metric', i, 'cosine') as string;

						const response = await axios.post(
							`/collections`,
							{
								name,
								dimension,
								metric,
							},
							config,
						);
						responseData = response.data;
					} else if (operation === 'delete') {
						const name = this.getNodeParameter('collectionName', i) as string;
						const response = await axios.delete(`/collections/${name}`, config);
						responseData = response.data;
					} else if (operation === 'get') {
						const name = this.getNodeParameter('collectionName', i) as string;
						const response = await axios.get(`/collections/${name}`, config);
						responseData = response.data;
					} else if (operation === 'list') {
						const response = await axios.get(`/collections`, config);
						responseData = response.data;
					}
				} else if (resource === 'vector') {
					const collectionName = this.getNodeParameter('collectionName', i) as string;

					if (operation === 'insert') {
						const vectorId = this.getNodeParameter('vectorId', i) as string;
						const vectorDataStr = this.getNodeParameter('vectorData', i) as string;
						const payloadStr = this.getNodeParameter('payload', i, '{}') as string;

						const vector = JSON.parse(vectorDataStr);
						const payload = JSON.parse(payloadStr);

						const response = await axios.post(
							`/collections/${collectionName}/insert`,
							{
								id: vectorId,
								vector,
								payload,
							},
							config,
						);
						responseData = response.data;
					} else if (operation === 'batchInsert') {
						const vectorsStr = this.getNodeParameter('vectors', i) as string;
						const vectors = JSON.parse(vectorsStr);

						const response = await axios.post(
							`/collections/${collectionName}/batch_insert`,
							{ vectors },
							config,
						);
						responseData = response.data;
					} else if (operation === 'delete') {
						const vectorId = this.getNodeParameter('vectorId', i) as string;
						const response = await axios.delete(
							`/collections/${collectionName}/vectors/${vectorId}`,
							config,
						);
						responseData = response.data;
					} else if (operation === 'get') {
						const vectorId = this.getNodeParameter('vectorId', i) as string;
						const response = await axios.get(
							`/collections/${collectionName}/vectors/${vectorId}`,
							config,
						);
						responseData = response.data;
					}
				} else if (resource === 'search') {
					const collectionName = this.getNodeParameter('collectionName', i) as string;
					const limit = this.getNodeParameter('limit', i, 10) as number;
					const scoreThreshold = this.getNodeParameter('scoreThreshold', i, 0) as number;

					if (operation === 'semanticSearch') {
						const query = this.getNodeParameter('query', i) as string;

						const response = await axios.post(
							`/collections/${collectionName}/search/semantic`,
							{
								query,
								limit,
								score_threshold: scoreThreshold,
							},
							config,
						);
						responseData = response.data;
					} else if (operation === 'vectorSearch') {
						const queryVectorStr = this.getNodeParameter('queryVector', i) as string;
						const queryVector = JSON.parse(queryVectorStr);

						const response = await axios.post(
							`/collections/${collectionName}/search`,
							{
								vector: queryVector,
								limit,
								score_threshold: scoreThreshold,
							},
							config,
						);
						responseData = response.data;
					} else if (operation === 'hybridSearch') {
						const query = this.getNodeParameter('query', i) as string;

						const response = await axios.post(
							`/collections/${collectionName}/search/hybrid`,
							{
								query,
								limit,
								score_threshold: scoreThreshold,
							},
							config,
						);
						responseData = response.data;
					}
				}

				if (Array.isArray(responseData)) {
					returnData.push(...responseData.map((item) => ({ json: item })));
				} else {
					returnData.push({ json: responseData });
				}
			} catch (error) {
				if (this.continueOnFail()) {
					returnData.push({
						json: {
							error: error.message,
						},
						pairedItem: {
							item: i,
						},
					});
					continue;
				}
				throw new NodeOperationError(this.getNode(), error.message, { itemIndex: i });
			}
		}

		return [returnData];
	}
}
