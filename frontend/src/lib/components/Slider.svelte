<script lang="ts">
    import RangeSlider from "svelte-range-slider-pips";
    import ArrowLabel from "./ArrowLabel.svelte";

    export let min: number = 0;
    export let max: number = 5;
    export let step: number = 0.25;

    export let values: number[] = [max / 2];
</script>

<div class="wrapper">
    <div class="slider-wrapper">
        <RangeSlider
            float
            hoverable={false}
            pips
            pipstep={2}
            step={step}
            min={min}
            max={max}
            all={false}
            first="label"
            last="label"
            rest="pip"
            range="min"
            springValues={{ stiffness: 1, damping: 1 }}
            bind:values={values}
        />
    </div>
    <ArrowLabel bind:value={values[0]} backgroundColor="var(--dark-purple)"/>
</div>


<style>
    * {
        --dark-purple: hsl(244.1, 63.2%, 54.1%);
        --pip-color: black;
    }

    .wrapper {
        /* margin-top: 100px; */
        display: flex;
        flex-direction: row;
    }

    .slider-wrapper {
        --range-handle-inactive: var(--dark-purple);
        --range-handle-focus: var(--dark-purple);
        --range-range-inactive: var(--dark-purple);
        --range-range: var(--dark-purple);
        --range-float: var(--dark-purple);

        --range-pip:               var(--pip-color);
        --range-pip-text:          var(--pip-color);
        --range-pip-active:        var(--pip-color);
        --range-pip-active-text:   var(--pip-color);
        --range-pip-hover:         var(--pip-color);
        --range-pip-hover-text:    var(--pip-color);
        --range-pip-in-range:      var(--pip-color);
        --range-pip-in-range-text: var(--pip-color);

        width: 300px;
    }
</style>
