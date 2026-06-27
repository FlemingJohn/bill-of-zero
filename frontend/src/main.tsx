import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { Buffer } from 'buffer';
import App from './App.tsx';
import { UiProvider } from './ui';
import './index.css';

// @stellar/stellar-sdk needs a Buffer global in the browser.
(globalThis as any).Buffer = (globalThis as any).Buffer || Buffer;

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <UiProvider>
        <App />
      </UiProvider>
    </BrowserRouter>
  </StrictMode>,
);
