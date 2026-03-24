import { createApp } from "vue";
import { createPinia } from "pinia";
import { createRouter, createWebHashHistory } from "vue-router";
import "./style.css";
import App from "./App.vue";

const pinia = createPinia();
const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", redirect: "/chat" },
    { path: "/chat", component: () => import("./views/ChatView.vue") },
    { path: "/history", component: () => import("./views/HistoryView.vue") },
    { path: "/trainees", component: () => import("./views/MyTraineesView.vue") },
    { path: "/cards", component: () => import("./views/MyCardsView.vue") },
    { path: "/team-trials", component: () => import("./views/TeamTrialsView.vue") },
    { path: "/races", component: () => import("./views/RaceTableView.vue") },
    { path: "/models", component: () => import("./views/ModelsView.vue") },
    { path: "/settings", component: () => import("./views/SettingsView.vue") },
  ],
});

const app = createApp(App);
app.use(pinia);
app.use(router);
app.mount("#app");
