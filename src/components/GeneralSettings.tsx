import { useState, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import type { Settings, VimKeyModifiers } from "./SettingsApp"

interface Props {
  settings: Settings
  onUpdate: (updates: Partial<Settings>) => void
}

interface RecordedKey {
  name: string
  display_name: string
  modifiers: VimKeyModifiers
}

interface PermissionStatus {
  accessibility: boolean
  capture_running: boolean
}

const PRESET_KEYS = [
  { value: "caps_lock", label: "Caps Lock" },
  { value: "escape", label: "Escape" },
  { value: "right_control", label: "Right Control" },
  { value: "right_option", label: "Right Option" },
]

const ICON_STYLE_OPTIONS = [
  { value: true, label: "Menu Bar" },
  { value: false, label: "Hidden" },
]

function formatKeyWithModifiers(displayName: string, modifiers: VimKeyModifiers): string {
  const parts: string[] = []
  if (modifiers.control) parts.push("Ctrl")
  if (modifiers.option) parts.push("Opt")
  if (modifiers.shift) parts.push("Shift")
  if (modifiers.command) parts.push("Cmd")
  parts.push(displayName)
  return parts.join(" + ")
}

function hasAnyModifier(modifiers: VimKeyModifiers): boolean {
  return modifiers.shift || modifiers.control || modifiers.option || modifiers.command
}

export function GeneralSettings({ settings, onUpdate }: Props) {
  const [isRecording, setIsRecording] = useState(false)
  const [displayName, setDisplayName] = useState<string | null>(null)
  const [permissionStatus, setPermissionStatus] = useState<PermissionStatus | null>(null)

  useEffect(() => {
    invoke<string | null>("get_key_display_name", { keyName: settings.vim_key })
      .then((name) => {
        if (name && hasAnyModifier(settings.vim_key_modifiers)) {
          setDisplayName(formatKeyWithModifiers(name, settings.vim_key_modifiers))
        } else {
          setDisplayName(name)
        }
      })
      .catch(() => setDisplayName(null))
  }, [settings.vim_key, settings.vim_key_modifiers])

  useEffect(() => {
    const checkPermissions = () => {
      invoke<PermissionStatus>("get_permission_status")
        .then(setPermissionStatus)
        .catch((e) => console.error("Failed to get permission status:", e))
    }
    checkPermissions()
    const interval = setInterval(checkPermissions, 2000)
    return () => clearInterval(interval)
  }, [])

  const handleRecordKey = async () => {
    setIsRecording(true)
    try {
      const recorded = await invoke<RecordedKey>("record_key")
      onUpdate({
        vim_key: recorded.name,
        vim_key_modifiers: recorded.modifiers,
      })
      const formatted = formatKeyWithModifiers(recorded.display_name, recorded.modifiers)
      setDisplayName(formatted)
    } catch (e) {
      console.error("Failed to record key:", e)
    } finally {
      setIsRecording(false)
    }
  }

  const handleCancelRecord = () => {
    invoke("cancel_record_key").catch(() => {})
    setIsRecording(false)
  }

  const handlePresetSelect = (value: string) => {
    onUpdate({
      vim_key: value,
      vim_key_modifiers: { shift: false, control: false, option: false, command: false },
    })
  }

  const isPresetKey = PRESET_KEYS.some((k) => k.value === settings.vim_key) && !hasAnyModifier(settings.vim_key_modifiers)

  const handleOpenAccessibility = () => {
    invoke("open_accessibility_settings").catch(console.error)
  }

  const handleOpenInputMonitoring = () => {
    invoke("open_input_monitoring_settings").catch(console.error)
  }

  const handleRequestPermission = async () => {
    await invoke("request_permission")
    const status = await invoke<PermissionStatus>("get_permission_status")
    setPermissionStatus(status)
  }

  const permissionsOk = permissionStatus?.accessibility && permissionStatus?.capture_running

  return (
    <div className="settings-section">
      <h2>General Settings</h2>

      {permissionStatus && !permissionsOk && (
        <div className="permission-warning">
          <div className="permission-title">Permissions Required</div>
          <div className="permission-items">
            {!permissionStatus.accessibility && (
              <div className="permission-item">
                <span className="permission-status missing">Accessibility</span>
                <button type="button" className="permission-btn" onClick={handleRequestPermission}>
                  Request
                </button>
                <button type="button" className="permission-btn secondary" onClick={handleOpenAccessibility}>
                  Open Settings
                </button>
              </div>
            )}
            {!permissionStatus.capture_running && (
              <div className="permission-item">
                <span className="permission-status missing">Input Monitoring</span>
                <button type="button" className="permission-btn" onClick={handleOpenInputMonitoring}>
                  Open Settings
                </button>
              </div>
            )}
          </div>
          <div className="permission-hint">
            Grant permissions and restart the app for changes to take effect.
          </div>
        </div>
      )}

      <div className="form-group">
        <label htmlFor="vim-key">Vim mode key</label>
        <div className="key-selector">
          <select
            id="vim-key"
            value={isPresetKey ? settings.vim_key : ""}
            onChange={(e) => handlePresetSelect(e.target.value)}
            disabled={isRecording}
          >
            {PRESET_KEYS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
            {!isPresetKey && (
              <option value="" disabled>
                {displayName || settings.vim_key}
              </option>
            )}
          </select>
          {isRecording ? (
            <button
              type="button"
              className="record-key-btn recording"
              onClick={handleCancelRecord}
            >
              Press any key...
            </button>
          ) : (
            <button
              type="button"
              className="record-key-btn"
              onClick={handleRecordKey}
            >
              Record Key
            </button>
          )}
        </div>
      </div>

      <div className="form-group">
        <label className="checkbox-label">
          <input
            type="checkbox"
            checked={settings.launch_at_login}
            onChange={(e) => onUpdate({ launch_at_login: e.target.checked })}
          />
          Launch ovim at login
        </label>
      </div>

      <div className="form-group">
        <label htmlFor="icon-style">Menu bar icon</label>
        <select
          id="icon-style"
          value={settings.show_in_menu_bar ? "true" : "false"}
          onChange={(e) => onUpdate({ show_in_menu_bar: e.target.value === "true" })}
        >
          {ICON_STYLE_OPTIONS.map((opt) => (
            <option key={String(opt.value)} value={String(opt.value)}>
              {opt.label}
            </option>
          ))}
        </select>
      </div>
    </div>
  )
}
