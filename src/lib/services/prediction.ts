import { invoke } from '@tauri-apps/api/core';

export interface PredictionQueryInput {
	/** Token opaco que cambia con cada conjunto de parámetros; no usar datos personales. */
	cacheKey: string;
	request: PredictionRequest;
}

export type ActivityType = 'purchase' | 'visit' | 'search';

export interface GraphEdge {
	source: string;
	target: string;
	weight: number;
	signals?: Array<
		| { type: 'sameEmployer'; employer: string }
		| { type: 'sameSurname'; score: number }
		| { type: 'sameVisitZone'; zoneId: string }
		| { type: 'confirmedInteraction'; interaction: 'call' | 'message' | 'visit' | 'referral' }
		| { type: 'profileSimilarity'; cosine: number }
	>;
}

export interface PredictionRequest {
	clientId: string;
	event: { clientId: string; activity: ActivityType };
	edges: GraphEdge[];
	activities: Array<{ clientId: string; activity: ActivityType }>;
	clientEmbedding: number[];
	propertyEmbedding: number[];
	egaScore: number;
	deltaT: number;
	parameters?: { alpha: number; beta: number; gamma: number; lambda: number };
}

export interface PredictionResult {
	cascadeScore: number;
	purchaseProbability: number;
	networkContribution: number;
	geographicContribution: number;
	temporalContribution: number;
	cosineSimilarity: number;
	matchScore: number;
}

export function predictMatch(request: PredictionRequest): Promise<PredictionResult> {
	const localRequest: PredictionRequest = {
		...request,
		edges: request.edges.map(({ source, target, weight }) => ({ source, target, weight }))
	};
	return invoke<PredictionResult>('predict_match', { request: localRequest });
}
