import { useState } from "react";

export function RuleManager({ 
  title, 
  description, 
  rules, 
  onAddRule, 
  onRemoveRule 
}) {
  const [newRule, setNewRule] = useState("");

  const handleAdd = () => {
    if (newRule.trim()) {
      onAddRule(newRule.trim());
      setNewRule("");
    }
  };

  const getPlaceholder = () => {
    if (title === "Match Rules") {
      return "e.g., com.example.myapp.*";
    }
    return "e.g., android.view.*";
  };

  const getHelpText = () => {
    if (title === "Match Rules") {
      return "避免使用 .* (会追踪所有类导致卡死)";
    }
    return "推荐排除系统类以提高性能";
  };

  return (
    <div className="section">
      <h3>{title}</h3>
      <p className="section-desc">{description}</p>
      <div className="rule-input">
        <input
          type="text"
          value={newRule}
          onChange={(e) => setNewRule(e.target.value)}
          placeholder={getPlaceholder()}
          onKeyPress={(e) => e.key === 'Enter' && handleAdd()}
        />
        <button onClick={handleAdd}>Add</button>
      </div>
      <div className="rule-list">
        {rules.map((rule, index) => (
          <div key={index} className="rule-item">
            <span>{rule.pattern}</span>
            <button onClick={() => onRemoveRule(index)}>Delete</button>
          </div>
        ))}
      </div>
    </div>
  );
}
