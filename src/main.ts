import { createApp } from "vue";
import App from "./App.vue";
import {createPinia} from "pinia";
import "./assets/cs16.css";

const pinia = createPinia()
const app = createApp(App);
app.config.performance = true;
app.use(pinia);
app.mount("#app", true);
