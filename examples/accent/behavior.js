const button = context.root.querySelector("button");
const original = document.documentElement.style.zoom;
const toggle = () => {
  document.documentElement.style.zoom = document.documentElement.style.zoom === "0.9" ? original : "0.9";
};
button.addEventListener("click", toggle);
context.onCleanup(() => {
  button.removeEventListener("click", toggle);
  document.documentElement.style.zoom = original;
});
