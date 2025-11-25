import { useState } from "react";
import { fetch } from "@tauri-apps/plugin-http";
import "./App.css";

interface Message {
  role: "user" | "assistant" | "system";
  content: string;
}

function App() {
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(false);

  async function sendMessage(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim()) return;

    const userMsg: Message = { role: "user", content: input };
    const newMessages = [...messages, userMsg];
    setMessages(newMessages);
    setInput("");
    setLoading(true);

    try {
      // fetching with tauri fetch
      const response = await fetch("http://localhost:3030/chat", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          messages: newMessages,
        }),
      });

      if (response.ok) {
        const data = await response.json();
        // data.message matches the ChatResponse structure from the daemon
        setMessages([...newMessages, data.message]);
      } else {
        console.error("Failed to send message", response.status);
        const errorMsg: Message = { role: "system", content: `Error: ${response.statusText}` };
        setMessages([...newMessages, errorMsg]);
      }
    } catch (err) {
      console.error("Error:", err);
      const errorMsg: Message = { role: "system", content: `Connection error: ${err}` };
      setMessages([...newMessages, errorMsg]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="container">
      <h1>chat</h1>

      <div style={{ marginBottom: "20px", border: "1px solid #ccc", padding: "10px", height: "300px", overflowY: "auto", textAlign: "left" }}>
        {messages.map((msg, i) => (
          <div key={i}>
            <strong>{msg.role}:</strong> {msg.content}
          </div>
        ))}
        {loading && <div>test</div>}
      </div>

      <form className="row" onSubmit={sendMessage}>
        <input
          id="greet-input"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type something"
          disabled={loading}
        />
        <button type="submit" disabled={loading}>
          Send
        </button>
      </form>
    </main>
  );
}

export default App;
