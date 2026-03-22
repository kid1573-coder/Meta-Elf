import { createRouter, createWebHashHistory } from "vue-router";
import MainPanel from "../views/MainPanel.vue";
import SettingsView from "../views/SettingsView.vue";
import WatchlistAddView from "../views/WatchlistAddView.vue";
import WatchlistManageView from "../views/WatchlistManageView.vue";
import StockDetailView from "../views/StockDetailView.vue";
import BrandElfPreviewView from "../views/BrandElfPreviewView.vue";
import MarketMovesView from "../views/MarketMovesView.vue";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: "main", component: MainPanel },
    { path: "/elf-preview", name: "elf-preview", component: BrandElfPreviewView },
    { path: "/settings", name: "settings", component: SettingsView },
    { path: "/watchlist", name: "watchlist", component: WatchlistManageView },
    { path: "/watchlist/add", name: "watchlist-add", component: WatchlistAddView },
    { path: "/market-moves", name: "market-moves", component: MarketMovesView },
    { path: "/stock/:code", name: "stock", component: StockDetailView },
  ],
});

export default router;
