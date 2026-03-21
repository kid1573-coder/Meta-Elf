import { createRouter, createWebHashHistory } from "vue-router";
import MainPanel from "../views/MainPanel.vue";
import SettingsView from "../views/SettingsView.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "main", component: MainPanel },
    { path: "/settings", name: "settings", component: SettingsView },
  ],
});

export default router;
