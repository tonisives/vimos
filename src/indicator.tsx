import { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type VimMode = "insert" | "normal" | "visual";

function Indicator() {
  const [mode, setMode] = useState<VimMode>("insert");

  useEffect(() => {
    console.log("Indicator component mounted");

    // Get initial mode
    invoke<string>("get_vim_mode")
      .then((m) => {
        console.log("Initial mode fetched:", m);
        setMode(m as VimMode);
      })
      .catch((e) => console.error("Failed to get initial mode:", e));

    // Listen for mode changes
    console.log("Setting up mode-change listener...");
    const unlisten = listen<string>("mode-change", (event) => {
      console.log("Mode change event received:", event.payload);
      setMode(event.payload as VimMode);
    });

    unlisten.then(() => console.log("Event listener registered successfully"));

    return () => {
      console.log("Cleaning up listener");
      unlisten.then((fn) => fn());
    };
  }, []);

  const modeChar = mode === "insert" ? "i" : mode === "normal" ? "n" : "v";

  const bgColor =
    mode === "insert"
      ? "rgba(76, 175, 80, 0.9)"
      : mode === "normal"
        ? "rgba(33, 150, 243, 0.9)"
        : "rgba(255, 152, 0, 0.9)";

  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background: bgColor,
        borderRadius: "8px",
        fontFamily: "system-ui, -apple-system, sans-serif",
        fontSize: "24px",
        fontWeight: "bold",
        color: "white",
        textTransform: "uppercase",
      }}
    >
      {modeChar}
    </div>
  );
}

ReactDOM.createRoot(document.getElementById("root")!).render(<Indicator />);
