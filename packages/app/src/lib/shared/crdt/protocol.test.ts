import { describe, expect, it } from 'vitest';
import { decodeFrame, encodeFrame, type Frame } from './protocol';

describe('sync frame codec', () => {
	it('round-trips each frame kind', () => {
		const frames: Frame[] = [
			{ kind: 'snapshot-request' },
			{ kind: 'snapshot', payload: new Uint8Array([1, 2, 3]) },
			{ kind: 'update', payload: new Uint8Array([9, 8]) },
			{ kind: 'presence', payload: new Uint8Array([5, 6, 7]) }
		];
		for (const frame of frames) {
			expect(decodeFrame(encodeFrame(frame))).toEqual(frame);
		}
	});

	it('rejects empty input and unknown tags', () => {
		expect(decodeFrame(new Uint8Array([]))).toBeNull();
		expect(decodeFrame(new Uint8Array([0xff, 1, 2]))).toBeNull();
		// The next unused tag stays unknown, so future tags are forward compatible.
		expect(decodeFrame(new Uint8Array([0x05, 1, 2]))).toBeNull();
	});
});
