import { classColors } from "$lib/constants/colors";
import { invoke } from "@tauri-apps/api";
import { emit } from "@tauri-apps/api/event";
import { register, unregisterAll } from "@tauri-apps/api/globalShortcut";
import { writable } from "svelte/store";

export const defaultSettings = {
    general: {
        showNames: true,
        showGearScore: false,
        showEsther: true,
        positionalDmgPercent: false,
        hideLogo: false,
        accentColor: "theme-pink",
        rawSocket: false,
        autoIface: true,
        ifDesc: "",
        ip: "",
        port: 6040,
        blur: true,
        transparent: true
    },
    shortcuts: {
        hideMeter: {
            modifier: "Ctrl",
            key: "ArrowDown"
        },
        showLogs: {
            modifier: "Ctrl",
            key: "ArrowUp"
        },
        showLatestEncounter: {
            modifier: "Ctrl",
            key: "ArrowRight"
        },
        resetSession: {
            modifier: "",
            key: ""
        },
        pauseSession: {
            modifier: "",
            key: ""
        }
    },
    meter: {
        bossHp: true,
        bossHpBar: true,
        splitBossHpBar: false,
        abbreviateHeader: true,
        showClassColors: true,
        damage: false,
        dps: true,
        damagePercent: true,
        deathTime: false,
        critRate: true,
        frontAtk: true,
        backAtk: true,
        counters: false,
        positionalDmgPercent: false,
        percentBuffBySup: false,
        percentBrand: false,
        breakdown: {
            damage: true,
            dps: true,
            damagePercent: true,
            critRate: true,
            frontAtk: true,
            backAtk: true,
            avgDamage: false,
            maxDamage: false,
            casts: true,
            cpm: true,
            hits: false,
            hpm: false,
            positionalDmgPercent: false,
            percentBuffBySup: false,
            percentBrand: false
        }
    },
    logs: {
        abbreviateHeader: false,
        damage: true,
        dps: true,
        damagePercent: true,
        deathTime: true,
        critRate: true,
        frontAtk: true,
        backAtk: true,
        counters: false,
        minEncounterDuration: 30,
        positionalDmgPercent: false,
        percentBuffBySup: false,
        percentBrand: false,
        breakdown: {
            damage: true,
            dps: true,
            damagePercent: true,
            critRate: true,
            frontAtk: true,
            backAtk: true,
            avgDamage: false,
            maxDamage: false,
            casts: true,
            cpm: true,
            hits: false,
            hpm: false,
            positionalDmgPercent: false,
            percentBuffBySup: false,
            percentBrand: false
        }
    },
    buffs: {
        default: true,
    }
};

const settingsStore = (key: string, defaultSettings: object) => {
    const storedSettings = localStorage.getItem(key);
    const value = storedSettings ? JSON.parse(storedSettings) : defaultSettings;
    const store = writable(value);
    if (typeof window !== "undefined") {
        window.addEventListener("storage", (event) => {
            if (event.key === key) {
                const newValue = JSON.parse(event.newValue || "");
                store.set(newValue);
            }
        });
    }
    return {
        subscribe: store.subscribe,
        set: (value: object) => {
            localStorage.setItem(key, JSON.stringify(value));
            if (key === "settings") {
                invoke("save_settings", { settings: value });
            }
            store.set(value);
        },
        update: store.update
    };
};

export const settings = settingsStore("settings", defaultSettings);
export const colors = settingsStore("classColors", classColors);

export async function registerShortcuts(shortcuts: any) {
    await unregisterAll();

    if (shortcuts.hideMeter.modifier && shortcuts.hideMeter.key) {
        await register(shortcuts.hideMeter.modifier + "+" + shortcuts.hideMeter.key, async () => {
            await invoke("toggle_meter_window");
        });
    }
    if (shortcuts.showLogs.modifier && shortcuts.showLogs.key) {
        await register(shortcuts.showLogs.modifier + "+" + shortcuts.showLogs.key, async () => {
            await invoke("open_url", { url: "logs" });
        });
    }
    if (shortcuts.showLatestEncounter.modifier && shortcuts.showLatestEncounter.key) {
        await register(shortcuts.showLatestEncounter.modifier + "+" + shortcuts.showLatestEncounter.key, async () => {
            await invoke("open_most_recent_encounter");
        });
    }
    if (shortcuts.resetSession.modifier && shortcuts.resetSession.key) {
        await register(shortcuts.resetSession.modifier + "+" + shortcuts.resetSession.key, async () => {
            await emit("reset-request");
        });
    }
    if (shortcuts.pauseSession.modifier && shortcuts.pauseSession.key) {
        await register(shortcuts.pauseSession.modifier + "+" + shortcuts.pauseSession.key, async () => {
            await emit("pause-request");
        });
    }
}

export const skillIcon = settingsStore("skillIcon", {});
export const classIconCache = settingsStore("classIconCache", {});

export const keyboardKeys = [
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "F1",
    "F2",
    "F3",
    "F4",
    "F5",
    "F6",
    "F7",
    "F8",
    "F9",
    "F10",
    "F11",
    "F12",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight"
];
