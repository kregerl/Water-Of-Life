<script lang="ts">
    export let text: string;
    export let checkedColor: string = "var(--dark-purple)";
    export let uncheckedColor: string = "var(--light-purple)";

    let checked = false;
    $: color = checked ? checkedColor : uncheckedColor;
</script>


<div class="wrapper" style="--color: {color};">
    <label>
        <input type="checkbox" bind:checked>
        {text}
    </label>
</div>

<style>
    * {
        --dark-purple: hsl(244.1, 63.2%, 54.1%);
        --light-purple: hsl(244, 43%, 34%);
    }

    .wrapper {
        display: flex;
        flex-direction: column;
        background: var(--color);
        min-width: 5rem;
        padding: 4px 8px 4px 8px;
        border-radius: 12px;
        margin: 0px 2px 0px 2px;
    }

    label {
        color: white;
        font: 1rem monospace;
        text-align: center;
        user-select: none;
    }

    input {
        display: none;
    }
</style>