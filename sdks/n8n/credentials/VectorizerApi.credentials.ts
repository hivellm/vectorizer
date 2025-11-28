import {
	IAuthenticateGeneric,
	ICredentialTestRequest,
	ICredentialType,
	INodeProperties,
} from 'n8n-workflow';

export class VectorizerApi implements ICredentialType {
	name = 'vectorizerApi';
	displayName = 'Vectorizer API';
	documentationUrl = 'https://github.com/hivellm/vectorizer';
	properties: INodeProperties[] = [
		{
			displayName: 'Host',
			name: 'host',
			type: 'string',
			default: 'http://localhost:15002',
			placeholder: 'http://localhost:15002',
			description: 'The base URL of your Vectorizer instance',
		},
		{
			displayName: 'API Key',
			name: 'apiKey',
			type: 'string',
			typeOptions: {
				password: true,
			},
			default: '',
			description: 'Optional API key for authentication (if enabled on server)',
		},
	];

	authenticate: IAuthenticateGeneric = {
		type: 'generic',
		properties: {
			headers: {
				'X-API-Key': '={{$credentials.apiKey}}',
			},
		},
	};

	test: ICredentialTestRequest = {
		request: {
			baseURL: '={{$credentials.host}}',
			url: '/health',
		},
	};
}
