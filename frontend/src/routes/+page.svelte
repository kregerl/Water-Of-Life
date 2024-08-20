<script lang="ts">
    import SelectableLabelList from "$lib/components/SelectableLabelList.svelte";
    import SuggestiveTextBox from "$lib/components/SuggestiveTextBox.svelte";
    import Slider from "$lib/components/Slider.svelte";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import NewPostModal from "$lib/modal/NewPostModal.svelte";

    let isModalOpen: boolean = false;
    let showAdmin: boolean = false;

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

    function closeModal() {
        isModalOpen = false;
    }

    type User = {
        username: string;
        role: string;
        scopes: string[];
    };

    onMount(async () => {
        const response = await fetch("/api/user_info");
        if (response.status != 200) {
            goto("/login");
        } else {
            const json = (await response.json()) as User;
            console.log("user_info:", json);
            if (json.role === "admin") {
                showAdmin = true;
            }
        }
    });
</script>

<!-- https://developers.google.com/identity/openid-connect/openid-connect#php -->
<!-- <SuggestiveTextBox id="test" bind:value label="Text"></SuggestiveTextBox> -->

<!-- <Slider />
<SelectableLabelList
    title="Serving Sizes"
    options={servingSizesMilliliters}
    suffix={unit}
/>

<SelectableLabelList title="Notes" options={["New", "Sherry"]} /> -->

<div class="menu">
    <h1>Water Of Life</h1>
    <ul>
        <li><h3 class="selected">Activity</h3></li>
        <li><h3 on:click={() => goto("/profile")}>Profile</h3></li>
        <li><h3>Search</h3></li>
    </ul>
</div>

<button class="post" on:click={() => (isModalOpen = true)}>New Post</button>
{#if isModalOpen}
    <NewPostModal on:close={closeModal} />
{/if}

{#if showAdmin}
    <button on:click={() => goto("/admin")}>Admin!</button>
{/if}

<style>
    :global(body) {
        margin: 0;
    }

    .post {
        border: none;
        background: blue;
        color: white;
        position: absolute;
        bottom: 5%;
        right: 5%;
    }

    .menu {
        display: flex;
        flex-direction: row;
        box-shadow: 5px 5px 5px 5px lightgray;
    }

    @media screen and (min-width: 1250px) {
        .menu {
            padding-left: 30%;
        }
    }

    ul {
        list-style-type: none;
        margin: 0;
        padding: 0;
        overflow: hidden;
        color: white;
    }

    li {
        float: left;
        padding: 14px 16px 0px 16px;
    }

    li h3 {
        display: block;
        color: black;
        text-align: center;
        text-decoration: none;
    }

    .selected {
        text-decoration-color: blue;
        text-decoration-line: underline;
        text-decoration-thickness: 3px;
    }
</style>
