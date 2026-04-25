import './styles.css';
import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app');

if (!target) {
  throw new Error('앱을 표시할 루트 요소를 찾을 수 없습니다.');
}

const app = mount(App, { target });

export default app;