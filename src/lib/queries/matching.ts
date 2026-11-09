import { createQuery } from '@tanstack/svelte-query';
import {
	getClientNetwork,
	getEgaHeatmap,
	recommendProperties,
	type MatchRequest
} from '$lib/services/matching';

export function useMatchingQuery(getRequest: () => MatchRequest) {
	return createQuery(() => {
		const request = getRequest();
		return {
			queryKey: [
				'local-matches',
				request.clientId,
				request.topK,
				request.maxDistanceKm
			],
			queryFn: () => recommendProperties(request),
			enabled: request.clientId.length > 0,
			retry: false,
			staleTime: 30_000
		};
	});
}

export function createClientNetworkQuery(getClientId: () => string) {
	return createQuery(() => {
		const clientId = getClientId();
		return {
			queryKey: ['client-network', clientId],
			queryFn: () => getClientNetwork(clientId),
			enabled: clientId.length > 0,
			retry: false,
			staleTime: 30_000
		};
	});
}

export function createEgaHeatmapQuery() {
	return createQuery(() => ({
		queryKey: ['ega-heatmap'],
		queryFn: getEgaHeatmap,
		retry: false,
		staleTime: 60_000
	}));
}
