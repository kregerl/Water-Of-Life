<script lang="ts">
    import { createEventDispatcher } from "svelte";

    const dispatch = createEventDispatcher();

    export function close() {
        dispatch("close");
    }
</script>

<slot />
<div class="modal-background" />
<svelte:body on:keydown/>

<style>
    .modal-background {
        z-index: 1;
        position: absolute;
        top: 0; 
        left: 0;
        width: 100vw;
        height: 100vh;
        background-color: rgba(0, 0, 0, var(--bg-opacity, 0.65));
    }
</style>
