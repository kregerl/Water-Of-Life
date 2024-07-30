<script lang="ts">
    import SelectableLabel from "./SelectableLabel.svelte";

    export let title: string;
    export let options: string[] | number[];
    export let suffix: string = "";

    $: optionStrings = options.map((option) => `${option}${suffix}`);
</script>

<span>{title}</span>
<div class="label-list">
    {#each optionStrings as option}
        <SelectableLabel text={option}/>
    {/each}
</div>

<style>
    .label-list {
        display: flex;
        flex-direction: row;
        width: 100%;
        overflow-x: scroll;
        padding: 4px;
    }
</style>