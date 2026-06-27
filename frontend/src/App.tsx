import React from "react";
import { Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Landing from "./pages/Landing";
import Prove from "./pages/Prove";
import Escrow from "./pages/Escrow";
import Audit from "./pages/Audit";

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Landing />} />
        <Route path="/prove" element={<Prove />} />
        <Route path="/escrow" element={<Escrow />} />
        <Route path="/audit" element={<Audit />} />
      </Route>
    </Routes>
  );
}
