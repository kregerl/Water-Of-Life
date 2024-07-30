<!-- <script lang="ts">
    export let title: string;
    export let options: string[];

    $: id = title.toLowerCase().replace(" ", "-");
</script>


<input id={title} list="{id}-options">
<datalist id="{id}-options">
    {#each options as option}
    <option value={option}></option>
    {/each}
</datalist>

<style>


</style> -->

<script lang="ts">
    export let id: string;
    export let value: string;
    export let label: string;
    export let disabled: boolean = false;
</script>

<div class="text-box-wrapper">
    <input {disabled} {id} type="text" bind:value required />
    <label for={id}>{label}</label>
</div>

<style>
    .text-box-wrapper {
        width: var(--width, 450px);
        white-space: nowrap;
    }

    .text-box-wrapper input {
        width: var(--width, 450px);
        height: auto;
        padding: 10px 4px;
        font-size: 1.6rem;
        color: black;
        background: transparent;
        outline: none;
        border: 2px solid transparent;
        opacity: 100%;
        border-bottom: 2px solid purple;
        box-sizing: border-box;
    }

    .text-box-wrapper label {
        position: relative;
        margin-left: 4px;
        right: var(--width, 450px);
        top: 0;
        padding: 10px 0;
        font-size: 1.6rem;
        color: black;
        transition: top 200ms cubic-bezier(0, 0, 0.2, 1) 0ms,
            color 200ms cubic-bezier(0, 0, 0.2, 1) 0ms;
        pointer-events: none;
    }

    .text-box-wrapper input:focus {
        border: 2px solid lightblue;
    }

    .text-box-wrapper input:focus ~ label {
        font-size: 1.2rem;
        top: -23px;
        padding: 0 4px 0 4px;
        opacity: 100%;
        color: lightblue;
        background-color: black;
    }

    .text-box-wrapper input:focus {
        border: 2px solid lightblue;
    }

    .text-box-wrapper input:focus ~ label,
    .text-box-wrapper input:valid ~ label {
        font-size: 1.2rem;
        top: -23px;
        padding: 0 4px 0 4px;
        opacity: 100%;
        background-color: gray;
    }

    .text-box-wrapper input:valid:not(:focus) ~ label {
        color: magenta;
    }

    .text-box-wrapper input:valid:not(:focus) {
        border: 2px solid purple;
    }

    input:disabled {
        cursor: not-allowed;
    }
</style>