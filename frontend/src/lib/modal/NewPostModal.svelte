<script lang="ts">
    import SelectableLabelList from "$lib/components/SelectableLabelList.svelte";
    import Slider from "$lib/components/Slider.svelte";
    import Modal from "./Modal.svelte";
    let modal: Modal;

    // Can be ml or Oz
    let unit = "ml";
    // https://www.craftbeering.com/how-many-ml-in-a-shot-glass/
    const servingSizesMilliliters = [20, 25, 30, 35.5, 40, 44, 45, 50, 60];
    // https://www.therail.media/stories/2017/7/20/the-daily-rail-how-big-is-a-shot-glass-in-each-country
    // Excluded Canada since its so close to the US
    const servingSizesOunces = [
        0.67, 0.84, 1.01, 1.2, 1.35, 1.48, 1.52, 1.69, 2.02,
    ]

    function close() {
        modal.close();
    }
</script>

<div class="background">
    <span id="title">Title</span>
    <div class="close" on:click={close}>
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="32"
            height="32"
            viewBox="0 0 20 20"
            ><path
                fill="#393a37"
                d="m15.8333 5.34166-1.175-1.175-4.6583 4.65834-4.65833-4.65834-1.175 1.175 4.65833 4.65834-4.65833 4.6583 1.175 1.175 4.65833-4.6583 4.6583 4.6583 1.175-1.175-4.6583-4.6583z"
            /></svg
        >
    </div>

    <div class="section">
        <span class="section-title">Description</span>
        <textarea cols="40" rows="5"></textarea>
    </div>

    <div class="section">
        <span class="section-title">Rating</span>
        <Slider />
    </div>

    <SelectableLabelList
        title="Serving Sizes"
        options={servingSizesMilliliters}
        suffix={unit}
    />
    <br>
    <SelectableLabelList title="Smelling Notes" options={["Fruit", "Sherry"]} /> 
    <br>
    <SelectableLabelList title="Tasting Notes" options={["Cinnamon", "Biscuit"]} /> 
</div>
<Modal bind:this={modal} on:close />

<style>
    .background {
        position: absolute;
        background-color: white;
        z-index: 6;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
    }

    @media screen and (max-width: 1000px) {
        .background {
            width: 100%;
            height: 100%;
            overflow-x: hidden;
        }
    }

    .close {
        width: 32px;
        height: 32px;
        cursor: pointer;
        margin-left: auto;
    }
    .close path {
        fill: black;
    }

    .section {
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    span,
    textarea {
        margin: 4px;
    }

    .section-title {
    }

    #title {
        position: absolute;
        left: 0;
        color: black;
            overflow-x: hidden;
    }
</style>
