import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

// Fade the static preloader out once React has painted its first frame.
requestAnimationFrame(() => {
  const el = document.getElementById("volt-preloader");
  if (!el) return;
  el.classList.add("vp-hide");
  window.setTimeout(() => el.remove(), 220);
});