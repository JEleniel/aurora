import './styles/app.css';
import App from './App.svelte';

const target = document.getElementById('app');

if (!target) {
	throw new Error('Aurora Editor root element was not found.');
}

const app = new App({
	target,
});

export default app;
