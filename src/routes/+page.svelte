<script lang="ts">
    import DamageMeter from "$lib/components/DamageMeter.svelte";
    import {
        classIconCache,
        colors,
        defaultSettings,
        imagePath,
        registerShortcuts,
        settings,
        skillIcon,
        update,
        updateSettings
    } from "$lib/utils/settings";
    import { appWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import merge from "lodash-es/merge";
    import { join, resourceDir } from "@tauri-apps/api/path";
    import { convertFileSrc } from "@tauri-apps/api/tauri";
    import { classesMap } from "$lib/constants/classes";
    import { estherMap } from "$lib/constants/esthers";
    import { invoke } from "@tauri-apps/api";
    import { classColors } from "$lib/constants/colors";
    import { queryParam } from "$lib/utils/strings";
    import { emit } from "@tauri-apps/api/event";
    import { checkUpdate } from "@tauri-apps/api/updater";

    onMount(() => {
        (async () => {
            await invoke("write_log", { message: "setting up live meter" });
            let data = await invoke("get_settings");
            if (data) {
                settings.set(merge(defaultSettings, $settings, data));
            } else {
                settings.set(merge(defaultSettings, $settings));
            }
            colors.set(merge(classColors, $colors));
            updateSettings.set(update);
            if ($settings.general.alwaysOnTop) {
                await appWindow.setAlwaysOnTop(true);
            } else {
                await appWindow.setAlwaysOnTop(false);
            }
            if ($settings.general.bossOnlyDamageDefaultOn && !$settings.general.bossOnlyDamage) {
                $settings.general.bossOnlyDamage = true;
                await emit("boss-only-damage-request", true);
            }
            imagePath.set({
                path: convertFileSrc(await join(await resourceDir(), "images"))
            });
            skillIcon.set({
                path: convertFileSrc(await join(await resourceDir(), "images", "skills"))
            });
            for (const key of Object.keys(classesMap)) {
                $classIconCache[key] =
                    convertFileSrc(await join(await resourceDir(), "images", "classes", key + ".png")) + queryParam;
            }
            for (const esther of estherMap) {
                $classIconCache[esther.name] =
                    convertFileSrc(await join(await resourceDir(), "images", "classes", esther.icon)) + queryParam;
            }

            try {
                const { shouldUpdate, manifest } = await checkUpdate();
                if (shouldUpdate) {
                    $updateSettings.available = true;
                    const oldManifest = $updateSettings.manifest;
                    $updateSettings.manifest = manifest;
                    if (oldManifest?.version !== $updateSettings.manifest?.version) {
                        $updateSettings.dismissed = false;
                    }
                    if (manifest?.version.includes("2024")) {
                        $updateSettings.isNotice = true;
                    } else {
                        $updateSettings.isNotice = false;
                    }
                }
            } catch (e) {
                await invoke("write_log", { message: String(e) });
            }

            await registerShortcuts($settings.shortcuts);

            // disable blur on windows 11
            let ua = await navigator.userAgentData.getHighEntropyValues(["platformVersion"]);
            if (navigator.userAgentData.platform === "Windows") {
                const majorPlatformVersion = Number(ua.platformVersion.split(".")[0]);
                if (majorPlatformVersion >= 13) {
                    $settings.general.isWin11 = true;
                    if ($settings.general.blurWin11) {
                        await invoke("enable_blur");
                    } else {
                        await invoke("disable_blur");
                    }
                } else if ($settings.general.blur) {
                    await invoke("enable_blur");
                } else {
                    await invoke("disable_blur");
                }
            }

            await invoke("write_log", { message: "finished meter setup" });
        })();
    });

    $: {
        if ($settings.general.scale === "1") {
            document.documentElement.style.setProperty("font-size", "medium");
        } else if ($settings.general.scale === "2") {
            document.documentElement.style.setProperty("font-size", "large");
        } else if ($settings.general.scale === "3") {
            document.documentElement.style.setProperty("font-size", "x-large");
        } else if ($settings.general.scale === "0") {
            document.documentElement.style.setProperty("font-size", "small");
        }
    }
</script>

<div
    class="h-screen overflow-hidden {$settings.general.transparent ? 'bg-zinc-800/[.2]' : 'bg-zinc-800 opacity-95'}"
    id="live-meter">
    <DamageMeter />
</div>
