import { useState, useRef, useEffect } from "react";
import { api, ChatMessage } from "../lib/api";

export function ChatInterface() {
  const [input, setInput] = useState("");
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const chatEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  async function sendMessage(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim()) return;

    const userMsg: ChatMessage = { role: "user", content: input };
    const newMessages = [...messages, userMsg];
    setMessages(newMessages);
    setInput("");
    setLoading(true);

    try {
      const data = await api.chat.send(newMessages);
      setMessages([...newMessages, data.message]);
    } catch (err) {
      const errorMsg: ChatMessage = { role: "system", content: `Error: ${err}` };
      setMessages([...newMessages, errorMsg]);
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="container">
      <h1>Boomai Chat</h1>
      <div className="chat-window">
        {messages.map((msg, i) => (
          <div key={i} className={`message ${msg.role}`}>
            <strong>{msg.role}:</strong> {msg.content}
          </div>
        ))}
        {loading && <div className="loading">Thinking...</div>}
        <div ref={chatEndRef} />
      </div>

      <form className="row" onSubmit={sendMessage}>
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Type a message..."
          disabled={loading}
        />
        <button type="submit" disabled={loading}>Send</button>
      </form>
    </main>
  );
}

