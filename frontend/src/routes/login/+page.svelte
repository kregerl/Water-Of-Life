<script lang="ts">
    import { goto } from "$app/navigation";

    function login(_event: MouseEvent) {
      goto("/oidc/login")
    }
</script>

<div>
  <h1>Water of Life</h1>
  <button on:click={login}>Login with Keycloak</button>
</div>

<style>
  h1 {
    text-align: center;
  }

  button {
    border: none;
    background: hsl(244.1, 63.2%, 54.1%);
    color: white;
    font-weight: bold;
    font-size: 1rem;
    cursor: pointer;
  }

  div {
    padding: 12px 24px 12px 24px;
    position: absolute;
    display: flex;
    border-radius: 4px;
    justify-content: center;
    flex-direction: column;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    box-shadow: 5px 5px 5px 5px lightgray;
    width: max-content;
  }  
</style>