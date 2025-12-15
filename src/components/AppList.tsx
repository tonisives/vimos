interface Props {
  items: string[];
  onAdd: () => void;
  onRemove: (item: string) => void;
}

export function AppList({ items, onAdd, onRemove }: Props) {
  return (
    <div className="app-list">
      <ul className="app-list-items">
        {items.map((item) => (
          <li key={item} className="app-list-item">
            <span className="app-bundle-id">{item}</span>
            <button
              className="remove-button"
              onClick={() => onRemove(item)}
              title="Remove"
            >
              {"\u2715"}
            </button>
          </li>
        ))}
        {items.length === 0 && (
          <li className="app-list-empty">No apps configured</li>
        )}
      </ul>
      <button className="add-button" onClick={onAdd}>
        + Add Application
      </button>
    </div>
  );
}
