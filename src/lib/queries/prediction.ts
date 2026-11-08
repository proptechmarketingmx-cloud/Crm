import { createQuery } from '@tanstack/svelte-query';
import { predictMatch, type PredictionQueryInput } from '$lib/services/prediction';

export function createPredictionQuery(getInput: () => PredictionQueryInput) {
	return createQuery(() => {
		const { cacheKey, request } = getInput();
		return {
			queryKey: ['local-prediction', cacheKey],
			queryFn: () => predictMatch(request),
			retry: false,
			staleTime: 0
		};
	});
}
