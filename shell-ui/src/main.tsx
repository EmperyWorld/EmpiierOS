import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import SecureApp from "./secure/SecureApp";
import "./index.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <SecureApp />
  </StrictMode>,
);
