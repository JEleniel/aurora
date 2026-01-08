document.addEventListener('DOMContentLoaded', function () {
	var btn = document.getElementById('mobile-toggle');
	var sidebar = document.getElementById('sidebar');
	if (!btn || !sidebar) return;
	btn.addEventListener('click', function () {
		sidebar.classList.toggle('open');
	});
	// Close sidebar when a nav link is clicked (mobile)
	var links = sidebar.querySelectorAll('a');
	links.forEach(function (a) {
		a.addEventListener('click', function () {
			if (window.innerWidth <= 900) {
				sidebar.classList.remove('open');
			}
		});
	});
});
