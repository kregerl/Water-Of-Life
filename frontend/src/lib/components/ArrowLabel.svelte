<script lang="ts">
    export let value: number | string;
    export let backgroundColor: string = "red";
    export let foregroundColor: string = "white";

    $: styleString = `--bg-color: ${backgroundColor}; --fg-color: ${foregroundColor};`;
</script>

<div class="triangle triangle-left" style={styleString} />
<div class="triangle triangle-value" style={styleString}>
    <span class="triangle-text">{value}</span>
</div>

<style>
    .triangle {
        margin-top: 0.5rem;
    }

    .triangle-value {
        background: var(--bg-color);
        text-align: center;
        line-height: 1rem;
        height: 1rem;
        width: 1.5rem;
        padding: 0.25rem 0.5rem 0.25rem 0.5rem;
    }

    .triangle-left {
        width: 0;
        height: 0;
        border-top: 0.75rem solid transparent;
        border-right: 1rem solid var(--bg-color);
        border-bottom: 0.75rem solid transparent;
    }

    .triangle-text {
        margin-left: -2px;
        color: var(--fg-color);
    }
</style>
