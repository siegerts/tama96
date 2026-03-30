import { useState } from "react";
import { usePetState } from "./hooks/usePetState";
import PetDisplay from "./components/PetDisplay";
import ActionBar from "./components/ActionBar";
import DeathScreen from "./components/DeathScreen";
import PermissionsPanel from "./components/PermissionsPanel";

function App() {
  const {
    state,
    loading,
    error,
    feedMeal,
    feedSnack,
    playGame,
    discipline,
    giveMedicine,
    cleanPoop,
    toggleLights,
    hatchNewEgg,
  } = usePetState();
  const [showSettings, setShowSettings] = useState(false);

  if (loading) {
    return (
      <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "100vh" }}>
        <p>Loading your pet…</p>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ display: "flex", justifyContent: "center", alignItems: "center", height: "100vh", color: "#f44336" }}>
        <p>Error: {error}</p>
      </div>
    );
  }

  if (!state) {
    return null;
  }

  return (
    <div style={{ minHeight: "100vh", display: "flex", flexDirection: "column", alignItems: "center", paddingTop: 24 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
        <h1 style={{ fontSize: 20, margin: 0, fontFamily: "monospace" }}>tama96</h1>
        <button
          onClick={() => setShowSettings(true)}
          style={{
            background: "none",
            border: "none",
            cursor: "pointer",
            fontSize: 18,
            padding: "2px 4px",
            color: "#aaa",
          }}
          aria-label="Open agent permissions settings"
          title="Agent Permissions"
        >
          ⚙️
        </button>
      </div>
      {!state.is_alive ? (
        <DeathScreen state={state} onHatchNewEgg={hatchNewEgg} />
      ) : (
        <>
          <PetDisplay state={state} />
          <ActionBar
            state={state}
            feedMeal={feedMeal}
            feedSnack={feedSnack}
            playGame={playGame}
            discipline={discipline}
            giveMedicine={giveMedicine}
            cleanPoop={cleanPoop}
            toggleLights={toggleLights}
          />
        </>
      )}
      {showSettings && <PermissionsPanel onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;
