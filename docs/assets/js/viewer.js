// Minimal helper: load local mermaid if present, otherwise attempt CDN
function loadScript(src) {
	return new Promise((resolve, reject) => {
		const s = document.createElement('script');
		s.src = src;
		s.async = true;
		s.onload = () => resolve();
		s.onerror = () => reject(new Error('Failed to load ' + src));
		document.head.appendChild(s);
	});
}

async function ensureMermaid() {
	if (window.mermaid) return;
	// Try local copy first
	try {
		await loadScript('./mermaid.min.js');
		console.log('Loaded local mermaid');
	} catch (e) {
		console.warn('Local mermaid not found, trying CDN...');
		try {
			await loadScript('https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js');
			console.log('Loaded CDN mermaid');
		} catch (e2) {
			console.warn('CDN mermaid failed');
		}
	}
	if (window.mermaid) {
		try {
			mermaid.initialize({
				startOnLoad: false,
				theme: document.documentElement.getAttribute('data-theme') === 'light' ? 'forest' : 'dark',
				securityLevel: 'loose',
				flowchart: {
					htmlLabels: true,
					useMaxWidth: false,
					// prefer ELK layout when available
					layout: 'elk',
					// spacing tuned to font size
					nodeSpacing: 16,
					rankSpacing: 32,
					diagramPadding: 8,
				},
			});
		} catch (e) {
			console.warn(e);
		}
	}
}

// App state
const cards = {}; // uuid -> card object
const idToMermaid = {}; // uuid -> mermaid node id

const viewDefs = [
	'All Cards',
	'Requirements Traceability',
	'Capability Map',
	'Component Architecture',
	'Interface / API Surface',
	'Data Flow and Storage',
	'Threat Model and Controls',
	'Deployment Topology',
	'Action / Process Flow',
	'State Diagram',
	'Communication Diagram',
	'By Type',
];

// current included cards for the active view
let currentIncludedCards = [];

const viewSelect = document.getElementById('viewSelect');
viewDefs.forEach((v) => {
	const o = document.createElement('option');
	o.value = v;
	o.textContent = v;
	viewSelect.appendChild(o);
});

document.getElementById('themeToggle').addEventListener('click', () => {
	const current = document.documentElement.getAttribute('data-theme');
	document.documentElement.setAttribute('data-theme', current === 'light' ? 'dark' : 'light');
	ensureMermaid().then(() => renderCurrentView());
});

document.getElementById('openFolder').addEventListener('click', async () => {
	if (window.showDirectoryPicker) {
		try {
			const dir = await window.showDirectoryPicker();
			await loadFromDirectoryHandle(dir);
			renderCurrentView();
			return;
		} catch (err) {
			console.warn('Directory picker canceled or failed', err);
		}
	}
	document.getElementById('folderInput').click();
});
document.getElementById('reloadBtn').addEventListener('click', () => {
	init();
});

document.getElementById('folderInput').addEventListener('change', async (ev) => {
	const files = Array.from(ev.target.files || []);
	if (files.length === 0) return;
	await loadFilesFromFileInput(files);
	renderCurrentView();
});

viewSelect.addEventListener('change', () => renderCurrentView());

// drag & drop support: drop Aurora folder or JSON files onto the chart
const chartEl = document.getElementById('chart');
chartEl.addEventListener('dragover', (e) => {
	e.preventDefault();
	chartEl.style.outline = '2px dashed var(--aurora-blue)';
});
chartEl.addEventListener('dragleave', (e) => {
	chartEl.style.outline = '';
});
chartEl.addEventListener('drop', async (e) => {
	e.preventDefault();
	chartEl.style.outline = '';
	const files = Array.from(e.dataTransfer.files || []);
	if (files.length === 0) return;
	await loadFilesFromFileInput(files);
	renderCurrentView();
});

// File System Access helpers (for local file:// usage)
async function collectFilesFromHandle(handle, path = '') {
	const results = [];
	try {
		if (handle.kind === 'file') {
			const file = await handle.getFile();
			results.push({ path: path + '/' + handle.name, name: handle.name, file });
			return results;
		}
		if (handle.entries) {
			for await (const [name, child] of handle.entries()) {
				if (child.kind === 'file') {
					const file = await child.getFile();
					results.push({ path: path + '/' + name, name, file });
				} else if (child.kind === 'directory') {
					const nested = await collectFilesFromHandle(child, path + '/' + name);
					results.push(...nested);
				}
			}
		} else if (handle.values) {
			for await (const child of handle.values()) {
				const name = child.name || '';
				if (child.kind === 'file') {
					const file = await child.getFile();
					results.push({ path: path + '/' + name, name, file });
				} else if (child.kind === 'directory') {
					const nested = await collectFilesFromHandle(child, path + '/' + name);
					results.push(...nested);
				}
			}
		} else {
			console.warn('Directory handle is not iterable in this browser');
		}
	} catch (e) {
		console.warn('collectFilesFromHandle error', e);
	}
	return results;
}

async function loadFromDirectoryHandle(dirHandle) {
	const files = await collectFilesFromHandle(dirHandle, dirHandle.name || '');
	let jsonFiles = files.filter((f) => f.name && f.name.toLowerCase().endsWith('.json'));
	// prefer files under an Aurora subfolder if present
	const auroraFiles = jsonFiles.filter((f) => (f.path || '').toLowerCase().includes('/aurora/'));
	if (auroraFiles.length) jsonFiles = auroraFiles;
	for (const f of jsonFiles) {
		try {
			const text = await f.file.text();
			const obj = JSON.parse(text);
			if (obj && obj.uuid) cards[obj.uuid] = obj;
		} catch (e) {
			console.warn('skip', f.path, e);
		}
	}
	console.log('Loaded', Object.keys(cards).length, 'cards from directory');
}

async function tryPickDirectory() {
	try {
		const dir = await window.showDirectoryPicker();
		await loadFromDirectoryHandle(dir);
		renderCurrentView();
	} catch (e) {
		console.warn('Directory picker cancelled or failed', e);
	}
}

// modal
const modal = document.getElementById('cardModal');
document.getElementById('modalClose').addEventListener('click', () => modal.classList.remove('open'));
modal.addEventListener('click', (e) => {
	if (e.target === modal) modal.classList.remove('open');
});

function escapeLabel(s) {
	if (!s) return '';
	return String(s).replace(/"/g, '\\"').replace(/\n/g, '\\\\n');
}

function mkNodeId(uuid) {
	return 'c_' + uuid.replace(/-/g, '_');
}

function buildMermaidForView(viewName) {
	const nodes = [];
	const nodeDefs = {};
	const edges = [];
	const clicks = [];
	const groups = {};
	const groupNames = {};
	// reset included cards for this view
	currentIncludedCards = [];
	// filter by view
	const filters = {
		'Requirements Traceability': [
			'mission',
			'driver',
			'capability',
			'feature',
			'requirement',
			'constraint',
			'test',
		],
		'Capability Map': ['driver', 'capability', 'feature', 'process'],
		'Component Architecture': [
			'system',
			'application',
			'boundary',
			'component',
			'interface',
			'artifact',
			'datastore',
			'node',
			'test',
		],
		'Interface / API Surface': ['interface', 'actor', 'component', 'action'],
		'Data Flow and Storage': ['datastore', 'component', 'interface', 'artifact'],
		'Threat Model and Controls': ['constraint', 'boundary', 'component', 'datastore', 'interface', 'test'],
		'Deployment Topology': ['node', 'boundary', 'application', 'component', 'datastore'],
		'Action / Process Flow': ['actor', 'action', 'condition', 'event', 'state', 'process', 'interface'],
		'State Diagram': ['state', 'condition', 'event'],
		'Communication Diagram': ['actor', 'component', 'interface'],
	};

	const includeAll = viewName === 'All Cards' || viewName === 'By Type';
	const allowed = filters[viewName] || [];

	// build nodes — label with human-readable name only
	for (const uuid in cards) {
		const card = cards[uuid];
		const rawType = card.card_type || 'unknown';
		const type = rawType.toLowerCase();
		if (!includeAll && allowed.length && !allowed.includes(type)) continue;
		// track included uuid for header listing
		currentIncludedCards.push(uuid);
		const nid = mkNodeId(uuid);
		idToMermaid[uuid] = nid;
		const humanName = card.name || '(Unnamed)';
		const typeDisplay = (card.card_type || '').toString();
		const subtypeDisplay = card.card_subtype ? ` (${card.card_subtype})` : '';
		// Build an HTML label: first line = Type (subtype), second line = Name
		const topLine = escapeHtml(typeDisplay + subtypeDisplay);
		const nameLine = escapeHtml(humanName);
		// Use a simple HTML block with a data-nid attribute (single quotes only) so
		// Mermaid's parser doesn't get confused by nested double quotes.
		const anchorHtml = `<div data-nid='${nid}' style='color:inherit; text-decoration:none'><div style='text-align:center'><div style='font-weight:700'>${topLine}</div><div style='margin-top:6px'>${nameLine}</div></div></div>`;
		nodeDefs[nid] = `${nid}["${escapeLabel(anchorHtml)}"]`;
		// keep mermaid click as fallback
		clicks.push(`click ${nid} showCard`);
		// grouping by card_type for all views; remember display name
		groups[type] = groups[type] || [];
		groups[type].push(nid);
		groupNames[type] = rawType;
	}

	// edges — only include edges between nodes included in this view
	const includedSet = new Set(currentIncludedCards);
	for (const uuid in cards) {
		if (!includedSet.has(uuid)) continue;
		const card = cards[uuid];
		const src = mkNodeId(uuid);
		const links = card.links || [];
		for (const l of links) {
			const tgtUuid = l.target;
			if (!cards[tgtUuid] || !includedSet.has(tgtUuid)) continue;
			const tgt = mkNodeId(tgtUuid);
			const rel = escapeLabel(l.relationship || '');
			edges.push(`${src} -->|${rel}| ${tgt}`);
		}
	}

	// build graph text - force top-down orientation
	const dir = 'TB';
	let out = `flowchart ${dir}\n`;
	// ensure we render subgraphs for all relevant types (include empty placeholders when needed)
	const typesSet = new Set(Object.keys(groups));
	if (includeAll) {
		for (const u in cards) {
			const ct = ((cards[u] && cards[u].card_type) || 'unknown').toLowerCase().trim();
			typesSet.add(ct);
			groupNames[ct] = groupNames[ct] || (cards[u] && cards[u].card_type) || ct;
			groups[ct] = groups[ct] || [];
		}
	} else {
		for (const a of allowed) {
			typesSet.add(a);
			groupNames[a] = groupNames[a] || a;
			groups[a] = groups[a] || [];
		}
	}
	const types = Array.from(typesSet.values()).sort();
	for (const t of types) {
		const sid = 'sg_' + String(t).replace(/[^a-zA-Z0-9_]/g, '_');
		const display = groupNames[t] || t;
		out += `  subgraph ${sid}["${escapeLabel(display)}"]\n`;
		if (groups[t] && groups[t].length) {
			for (const n of groups[t]) out += `    ${nodeDefs[n] || n}\n`;
		} else {
			// placeholder node when a subgraph has no members for this view
			const ph = `${sid}_empty`;
			out += `    ${ph}[(no items)]\n`;
		}
		out += `  end\n`;
	}
	out += '\n';
	for (const e of edges) out += '  ' + e + '\n';
	out += '\n';
	for (const c of clicks) out += '  ' + c + '\n';
	return out;
}

function openModal(card) {
	document.getElementById('modalTitle').textContent = card.name || card.id || card.uuid;
	const main = document.getElementById('modalMain');
	const side = document.getElementById('modalSide');
	main.innerHTML = '';
	side.innerHTML = '';
	const desc = document.createElement('div');
	desc.innerHTML = `<div class="muted">Type</div><div style="font-weight:600">${card.card_type || ''}${
		card.card_subtype ? ' · ' + card.card_subtype : ''
	}</div><p class=muted>${card.id ? escapeHtml(card.id) : ''}</p><hr/>`;
	main.appendChild(desc);
	const p = document.createElement('div');
	p.innerHTML = `<h3>Description</h3><div>${escapeHtml(card.description || '')}</div>`;
	main.appendChild(p);
	const links = document.createElement('div');
	links.innerHTML = '<h3>Links</h3>';
	const ul = document.createElement('ul');
	(card.links || []).forEach((l) => {
		const li = document.createElement('li');
		const target = l.target;
		const label = `${l.relationship || ''} → ${target}`;
		const a = document.createElement('a');
		a.href = '#';
		a.textContent = label;
		a.addEventListener('click', (e) => {
			e.preventDefault();
			if (cards[target]) showCardByUUID(target);
		});
		li.appendChild(a);
		ul.appendChild(li);
	});
	links.appendChild(ul);
	main.appendChild(links);
	const attrs = document.createElement('div');
	attrs.innerHTML = '<h3>Attributes</h3>';
	const pre = document.createElement('pre');
	pre.textContent = JSON.stringify(card.attributes || {}, null, 2);
	attrs.appendChild(pre);
	side.appendChild(attrs);
	const meta = document.createElement('div');
	meta.innerHTML =
		'<h3>Meta</h3>' +
		`<div class=muted>uuid: ${card.uuid || ''}</div><div class=muted>status: ${
			card.status || ''
		}</div><div class=muted>version: ${card.version || ''}</div>`;
	side.appendChild(meta);
	modal.classList.add('open');
}

function escapeHtml(s) {
	if (!s) return '';
	return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\n/g, '<br/>');
}

window.showCard = function (nodeId) {
	// called by mermaid click
	// nodeId may be like c_uuid_with_underscores
	const uuid = nodeId.replace(/^c_/, '').replace(/_/g, '-');
	showCardByUUID(uuid);
};

function showCardByUUID(uuid) {
	const card = cards[uuid];
	if (!card) {
		alert('Card not found: ' + uuid);
		return;
	}
	openModal(card);
}

async function renderWithMermaid(def) {
	const container = document.getElementById('chart');
	try {
		if (window.mermaid && mermaid.mermaidAPI && mermaid.mermaidAPI.render) {
			const { svg } = await mermaid.mermaidAPI.render('mermaidSvg', def);
			container.innerHTML = svg;
			postRenderAdjustments(container);
		} else if (window.mermaid && mermaid.render) {
			mermaid.render('mermaidSvg', def, (svgCode) => {
				container.innerHTML = svgCode;
				postRenderAdjustments(container);
			});
		} else {
			container.textContent = 'Mermaid not available — cannot render graph.';
			container.style.whiteSpace = 'pre-wrap';
			container.textContent = def;
		}
	} catch (err) {
		console.error(err);
		container.textContent = 'Render error: ' + err.message;
	}
}

// populate header badges for the current view
function populateViewHeader() {
	const header = document.getElementById('viewHeader');
	header.innerHTML = '';
	const viewName = viewSelect.value || 'All Cards';
	const title = document.createElement('div');
	title.className = 'view-title';
	title.textContent = `${viewName} (${currentIncludedCards.length})`;
	header.appendChild(title);
	const list = document.createElement('div');
	list.className = 'card-list';
	// compute counts per card_type
	const typeCounts = {};
	for (const uuid of currentIncludedCards) {
		const card = cards[uuid];
		if (!card) continue;
		const type = (card.card_type || 'unknown').toLowerCase();
		typeCounts[type] = (typeCounts[type] || 0) + 1;
	}
	const types = Object.keys(typeCounts).sort((a, b) => {
		if (typeCounts[b] !== typeCounts[a]) return typeCounts[b] - typeCounts[a];
		return a.localeCompare(b);
	});
	for (const t of types) {
		const b = document.createElement('div');
		b.className = 'card-badge';
		b.textContent = `${t} (${typeCounts[t]})`;
		b.title = `${typeCounts[t]} cards of type ${t}`;
		b.addEventListener('click', () => {
			viewSelect.value = 'By Type';
			renderCurrentView();
		});
		list.appendChild(b);
	}
	header.appendChild(list);
}

// Simple pan & zoom for the mermaid-generated SVG
const _panZoom = { scale: 1, tx: 0, ty: 0, svg: null, g: null, registered: false };

function updateTransform() {
	if (!_panZoom.g) return;
	_panZoom.g.setAttribute('transform', `translate(${_panZoom.tx},${_panZoom.ty}) scale(${_panZoom.scale})`);
}

function postRenderAdjustments(container) {
	populateViewHeader();
	enablePanZoomOnSvg(container);
	// attach click handlers to HTML labels with data-nid so clicks open the card modal
	try {
		// labels may be rendered as foreignObject children or direct SVG HTML
		const labelEls = Array.from(container.querySelectorAll('[data-nid]'));
		for (const el of labelEls) {
			el.style.cursor = 'pointer';
			if (!el._auroraClickAttached) {
				el.addEventListener('click', (e) => {
					e.preventDefault();
					const nid = el.getAttribute('data-nid');
					if (nid) window.showCard(nid);
				});
				el._auroraClickAttached = true;
			}
		}
	} catch (e) {}

	// As a fallback, attach click handlers to the SVG node groups (they have id attributes like the mermaid node ids)
	try {
		for (const u of currentIncludedCards) {
			const nid = idToMermaid[u];
			if (!nid) continue;
			let nodeEl = container.querySelector('[id="' + nid + '"]');
			if (!nodeEl) nodeEl = container.querySelector('#' + nid);
			if (nodeEl && !nodeEl._auroraGroupClickAttached) {
				nodeEl.style.cursor = 'pointer';
				nodeEl.addEventListener('click', (e) => {
					e.preventDefault();
					window.showCard(nid);
				});
				nodeEl._auroraGroupClickAttached = true;
			}
		}
	} catch (e) {}
}

function enablePanZoomOnSvg(container) {
	try {
		const svg = container.querySelector('svg');
		if (!svg) return;
		// wrap children in a panRoot group if missing
		let panRoot = svg.querySelector('#panRoot');
		if (!panRoot) {
			panRoot = document.createElementNS('http://www.w3.org/2000/svg', 'g');
			panRoot.setAttribute('id', 'panRoot');
			while (svg.firstChild) {
				panRoot.appendChild(svg.firstChild);
			}
			svg.appendChild(panRoot);
		}
		_panZoom.svg = svg;
		_panZoom.g = panRoot;
		// Ensure Mermaid-generated text is visible on the theme
		try {
			const textColor = getComputedStyle(document.documentElement).getPropertyValue('--text') || '#e6f0fb';
			svg.querySelectorAll('text, .label, tspan').forEach((el) => {
				el.style.fill = textColor.trim();
				el.style.stroke = 'none';
				el.style.fillOpacity = '1';
				el.style.opacity = '1';
				el.style.fontSize = '16px';
			});
			// handle HTML labels if any
			svg.querySelectorAll('foreignObject').forEach((fo) => {
				fo.style.color = textColor.trim();
			});
		} catch (e) {
			// ignore styling errors
		}
		// attempt initial fit-to-view so nodes are readable rather than tiny
		_panZoom.scale = 1;
		_panZoom.tx = 0;
		_panZoom.ty = 0;
		try {
			const bbox = panRoot.getBBox();
			const svgRect = svg.getBoundingClientRect();
			const margin = 40;
			let scaleX = (svgRect.width - margin * 2) / Math.max(1, bbox.width);
			let scaleY = (svgRect.height - margin * 2) / Math.max(1, bbox.height);
			if (!isFinite(scaleX) || scaleX <= 0) scaleX = 1;
			if (!isFinite(scaleY) || scaleY <= 0) scaleY = 1;
			// compute a fit scale and clamp to reasonable bounds so diagrams fill the area
			const fitScale = Math.min(scaleX, scaleY);
			const MIN_INIT_SCALE = 2.0; // prefer at least 200% so content is readable
			const MAX_INIT_SCALE = 3.0; // allow modest upscales
			let initScale = fitScale;
			if (!isFinite(initScale) || initScale <= 0) initScale = MIN_INIT_SCALE;
			initScale = Math.max(initScale, MIN_INIT_SCALE);
			initScale = Math.min(initScale, MAX_INIT_SCALE);
			_panZoom.scale = initScale;
			_panZoom.tx = (svgRect.width - bbox.width * initScale) / 2 - bbox.x * initScale;
			_panZoom.ty = (svgRect.height - bbox.height * initScale) / 2 - bbox.y * initScale;
		} catch (err) {
			// ignore bbox problems
		}
		updateTransform();

		svg.style.touchAction = 'none';

		// remove previous listeners
		if (_panZoom.registered && _panZoom.svg) {
			try {
				_panZoom.svg.removeEventListener('wheel', _panZoom._onWheel);
				_panZoom.svg.removeEventListener('pointerdown', _panZoom._onPointerDown);
				_panZoom.svg.removeEventListener('pointermove', _panZoom._onPointerMove);
				_panZoom.svg.removeEventListener('pointerup', _panZoom._onPointerUp);
				_panZoom.svg.removeEventListener('pointerleave', _panZoom._onPointerUp);
			} catch (e) {}
		}

		let isPanning = false;
		let last = { x: 0, y: 0 };

		function onWheel(e) {
			e.preventDefault();
			const rect = svg.getBoundingClientRect();
			const cx = e.clientX - rect.left;
			const cy = e.clientY - rect.top;
			let newScale = _panZoom.scale * (e.deltaY < 0 ? 1.12 : 0.9);
			newScale = Math.max(0.2, Math.min(10, newScale));
			try {
				const pt = svg.createSVGPoint();
				pt.x = cx;
				pt.y = cy;
				const ctm = _panZoom.g.getCTM();
				if (ctm) {
					const inv = ctm.inverse();
					const p = pt.matrixTransform(inv);
					_panZoom.tx = _panZoom.tx - p.x * (newScale - _panZoom.scale);
					_panZoom.ty = _panZoom.ty - p.y * (newScale - _panZoom.scale);
				}
			} catch (err) {
				// ignore
			}
			_panZoom.scale = newScale;
			updateTransform();
		}

		function onPointerDown(e) {
			isPanning = true;
			last.x = e.clientX;
			last.y = e.clientY;
			svg.setPointerCapture && svg.setPointerCapture(e.pointerId);
		}
		function onPointerMove(e) {
			if (!isPanning) return;
			const dx = e.clientX - last.x;
			const dy = e.clientY - last.y;
			last.x = e.clientX;
			last.y = e.clientY;
			_panZoom.tx += dx;
			_panZoom.ty += dy;
			updateTransform();
		}
		function onPointerUp(e) {
			isPanning = false;
			try {
				svg.releasePointerCapture && svg.releasePointerCapture(e.pointerId);
			} catch (e) {}
		}

		_panZoom._onWheel = onWheel;
		_panZoom._onPointerDown = onPointerDown;
		_panZoom._onPointerMove = onPointerMove;
		_panZoom._onPointerUp = onPointerUp;

		svg.addEventListener('wheel', onWheel, { passive: false });
		svg.addEventListener('pointerdown', onPointerDown);
		svg.addEventListener('pointermove', onPointerMove);
		svg.addEventListener('pointerup', onPointerUp);
		svg.addEventListener('pointerleave', onPointerUp);
		_panZoom.registered = true;

		// set up zoom controls
		const zc = document.getElementById('zoomControls');
		zc.innerHTML = '';
		const btnZoomIn = document.createElement('button');
		btnZoomIn.title = 'Zoom in';
		btnZoomIn.textContent = '+';
		const btnZoomOut = document.createElement('button');
		btnZoomOut.title = 'Zoom out';
		btnZoomOut.textContent = '−';
		const btnReset = document.createElement('button');
		btnReset.title = 'Reset';
		btnReset.textContent = '⤢';
		btnZoomIn.addEventListener('click', () => {
			_panZoom.scale = Math.min(10, _panZoom.scale * 1.2);
			updateTransform();
		});
		btnZoomOut.addEventListener('click', () => {
			_panZoom.scale = Math.max(0.2, _panZoom.scale * 0.8);
			updateTransform();
		});
		btnReset.addEventListener('click', () => {
			_panZoom.scale = 1;
			_panZoom.tx = 0;
			_panZoom.ty = 0;
			updateTransform();
		});
		zc.appendChild(btnZoomIn);
		zc.appendChild(btnZoomOut);
		zc.appendChild(btnReset);
	} catch (e) {
		console.warn('pan/zoom init failed', e);
	}
}

async function renderCurrentView() {
	const view = viewSelect.value || viewDefs[0];
	document.getElementById('chart').textContent = 'Rendering ' + view + '...';
	const def = buildMermaidForView(view);
	// debug: log a snippet of the mermaid definition to verify labels
	try {
		console.debug('MERMAID_DEF_SNIPPET:', def.slice(0, 2000));
	} catch (e) {}
	// populate header immediately
	try {
		populateViewHeader();
	} catch (e) {}
	await ensureMermaid();
	await renderWithMermaid(def);
}

async function loadFilesFromFileInput(files) {
	// files is FileList with webkitRelativePath if directory chosen
	const arr = Array.from(files).filter((f) => f.name && f.name.toLowerCase().endsWith('.json'));
	if (arr.length === 0) return;
	// prefer files inside an Aurora subfolder when provided via directory selection
	const aurora = arr.filter((f) => (f.webkitRelativePath || '').toLowerCase().includes('/aurora/'));
	const chosen = aurora.length ? aurora : arr;
	for (const f of chosen) {
		try {
			const text = await f.text();
			const obj = JSON.parse(text);
			if (obj && obj.uuid) {
				cards[obj.uuid] = obj;
			}
		} catch (e) {
			console.warn('skip', f.name, e);
		}
	}
	console.log('Loaded', Object.keys(cards).length, 'cards');
}

async function attemptAutoDiscover() {
	// Only attempt network autodiscovery when served over http(s).
	if (!location.protocol || !location.protocol.startsWith('http')) {
		console.log('Autodiscovery skipped: not an http(s) origin ->', location.protocol);
		return false;
	}
	// Try simple index or directory listing at ./Aurora/
	try {
		const idxUrls = ['./Aurora/index.json', './Aurora/_index.json', './Aurora/manifest.json'];
		for (const u of idxUrls) {
			try {
				const r = await fetch(u);
				if (r.ok) {
					const list = await r.json();
					if (Array.isArray(list.files)) {
						for (const p of list.files) {
							try {
								const rr = await fetch('./Aurora/' + p);
								if (rr.ok) {
									const j = await rr.json();
									if (j && j.uuid) cards[j.uuid] = j;
								}
							} catch (e) {}
						}
					}
					renderCurrentView();
					return true;
				}
			} catch (e) {}
		}
		// try directory HTML listing
		const res = await fetch('./Aurora/');
		if (res.ok) {
			const ct = res.headers.get('content-type') || '';
			if (ct.includes('text/html')) {
				const txt = await res.text();
				const doc = new DOMParser().parseFromString(txt, 'text/html');
				const anchors = Array.from(doc.querySelectorAll('a'));
				const jsons = anchors.map((a) => a.getAttribute('href')).filter((h) => h && h.endsWith('.json'));
				for (const href of jsons) {
					try {
						const rr = await fetch('./Aurora/' + href);
						if (rr.ok) {
							const j = await rr.json();
							if (j && j.uuid) cards[j.uuid] = j;
						}
					} catch (e) {}
				}
				if (Object.keys(cards).length) {
					renderCurrentView();
					return true;
				}
			}
		}
	} catch (e) {
		console.warn('autodiscover failed', e);
	}
	return false;
}

async function init() {
	// reset cards
	// attempt to auto-discover; if not available prompt user
	Object.keys(cards).forEach((k) => delete cards[k]);
	viewSelect.value = 'All Cards';
	// If running from file://, use local file access patterns (no network fetches allowed)
	if (!location.protocol || !location.protocol.startsWith('http')) {
		// If the File System Access API is available, offer a one-click/user-gesture prompt
		if (window.showDirectoryPicker) {
			document.getElementById('chart').innerHTML = `
					<div style="padding:20px">
						<strong>Local mode detected</strong>
						<p class="note">Click anywhere on the page (or use the button) to grant access to a folder containing the Aurora model. The viewer will then load JSON files automatically.</p>
						<button id="pickDirBtn" class="btn-primary">Select Aurora Folder</button>
						<button id="fallbackFiles" class="btn-ghost">Or select JSON files</button>
					</div>
				`;
			document.getElementById('pickDirBtn').addEventListener('click', async () => {
				await tryPickDirectory();
			});
			document
				.getElementById('fallbackFiles')
				.addEventListener('click', () => document.getElementById('folderInput').click());
			// Use first user gesture to open the directory picker automatically (one-time)
			const oneTime = async () => {
				document.removeEventListener('pointerdown', oneTime);
				await tryPickDirectory();
			};
			document.addEventListener('pointerdown', oneTime, { once: true });
			return;
		}
		// Older browsers without directory picker: instruct user to pick files manually
		document.getElementById('chart').innerHTML = `
				<div style="padding:20px">
					<strong>No automatic discovery (file:// detected).</strong>
					<p class="note">Browser security prevents fetching local files from pages opened via <code>file://</code>. Click <strong>Open Aurora Folder</strong> to select the folder or drag & drop the Aurora folder (or JSON files) onto this area.</p>
					<button id="openFiles" class="btn-primary">Choose JSON files</button>
				</div>
			`;
		document
			.getElementById('openFiles')
			.addEventListener('click', () => document.getElementById('folderInput').click());
		return;
	}
	const ok = await attemptAutoDiscover();
	if (!ok) {
		document.getElementById('chart').textContent =
			'No Aurora index discovered. Click "Open Aurora Folder" and select the folder containing the Aurora model.';
	}
}

// initial
(async () => {
	document.documentElement.setAttribute('data-theme', 'dark');
	await ensureMermaid();
	await init();
})();
