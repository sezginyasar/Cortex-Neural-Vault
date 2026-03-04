import { createApp } from 'vue';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import Aura from '@primevue/themes/aura'; 
import App from './App.vue';
import './assets/main.css';

const app = createApp(App);

app.use(createPinia());
app.use(PrimeVue, {
    theme: {
        preset: Aura,
        options: {
            darkModeSelector: '.my-app-dark', // App.vue'da sarmalayıcıya bu class'ı ekleyebilirsin
        }
    }
});

app.mount('#app');