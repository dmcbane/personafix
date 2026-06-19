import { useState } from "react";
import { useCharacterStore, type Contact } from "../store/characterStore";

const ARCHETYPES = [
  "Fixer", "Street Doc", "Decker", "Rigger", "Face", "Weapons Dealer",
  "Gang Leader", "Corp Contact", "Law Enforcement", "Smuggler", "Other",
];

export default function ContactPanel() {
  const draft = useCharacterStore((s) => s.draft);
  const addContact = useCharacterStore((s) => s.addContact);
  const removeContact = useCharacterStore((s) => s.removeContact);

  const [name, setName] = useState("");
  const [archetype, setArchetype] = useState(ARCHETYPES[0]);
  const [connection, setConnection] = useState(2);
  const [loyalty, setLoyalty] = useState(2);

  if (!draft) return null;

  const isSR4 = draft.edition === "SR4";
  const costUnit = isSR4 ? "BP" : "karma";

  const totalContactBP = draft.contacts.reduce(
    (sum, c) => sum + c.connection + c.loyalty,
    0,
  );

  const handleAdd = () => {
    if (!name.trim()) return;
    const contact: Contact = {
      id: `${name.toLowerCase().replace(/\s+/g, "_")}_${Date.now()}`,
      name: name.trim(),
      connection,
      loyalty,
      archetype,
      notes: "",
    };
    addContact(contact);
    setName("");
    setConnection(2);
    setLoyalty(2);
  };

  return (
    <div>
      <h2 className="text-xl font-semibold mb-4 text-cyber-heading">
        // Contacts
      </h2>
      <div className="text-sm text-cyber-text-dim font-mono mb-4">
        Contact {costUnit}:{" "}
        <span className="text-cyber-blue">{totalContactBP}</span>
        <span className="text-cyber-text-dim">
          {" "}(1 {costUnit} per connection + loyalty point)
        </span>
      </div>

      {/* Current contacts */}
      {draft.contacts.length > 0 && (
        <div className="space-y-1 mb-4">
          {draft.contacts.map((c) => (
            <div
              key={c.id}
              className="flex items-center gap-3 bg-cyber-card border border-cyber-border rounded px-3 py-2 text-sm"
            >
              <div className="flex-1 min-w-0">
                <span className="text-cyber-text font-medium">{c.name}</span>
                <span className="text-cyber-text-dim font-mono text-xs ml-2">
                  {c.archetype}
                </span>
              </div>
              <div className="flex items-center gap-2 font-mono text-xs shrink-0">
                <span title="Connection" className="text-cyber-blue">
                  CON {c.connection}
                </span>
                <span className="text-cyber-border">·</span>
                <span title="Loyalty" className="text-cyber-green">
                  LOY {c.loyalty}
                </span>
                <span className="text-cyber-text-dim">
                  ({c.connection + c.loyalty} {costUnit})
                </span>
              </div>
              <button
                onClick={() => removeContact(c.id)}
                className="text-cyber-red hover:text-cyber-red/80 transition-colors ml-1"
              >
                X
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Add contact form */}
      <div className="bg-cyber-card border border-cyber-border rounded-lg p-4 space-y-3">
        <h3 className="text-sm font-mono text-cyber-text-dim">Add Contact</h3>
        <div className="grid grid-cols-2 gap-3">
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAdd()}
            placeholder="Contact name"
            className="col-span-2 bg-cyber-surface border border-cyber-border rounded px-3 py-1.5 text-sm"
          />
          <div>
            <label className="text-xs text-cyber-text-dim block mb-1 font-mono">
              Archetype
            </label>
            <select
              value={archetype}
              onChange={(e) => setArchetype(e.target.value)}
              className="w-full bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm"
            >
              {ARCHETYPES.map((a) => (
                <option key={a} value={a}>
                  {a}
                </option>
              ))}
            </select>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="text-xs text-cyber-text-dim block mb-1 font-mono">
                Connection (1–6)
              </label>
              <input
                type="number"
                min={1}
                max={6}
                value={connection}
                onChange={(e) =>
                  setConnection(Math.min(6, Math.max(1, Number(e.target.value))))
                }
                className="w-full bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center"
              />
            </div>
            <div>
              <label className="text-xs text-cyber-text-dim block mb-1 font-mono">
                Loyalty (1–6)
              </label>
              <input
                type="number"
                min={1}
                max={6}
                value={loyalty}
                onChange={(e) =>
                  setLoyalty(Math.min(6, Math.max(1, Number(e.target.value))))
                }
                className="w-full bg-cyber-surface border border-cyber-border rounded px-2 py-1.5 text-sm text-center"
              />
            </div>
          </div>
        </div>
        <div className="flex items-center justify-between">
          <span className="text-xs text-cyber-text-dim font-mono">
            Cost: <span className="text-cyber-blue">{connection + loyalty} {costUnit}</span>
          </span>
          <button
            onClick={handleAdd}
            disabled={!name.trim()}
            className="px-4 py-1.5 bg-cyber-green-dim hover:bg-cyber-green/20 border border-cyber-green-dim hover:border-cyber-green disabled:opacity-40 rounded text-sm font-medium text-cyber-green transition-all"
          >
            Add Contact
          </button>
        </div>
      </div>
    </div>
  );
}
