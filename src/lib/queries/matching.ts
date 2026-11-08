import { createQuery } from '@tanstack/svelte-query';
import { recommendProperties, type MatchRequest } from '$lib/services/matching';

export function createMatchingQuery(getRequest: () => MatchRequest) {
	return createQuery(() => {
		const request = getRequest();
		return {
			queryKey: ['local-matches', request.clientId, request.topK, request.maxDistanceKm],
			queryFn: () => recommendProperties(request),
			retry: false,
			staleTime: 0
		};
	});
}
