export type Pt = { x: number; y: number };

/**
 * Lay out points at the centers of N equal columns spanning [0, width].
 * So point i sits at x = (i + 0.5) * (width / N) — matching a 1fr grid of N
 * labels rendered with text-center underneath.
 */
export function computePositions(values: number[], width: number, height: number): Pt[] {
	if (values.length === 0) return [];
	const min = Math.min(...values);
	const max = Math.max(...values);
	const range = max - min || 1;
	const stepX = width / values.length;
	return values.map((v, i) => ({
		x: (i + 0.5) * stepX,
		y: height - ((v - min) / range) * height
	}));
}

export function pointsToPath(points: Pt[]): string {
	return points.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ');
}

export function pointsToArea(points: Pt[], width: number, height: number): string {
	if (points.length === 0) return `0,${height} ${width.toFixed(1)},${height}`;
	const first = points[0];
	const last = points[points.length - 1];
	return `${first.x.toFixed(1)},${height} ${pointsToPath(points)} ${last.x.toFixed(1)},${height}`;
}

function shiftAmount(prev: string[], next: string[], direction: 1 | -1): number | null {
	const n = prev.length;
	if (n === 0 || n !== next.length) return null;
	for (let s = 1; s < n; s++) {
		let ok = true;
		const overlap = n - s;
		for (let i = 0; i < overlap; i++) {
			const lhs = direction === 1 ? next[i] : next[i + s];
			const rhs = direction === 1 ? prev[i + s] : prev[i];
			if (lhs !== rhs) {
				ok = false;
				break;
			}
		}
		if (ok) return s;
	}
	return null;
}

/**
 * Build a (start, end) tween pair whose lengths match. The array order matches
 * left-to-right x, so the polyline path interpolates monotonically.
 *
 * Cases:
 *   - identical labels: value change → start = prev, end = new.
 *   - same length, shifted forward (newer window): leaving on left exit off-screen,
 *     shared slide left, entering on right slide in from off-screen right.
 *   - same length, shifted backward (older window): mirror of above.
 *   - newLen > oldLen: granularity finer → each new point starts at its parent
 *     old position and spreads out.
 *   - newLen < oldLen: granularity coarser → keep oldLen entries throughout so
 *     each old point can slide into its cluster's new position.
 *   - same length, no detectable shift: element-wise tween.
 */
export function computeAnim(
	prevLabels: string[],
	prevPositions: Pt[],
	newLabels: string[],
	newPositions: Pt[],
	width: number
): { start: Pt[]; end: Pt[] } {
	if (prevLabels.length === 0 || prevPositions.length !== prevLabels.length) {
		return { start: newPositions, end: newPositions };
	}
	const oldLen = prevLabels.length;
	const newLen = newLabels.length;

	if (oldLen === newLen) {
		const allMatch = prevLabels.every((l, i) => l === newLabels[i]);
		if (allMatch) return { start: prevPositions, end: newPositions };

		const stepX = newLen > 0 ? width / newLen : width;

		const fwd = shiftAmount(prevLabels, newLabels, 1);
		if (fwd !== null) {
			const start: Pt[] = [];
			const end: Pt[] = [];
			const rightEdgeY = prevPositions[prevPositions.length - 1].y;
			for (let i = 0; i < fwd; i++) {
				start.push(prevPositions[i]);
				end.push({ x: prevPositions[i].x - fwd * stepX, y: prevPositions[i].y });
			}
			for (let j = 0; j < newLen; j++) {
				if (j < newLen - fwd) {
					start.push(prevPositions[j + fwd]);
				} else {
					const offset = j - (newLen - fwd);
					start.push({ x: width + (offset + 0.5) * stepX, y: rightEdgeY });
				}
				end.push(newPositions[j]);
			}
			return { start, end };
		}

		const back = shiftAmount(prevLabels, newLabels, -1);
		if (back !== null) {
			const start: Pt[] = [];
			const end: Pt[] = [];
			const leftEdgeY = prevPositions[0].y;
			for (let j = 0; j < newLen; j++) {
				if (j < back) {
					start.push({ x: (j + 0.5 - back) * stepX, y: leftEdgeY });
				} else {
					start.push(prevPositions[j - back]);
				}
				end.push(newPositions[j]);
			}
			for (let i = 0; i < back; i++) {
				const srcIdx = newLen - back + i;
				start.push(prevPositions[srcIdx]);
				end.push({ x: prevPositions[srcIdx].x + back * stepX, y: prevPositions[srcIdx].y });
			}
			return { start, end };
		}

		return { start: prevPositions, end: newPositions };
	}

	if (newLen > oldLen) {
		const ratio = newLen / oldLen;
		const start: Pt[] = newPositions.map((_, i) => {
			const parentIdx = Math.min(oldLen - 1, Math.floor(i / ratio));
			return prevPositions[parentIdx];
		});
		return { start, end: newPositions };
	}

	const ratio = oldLen / newLen;
	const end: Pt[] = prevPositions.map((_, i) => {
		const targetIdx = Math.min(newLen - 1, Math.floor(i / ratio));
		return newPositions[targetIdx];
	});
	return { start: prevPositions, end };
}
