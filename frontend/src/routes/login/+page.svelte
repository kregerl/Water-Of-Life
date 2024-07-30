<script>
    import { goto } from "$app/navigation";
</script>
<button on:click={() => {
  goto("/oidc/login")
}}>Login</button>

<button on:click={() => {
  goto("/oidc/logout")
}}>Logout</button>