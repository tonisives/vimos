import { useEffect, useState } from "react"
import { listen } from "@tauri-apps/api/event"
import { invoke } from "@tauri-apps/api/core"
import { openUrl } from "@tauri-apps/plugin-opener"
import { Widget } from "./widgets"
import { applyWindowSettings } from "./windowPosition"
import type { VimMode, Settings, ModeColors } from "./types"

interface PendingUpdate {
  version: string
}

const defaultColors: ModeColors = {
  insert: { r: 74, g: 144, b: 217 },
  normal: { r: 232, g: 148, b: 74 },
  visual: { r: 155, g: 109, b: 215 },
}

export function Indicator() {
  const [mode, setMode] = useState<VimMode>("insert")
  const [settings, setSettings] = useState<Settings | null>(null)
  const [pendingUpdate, setPendingUpdate] = useState<PendingUpdate | null>(null)

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(async (s) => {
        setSettings(s)
        await applyWindowSettings(s)
      })
      .catch((e) => console.error("Failed to get settings:", e))

    const unlistenSettings = listen<Settings>("settings-changed", async (event) => {
      setSettings(event.payload)
      await applyWindowSettings(event.payload)
    })

    return () => {
      unlistenSettings.then((fn) => fn())
    }
  }, [])

  useEffect(() => {
    invoke<string>("get_vim_mode")
      .then((m) => setMode(m as VimMode))
      .catch((e) => console.error("Failed to get initial mode:", e))

    const unlisten = listen<string>("mode-change", (event) => {
      setMode(event.payload as VimMode)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  // Listen for update-installed events
  useEffect(() => {
    const unlisten = listen<PendingUpdate>("update-installed", (event) => {
      setPendingUpdate(event.payload)
      // Make window clickable when update is available
      invoke("set_indicator_clickable", { clickable: true }).catch((e) =>
        console.error("Failed to make indicator clickable:", e)
      )
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  const handleRestartClick = async (e: React.MouseEvent) => {
    e.preventDefault()
    e.stopPropagation()
    // Open GitHub releases page
    const url = `https://github.com/tonisives/ovim/releases/tag/v${pendingUpdate?.version}`
    await openUrl(url).catch((err) => console.error("Failed to open releases:", err))
    // Small delay to ensure browser opens before restart
    setTimeout(() => {
      invoke("restart_app").catch((e) => console.error("Failed to restart:", e))
    }, 500)
  }

  const modeChar = mode === "insert" ? "i" : mode === "normal" ? "n" : "v"
  const opacity = settings?.indicator_opacity ?? 0.9
  const colors = settings?.mode_colors ?? defaultColors
  const color = colors[mode]
  const bgColor = `rgb(${color.r}, ${color.g}, ${color.b})`

  const fontFamily = settings?.indicator_font ?? "system-ui, -apple-system, sans-serif"
  const topWidget = settings?.top_widget ?? "None"
  const bottomWidget = settings?.bottom_widget ?? "None"

  const hasTop = topWidget !== "None"
  const hasBottom = bottomWidget !== "None"
  let gridTemplateRows = "1fr"
  if (hasTop && hasBottom) {
    gridTemplateRows = "auto 1fr auto"
  } else if (hasTop) {
    gridTemplateRows = "auto 1fr"
  } else if (hasBottom) {
    gridTemplateRows = "1fr auto"
  }

  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "grid",
        gridTemplateRows,
        alignItems: "center",
        justifyItems: "center",
        background: bgColor,
        borderRadius: "8px",
        fontFamily,
        color: "white",
        boxSizing: "border-box",
        overflow: "hidden",
        paddingBottom: "1px",
        opacity,
        position: "relative",
      }}
    >
      {/* Update badge overlay - covers entire indicator */}
      {pendingUpdate && (
        <button
          onClick={handleRestartClick}
          title={`Update to v${pendingUpdate.version} ready - click to restart`}
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            borderRadius: "8px",
            background: "#30d158",
            border: "none",
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: "20px",
            fontWeight: "bold",
            color: "white",
            zIndex: 10,
            padding: 0,
          }}
        >
          ↑
        </button>
      )}
      {hasTop && <Widget type={topWidget} fontFamily={fontFamily} />}
      <div
        style={{
          display: "grid",
          placeItems: "center",
          width: "100%",
          height: "100%",
        }}
      >
        <span
          style={{
            fontSize: "36px",
            fontWeight: "bold",
            textTransform: "uppercase",
            lineHeight: "0.75em",
            display: "block",
            transform: "translateY(1px)",
          }}
        >
          {modeChar}
        </span>
      </div>
      {hasBottom && <Widget type={bottomWidget} fontFamily={fontFamily} />}
    </div>
  )
}
