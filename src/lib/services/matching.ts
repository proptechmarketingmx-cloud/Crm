import { invoke } from '@tauri-apps/api/core';

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

export function recommendProperties(request: MatchRequest): Promise<MatchResponse> {
	return invoke<MatchResponse>('recommend_properties', { request });
}
