import type { Settings } from "./SettingsApp";

interface Props {
  settings: Settings;
  onUpdate: (updates: Partial<Settings>) => void;
}

const POSITION_OPTIONS = [
  { value: 0, label: "Top Left" },
  { value: 1, label: "Top Middle" },
  { value: 2, label: "Top Right" },
  { value: 3, label: "Bottom Left" },
  { value: 4, label: "Bottom Middle" },
  { value: 5, label: "Bottom Right" },
];

export function IndicatorSettings({ settings, onUpdate }: Props) {
  return (
    <div className="settings-section">
      <h2>Indicator</h2>

      <div className="indicator-controls">
        <div className="slider-group">
          <label>
            Alpha: {Math.round(settings.indicator_opacity * 100)}%
          </label>
          <input
            type="range"
            min="0"
            max="100"
            value={settings.indicator_opacity * 100}
            onChange={(e) =>
              onUpdate({ indicator_opacity: Number(e.target.value) / 100 })
            }
          />
          <div className="slider-labels">
            <span>0%</span>
            <span>100%</span>
          </div>
        </div>

        <div className="position-group">
          <label>Location</label>
          <div className="position-options">
            {POSITION_OPTIONS.map((opt) => (
              <label key={opt.value} className="radio-label">
                <input
                  type="radio"
                  name="position"
                  value={opt.value}
                  checked={settings.indicator_position === opt.value}
                  onChange={() => onUpdate({ indicator_position: opt.value })}
                />
                {opt.label}
              </label>
            ))}
          </div>
        </div>

        <div className="slider-group">
          <label>
            Size: {settings.indicator_size.toFixed(1)}x
          </label>
          <input
            type="range"
            min="50"
            max="200"
            value={settings.indicator_size * 100}
            onChange={(e) =>
              onUpdate({ indicator_size: Number(e.target.value) / 100 })
            }
          />
          <div className="slider-labels">
            <span>small</span>
            <span>big</span>
          </div>
        </div>
      </div>
    </div>
  );
}
