import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { LinkornDesktop } from "./LinkornDesktop";
import "./linkorn.css";

const root = document.getElementById("root");
if (!root) {
  throw new Error("linkorn_root_missing");
}

createRoot(root).render(
  <StrictMode>
    <LinkornDesktop />
  </StrictMode>,
);
