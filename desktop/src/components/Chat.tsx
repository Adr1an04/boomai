import { useState, useRef, useEffect } from 'react';
import { api, ChatMessage } from '../lib/api';
import { Send, Paperclip, Copy, Check, Code, FileText, Wrench, RefreshCw } from 'lucide-react';

export function Chat() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  const handleSend = async () => {
    if (!input.trim() || loading) return;

    const userMessage: ChatMessage = { role: 'user', content: input };
    const newMessages = [...messages, userMessage];
    setMessages(newMessages);
    setInput('');
    setLoading(true);

    try {
      const data = await api.chat.send(newMessages);
      setMessages([...newMessages, data.message]);
    } catch (err) {
      const errorMessage: ChatMessage = {
        role: 'system',
        content: `Error: ${err}`,
      };
      setMessages([...newMessages, errorMessage]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleCopy = async (content: string, index: number) => {
    await navigator.clipboard.writeText(content);
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 2000);
  };

  return (
    <div className="flex flex-col h-screen">
      <div className="flex-1 overflow-y-auto py-4">
        <div className="max-w-3xl mx-auto px-6">
          {messages.length === 0 ? (
            <div className="flex flex-col items-center justify-center min-h-[60vh] text-center">
              <img src="/boomai.svg" alt="Boomai" className="w-16 h-16 mb-6 opacity-80 invert" />
              <h2 className="font-display text-3xl tracking-wide uppercase mb-2">
                Start a Conversation
              </h2>
              <p className="text-lg text-ink-gray-light max-w-md mb-8">
                Ask me anything. I'm here to help with coding, writing, analysis, and more.
              </p>
              <div className="flex flex-wrap justify-center gap-3">
                <button 
                  className="flex items-center gap-2 px-4 py-3 bg-white border border-paper-dark rounded-lg text-sm hover:border-rocket-red transition-colors"
                  onClick={() => setInput('Help me write a Python script to...')}
                >
                  <Code size={16} className="text-rocket-red" />
                  <span>Help me write code</span>
                </button>
                <button 
                  className="flex items-center gap-2 px-4 py-3 bg-white border border-paper-dark rounded-lg text-sm hover:border-rocket-red transition-colors"
                  onClick={() => setInput('Explain how...')}
                >
                  <FileText size={16} className="text-rocket-red" />
                  <span>Explain a concept</span>
                </button>
                <button 
                  className="flex items-center gap-2 px-4 py-3 bg-white border border-paper-dark rounded-lg text-sm hover:border-rocket-red transition-colors"
                  onClick={() => setInput('Review this code and suggest improvements:')}
                >
                  <Wrench size={16} className="text-rocket-red" />
                  <span>Review my code</span>
                </button>
              </div>
            </div>
          ) : (
            <>
              {messages.map((msg, index) => (
                <div
                  key={index}
                  className={`flex gap-4 py-5 animate-fade-in-up ${
                    index > 0 ? 'border-t border-paper-dark' : ''
                  }`}
                >
                  <div className={`w-8 h-8 rounded-lg flex items-center justify-center font-bold text-sm flex-shrink-0 ${
                    msg.role === 'user' 
                      ? 'bg-deep-space text-paper' 
                      : msg.role === 'assistant'
                      ? 'bg-rocket-red text-white'
                      : 'bg-error text-white'
                  }`}>
                    {msg.role === 'user' ? 'U' : msg.role === 'assistant' ? 'B' : '!'}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-xs font-semibold uppercase tracking-wider text-ink-gray-light mb-2">
                      {msg.role === 'user' ? 'You' : msg.role === 'assistant' ? 'Boomai' : 'System'}
                    </div>
                    <div className="text-base leading-relaxed whitespace-pre-wrap">
                      {msg.content}
                    </div>
                    {msg.role === 'assistant' && (
                      <div className="flex gap-2 mt-3">
                        <button 
                          className="flex items-center gap-1 px-2 py-1 text-xs text-ink-gray-light border border-paper-dark rounded hover:bg-paper-light transition-colors"
                          onClick={() => handleCopy(msg.content, index)}
                        >
                          {copiedIndex === index ? <Check size={12} /> : <Copy size={12} />}
                          {copiedIndex === index ? 'Copied' : 'Copy'}
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              ))}

              {loading && (
                <div className="flex gap-4 py-5 border-t border-paper-dark animate-fade-in-up">
                  <div className="w-8 h-8 rounded-lg bg-rocket-red text-white flex items-center justify-center font-bold text-sm flex-shrink-0">
                    B
                  </div>
                  <div className="flex-1">
                    <div className="text-xs font-semibold uppercase tracking-wider text-ink-gray-light mb-2">
                      Boomai
                    </div>
                    <div className="flex items-center gap-3 text-ink-gray-light">
                      <RefreshCw className="animate-spin" size={16} />
                      <span>Thinking...</span>
                    </div>
                  </div>
                </div>
              )}
            </>
          )}
          <div ref={messagesEndRef} />
        </div>
      </div>

      <div className="border-t border-paper-dark p-4">
        <div className="max-w-3xl mx-auto">
          <div className="flex items-end gap-3 bg-white border-2 border-muted-tan rounded-lg p-3 focus-within:border-rocket-red focus-within:ring-2 focus-within:ring-rocket-red/10 transition-all">
            <button className="p-2 text-ink-gray-light hover:text-ink-gray hover:bg-paper-dark rounded transition-colors">
              <Paperclip size={18} />
            </button>
            <textarea
              ref={textareaRef}
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="Message Boomai..."
              disabled={loading}
              rows={1}
              className="flex-1 resize-none bg-transparent text-base leading-normal outline-none min-h-[24px] max-h-[200px] placeholder:text-muted-tan"
            />
            <button
              onClick={handleSend}
              disabled={!input.trim() || loading}
              className="p-2 bg-rocket-red text-white rounded-lg hover:bg-rocket-red-hover disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <Send size={18} />
            </button>
          </div>
          <p className="text-xs text-muted-tan text-center mt-2">
            Press Enter to send, Shift+Enter for new line
          </p>
        </div>
      </div>
    </div>
  );
}
