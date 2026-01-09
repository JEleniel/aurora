// Sidebar toggle behavior for docs viewer
(function () {
	function qs(sel) {
		return document.querySelector(sel);
	}
	document.addEventListener('DOMContentLoaded', () => {
		const btn = qs('#mobile-toggle');
		const sidebar = qs('#sidebar');
		if (!btn || !sidebar) return;

		btn.addEventListener('click', (e) => {
			e.stopPropagation();
			document.body.classList.toggle('sidebar-open');
		});

		// close when clicking outside on mobile
		document.addEventListener('click', (e) => {
			if (!document.body.classList.contains('sidebar-open')) return;
			if (!e.target.closest('#sidebar') && !e.target.closest('#mobile-toggle')) {
				document.body.classList.remove('sidebar-open');
			}
		});

		// close on escape
		document.addEventListener('keydown', (e) => {
			if (e.key === 'Escape') document.body.classList.remove('sidebar-open');
		});

		// collapse sidebar after clicking a link (mobile)
		const links = Array.from(document.querySelectorAll('#sidebar .nav a'));
		links.forEach((a) => a.addEventListener('click', () => document.body.classList.remove('sidebar-open')));

		// Smooth scroll for internal anchors
		const anchors = Array.from(document.querySelectorAll('a[href^="#"]'));
		anchors.forEach((a) =>
			a.addEventListener('click', (e) => {
				const href = a.getAttribute('href');
				if (!href || href === '#') return;
				const id = href.slice(1);
				const target = document.getElementById(id);
				if (target) {
					e.preventDefault();
					target.scrollIntoView({ behavior: 'smooth', block: 'start' });
					history.replaceState(null, '', '#' + id);
				}
			}),
		);
	});
})();
