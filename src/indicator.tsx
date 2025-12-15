import { useEffect, useState } from "react";
import ReactDOM from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, LogicalSize, LogicalPosition, availableMonitors } from "@tauri-apps/api/window";

type VimMode = "insert" | "normal" | "visual";

interface Settings {
  indicator_position: number;
  indicator_opacity: number;
  indicator_size: number;
  top_widget: string;
  bottom_widget: string;
}

const BASE_SIZE = 40;

async function applyWindowSettings(settings: Settings) {
  console.log("applyWindowSettings called with:", settings);
  const window = getCurrentWindow();
  const size = Math.round(BASE_SIZE * settings.indicator_size);
  console.log("Calculated size:", size);

  // Get primary monitor dimensions
  const monitors = await availableMonitors();
  console.log("Available monitors:", monitors);
  const monitor = monitors[0];
  if (!monitor) {
    console.error("No monitor found!");
    return;
  }

  const screenWidth = monitor.size.width / monitor.scaleFactor;
  const screenHeight = monitor.size.height / monitor.scaleFactor;
  const padding = 20;

  // Calculate position based on indicator_position (0-5 for 2x3 grid)
  // 0: Top Left, 1: Top Middle, 2: Top Right
  // 3: Bottom Left, 4: Bottom Middle, 5: Bottom Right
  let x: number;
  let y: number;

  const col = settings.indicator_position % 3;
  const row = Math.floor(settings.indicator_position / 3);

  switch (col) {
    case 0: // Left
      x = padding;
      break;
    case 1: // Middle
      x = (screenWidth - size) / 2;
      break;
    case 2: // Right
      x = screenWidth - size - padding;
      break;
    default:
      x = padding;
  }

  switch (row) {
    case 0: // Top
      y = padding + 30; // Account for menu bar
      break;
    case 1: // Bottom
      y = screenHeight - size - padding;
      break;
    default:
      y = padding + 30;
  }

  console.log("Setting size to:", size, "position to:", x, y);
  try {
    await window.setSize(new LogicalSize(size, size));
    await window.setPosition(new LogicalPosition(Math.round(x), Math.round(y)));
    console.log("Window settings applied successfully");
  } catch (err) {
    console.error("Failed to apply window settings:", err);
  }
}

function Indicator() {
  const [mode, setMode] = useState<VimMode>("insert");
  const [settings, setSettings] = useState<Settings | null>(null);

  useEffect(() => {
    console.log("Indicator useEffect running - loading settings");
    // Load initial settings
    invoke<Settings>("get_settings")
      .then((s) => {
        console.log("Got initial settings:", s);
        setSettings(s);
        applyWindowSettings(s);
      })
      .catch((e) => console.error("Failed to get settings:", e));

    // Listen for settings changes
    console.log("Setting up settings-changed listener");
    const unlistenSettings = listen<Settings>("settings-changed", (event) => {
      console.log("settings-changed event received:", event.payload);
      setSettings(event.payload);
      applyWindowSettings(event.payload);
    });

    return () => {
      unlistenSettings.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    // Get initial mode
    invoke<string>("get_vim_mode")
      .then((m) => setMode(m as VimMode))
      .catch((e) => console.error("Failed to get initial mode:", e));

    // Listen for mode changes
    const unlisten = listen<string>("mode-change", (event) => {
      setMode(event.payload as VimMode);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const modeChar = mode === "insert" ? "i" : mode === "normal" ? "n" : "v";
  const opacity = settings?.indicator_opacity ?? 0.9;

  const bgColor =
    mode === "insert"
      ? `rgba(76, 175, 80, ${opacity})`
      : mode === "normal"
        ? `rgba(33, 150, 243, ${opacity})`
        : `rgba(255, 152, 0, ${opacity})`;

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
