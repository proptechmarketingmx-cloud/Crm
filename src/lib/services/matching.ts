import { invokeWithLocalCache, type CachedCommandResult } from '$lib/services/local-cache';

export interface MatchRequest {
	clientId: string;
	topK: number;
	maxDistanceKm: number;
}

export interface MatchScoreBreakdown {
	/** Aporte ponderado: 0.4 × similitud coseno. */
	contentSimilarity: number;
	/** Aporte ponderado: 0.3 × score CNA sin limitar. */
	cascadeScore: number;
	/** Aporte ponderado: 0.3 × score EGA. */
	egaScore: number;
}

export interface AuditedEvidence {
	evidenceType: string;
	distanceKm: number;
	sourceHash: string;
	capturedAt: string;
	consentId: string;
}

export interface PropertyMatch {
	propertyId: string;
	totalScore: number;
	distanceKm: number;
	breakdown: MatchScoreBreakdown;
	explanation: string;
	evidence: AuditedEvidence | null;
}

export interface MatchResponse {
	matches: PropertyMatch[];
}

export type MatchResult = PropertyMatch;

export interface GraphVisualizationData {
	clientId: string;
	nodes: Array<{
		id: string;
		label: string;
		activity: 'purchase' | 'visit' | 'search' | 'none';
	}>;
	edges: Array<{
		source: string;
		target: string;
		weight: number;
		signals: string[];
	}>;
}

export interface EGAHeatmapLayer {
	type: 'FeatureCollection';
	features: Array<{
		type: 'Feature';
		id: string;
		geometry: {
			type: 'Polygon';
			coordinates: number[][][];
		};
		properties: {
			score: number;
			zoneId: string;
			activity: number;
			density: number;
		};
	}>;
}

export function recommendProperties(
	request: MatchRequest
): Promise<CachedCommandResult<MatchResponse>> {
	return invokeWithLocalCache(
		'recommend_properties',
		{ request },
		`matching:${request.clientId}:${request.topK}:${request.maxDistanceKm}`
	);
}

export function getClientNetwork(
	clientId: string
): Promise<CachedCommandResult<GraphVisualizationData>> {
	return invokeWithLocalCache(
		'get_client_network',
		{ clientId },
		`network:${clientId}`
	);
}

export function getEgaHeatmap(): Promise<CachedCommandResult<EGAHeatmapLayer>> {
	return invokeWithLocalCache('get_ega_heatmap', {}, 'ega:heatmap');
}
