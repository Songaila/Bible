<script lang="ts">
    import { installUpdate } from "@tauri-apps/api/updater";
    import { writable } from "svelte/store";
    import { updateSettings } from "$lib/utils/settings";
    import { markdownIt } from "$lib/utils/stores.js";

    let updateText = writable("Update Now");
</script>

{#if $updateSettings.available && $updateSettings.manifest && !$updateSettings.dismissed}
    <div class="fixed inset-0 z-50 bg-zinc-900 bg-opacity-80" />
    <div class="fixed left-0 right-0 top-0 z-50 h-modal w-full items-center justify-center p-4">
        <div class="relative top-[10%] mx-auto flex max-h-[95%] w-full max-w-md md:max-w-lg lg:max-w-2xl xl:max-w-3xl">
            <div class="relative mx-auto flex flex-col rounded-lg border-gray-700 bg-zinc-800 text-gray-400 shadow-md">
                <button
                    type="button"
                    class="absolute right-2.5 top-3 ml-auto whitespace-normal rounded-lg p-1.5 hover:bg-zinc-600 focus:outline-none"
                    aria-label="Close modal"
                    on:click={() => ($updateSettings.dismissed = true)}>
                    <span class="sr-only">Close modal</span>
                    <svg class="size-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
                        <path
                            fill-rule="evenodd"
                            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                            clip-rule="evenodd" />
                    </svg>
                </button>
                <div id="modal" class="flex-1 space-y-6 overflow-y-auto overscroll-contain px-6 py-4">
                    <div class="">
                        <div class="mb-1 flex items-center justify-center space-x-1">
                            {#if $updateSettings.isNotice}
                                <div class="text-lg font-semibold text-gray-200 py-2">Notice</div>
                            {:else}
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    viewBox="0 -960 960 960"
                                    class="h-12 w-12 fill-gray-200">
                                    <path
                                        d="M281.5-165v-57.5H679v57.5H281.5Zm170-165v-356L329-563.5 289-604l191-191 191.5 191-40.5 40.5L509-686v356h-57.5Z" />
                                </svg>
                                <div class="text-lg font-semibold text-gray-200">New Update Available!</div>
                            {/if}
                        </div>
                        <div class="prose-a:text-accent-500 prose prose-sm prose-zinc prose-invert mb-5" id="notes">
                            {@html $markdownIt.render($updateSettings.manifest.body)}
                        </div>
                        {#if !$updateSettings.isNotice}
                            <div class="flex justify-center">
                                <button
                                    type="button"
                                    class="bg-accent-900 hover:bg-accent-800 mr-2 inline-flex items-center justify-center rounded-lg px-5 py-2.5 text-center text-sm text-white focus:outline-none"
                                    on:click={async () => {
                                        $updateText = "Updating...";
                                        await installUpdate();
                                    }}>
                                    {$updateText}
                                </button>
                            </div>
                        {/if}
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}
