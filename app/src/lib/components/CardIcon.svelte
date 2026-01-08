<script lang="ts">
	import type { CardType } from '$lib/types';

	interface Props {
		type: CardType;
		size?: number;
	}

	const { type, size = 20 }: Props = $props();

	function renderIcon(cardType: CardType, iconSize: number): string {
		const s = iconSize;
		const h = s / 2;
		const ns = 'http://www.w3.org/2000/svg';

		const svgStart = `<svg viewBox="0 0 ${s} ${s}" width="${s}" height="${s}" xmlns="${ns}">`;
		const svgEnd = '</svg>';

		switch (cardType) {
			case 'mission':
			case 'requirement':
			case 'driver':
				// Beveled square
				const pts1 = `${s * 0.2},0 ${s},0 ${s},${s * 0.8} ${s * 0.8},${s} 0,${s} 0,${s * 0.2}`;
				return `${svgStart}<polygon points="${pts1}" fill="currentColor"/>${svgEnd}`;

			case 'behavior':
				// Oval
				return `${svgStart}<ellipse cx="${h}" cy="${h}" rx="${s * 0.4}" ry="${s * 0.35}" fill="currentColor"/>${svgEnd}`;

			case 'constraint':
				// Elongated octagon
				const pts2 = `${s * 0.2},0 ${s * 0.8},0 ${s},${s * 0.25} ${s},${s * 0.75} ${s * 0.8},${s} ${s * 0.2},${s} 0,${s * 0.75} 0,${s * 0.25}`;
				return `${svgStart}<polygon points="${pts2}" fill="currentColor"/>${svgEnd}`;

			case 'logical-component':
				// 3D shadow
				return `${svgStart}<rect x="${s * 0.15}" y="${s * 0.6}" width="${s * 0.6}" height="${s * 0.25}" fill="currentColor" opacity="0.3"/><rect x="${s * 0.1}" y="${s * 0.15}" width="${s * 0.65}" height="${s * 0.55}" fill="currentColor"/>${svgEnd}`;

			case 'deployable-node':
				// Server with legs
				return `${svgStart}<rect x="${s * 0.15}" y="${s * 0.2}" width="${s * 0.7}" height="${s * 0.5}" fill="currentColor"/><line x1="${s * 0.2}" y1="${s * 0.72}" x2="${s * 0.35}" y2="${s * 0.85}" stroke="currentColor" stroke-width="${s * 0.08}"/><line x1="${s * 0.65}" y1="${s * 0.72}" x2="${s * 0.8}" y2="${s * 0.85}" stroke="currentColor" stroke-width="${s * 0.08}"/>${svgEnd}`;

			case 'actor':
				// Stick figure with computer head
				return `${svgStart}<circle cx="${h}" cy="${s * 0.2}" r="${s * 0.12}" fill="currentColor"/><rect x="${s * 0.08}" y="${s * 0.08}" width="${s * 0.84}" height="${s * 0.24}" fill="none" stroke="currentColor" stroke-width="${s * 0.06}"/><line x1="${h}" y1="${s * 0.32}" x2="${h}" y2="${s * 0.55}" stroke="currentColor" stroke-width="${s * 0.08}"/><line x1="${s * 0.2}" y1="${s * 0.4}" x2="${s * 0.8}" y2="${s * 0.4}" stroke="currentColor" stroke-width="${s * 0.08}"/><line x1="${s * 0.25}" y1="${s * 0.55}" x2="${s * 0.1}" y2="${s * 0.8}" stroke="currentColor" stroke-width="${s * 0.08}"/><line x1="${s * 0.75}" y1="${s * 0.55}" x2="${s * 0.9}" y2="${s * 0.8}" stroke="currentColor" stroke-width="${s * 0.08}"/>${svgEnd}`;

			case 'test':
				// Elongated hexagon
				const pts3 = `${s * 0.15},0 ${s * 0.85},0 ${s},${h} ${s * 0.85},${s} ${s * 0.15},${s} 0,${h}`;
				return `${svgStart}<polygon points="${pts3}" fill="currentColor"/>${svgEnd}`;

			case 'artifact':
				// Document
				return `${svgStart}<rect x="${s * 0.2}" y="${s * 0.1}" width="${s * 0.6}" height="${s * 0.8}" fill="currentColor"/><line x1="${s * 0.3}" y1="${s * 0.3}" x2="${s * 0.7}" y2="${s * 0.3}" stroke="white" stroke-width="${s * 0.06}"/><line x1="${s * 0.3}" y1="${s * 0.45}" x2="${s * 0.7}" y2="${s * 0.45}" stroke="white" stroke-width="${s * 0.06}"/><line x1="${s * 0.3}" y1="${s * 0.6}" x2="${s * 0.7}" y2="${s * 0.6}" stroke="white" stroke-width="${s * 0.06}"/>${svgEnd}`;

			case 'view':
				// Double circle
				return `${svgStart}<circle cx="${s * 0.35}" cy="${h}" r="${s * 0.2}" fill="currentColor"/><circle cx="${s * 0.65}" cy="${h}" r="${s * 0.2}" fill="currentColor"/>${svgEnd}`;

			case 'interface':
				// Elongated circle on stick
				return `${svgStart}<ellipse cx="${h}" cy="${s * 0.3}" rx="${s * 0.25}" ry="${s * 0.2}" fill="currentColor"/><line x1="${h}" y1="${s * 0.5}" x2="${h}" y2="${s * 0.85}" stroke="currentColor" stroke-width="${s * 0.08}"/>${svgEnd}`;

			case 'note':
				// Note card
				return `${svgStart}<path d="M ${s * 0.15} 0 L ${s * 0.85} 0 L ${s * 0.85} ${s * 0.15} L ${s} ${s * 0.15} L ${s} ${s} L ${s * 0.15} ${s} Z" fill="currentColor"/><polygon points="${s * 0.75},0 ${s},${s * 0.15} ${s * 0.85},${s * 0.15}" fill="white" opacity="0.3"/>${svgEnd}`;

			default:
				return `${svgStart}<circle cx="${h}" cy="${h}" r="${s * 0.35}" fill="currentColor"/>${svgEnd}`;
		}
	}
</script>

{@html renderIcon(type, size)}

<style>
	:global(svg) {
		color: inherit;
		display: inline-block;
		vertical-align: middle;
	}
</style>
