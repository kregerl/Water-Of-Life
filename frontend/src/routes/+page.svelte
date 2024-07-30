<script lang="ts">
    import SelectableLabelList from "$lib/components/SelectableLabelList.svelte";
    import SuggestiveTextBox from "$lib/components/SuggestiveTextBox.svelte";
    import Slider from "$lib/components/Slider.svelte";

    let value: string;

    // Can be ml or Oz
    let unit = "ml";
    // https://www.craftbeering.com/how-many-ml-in-a-shot-glass/
    const servingSizesMilliliters = [20, 25, 30, 35.5, 40, 44, 45, 50, 60];
    // https://www.therail.media/stories/2017/7/20/the-daily-rail-how-big-is-a-shot-glass-in-each-country
    // Excluded Canada since its so close to the US
    const servingSizesOunces = [
        0.67, 0.84, 1.01, 1.2, 1.35, 1.48, 1.52, 1.69, 2.02,
    ];
</script>

<!-- https://developers.google.com/identity/openid-connect/openid-connect#php -->
<!-- <SuggestiveTextBox id="test" bind:value label="Text"></SuggestiveTextBox> -->

<Slider />
<SelectableLabelList
    title="Serving Sizes"
    options={servingSizesMilliliters}
    suffix={unit}
/>

<SelectableLabelList 
title="Notes"
options={["New", "Sherry"]}/>

<style>
    
</style>
