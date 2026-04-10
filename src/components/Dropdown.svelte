<script lang="ts">
  interface Props {
    id?: string;
    value: string;
    options: { value: string; label: string }[];
    onchange: (value: string) => void;
  }

  let { id, value, options, onchange }: Props = $props();
</script>

<select {id} {value} onchange={(e) => onchange((e.target as HTMLSelectElement).value)}>
  {#each options as opt}
    <option value={opt.value}>{opt.label}</option>
  {/each}
</select>

<style>
  select {
    appearance: none;
    width: 100%;
    padding: 10px 40px 10px 16px;
    font-size: 0.9rem;
    font-weight: 500;
    font-family: inherit;
    color: rgba(255, 255, 255, 0.8);
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s ease;
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='8'%3E%3Cpath d='M1 1l5 5 5-5' stroke='%23666' fill='none' stroke-width='1.5'/%3E%3C/svg%3E");
    background-repeat: no-repeat;
    background-position: right 14px center;
  }

  select:hover {
    background-color: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.2);
  }

  select:focus {
    outline: none;
    border-color: rgba(255, 255, 255, 0.35);
    background-color: rgba(255, 255, 255, 0.1);
  }

  select option {
    background: #1a1a1a;
    color: #fff;
  }
</style>
