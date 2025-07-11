import { PetraWallet } from "petra-plugin-wallet-adapter";
import { CedraWalletAdapterProvider } from "@cedra-labs/wallet-adapter-react";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import reportWebVitals from "./reportWebVitals";

const wallets = [new PetraWallet()];

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);
root.render(
  <React.StrictMode>
    <CedraWalletAdapterProvider plugins={wallets} autoConnect={true}>
      <App />
    </CedraWalletAdapterProvider>
  </React.StrictMode>,
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://create-react-app.dev/docs/measuring-performance/
reportWebVitals();
