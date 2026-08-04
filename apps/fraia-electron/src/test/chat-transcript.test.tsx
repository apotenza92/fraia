import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import {
  ChatTranscript,
  ChatTranscriptActivity,
  ChatTranscriptCancel,
  ChatTranscriptMessage,
} from '@/components/chat/ChatTranscript';

describe('ChatTranscript', () => {
  it('composes the official chat primitives with native scrolling and complete content', () => {
    const onCancel = vi.fn();
    const { container } = render(
      <div className="h-96">
        <ChatTranscript busy>
          <ChatTranscriptMessage author="assistant" messageId="assistant-1">
            Check the support conditions before analysis.
          </ChatTranscriptMessage>
          <ChatTranscriptMessage author="user" messageId="user-1">
            Keep the pinned bases.
          </ChatTranscriptMessage>
          <ChatTranscriptActivity label="Fraia AI is thinking">
            <ChatTranscriptCancel onClick={onCancel} />
          </ChatTranscriptActivity>
        </ChatTranscript>
      </div>,
    );

    const viewport = screen.getByRole('region', { name: 'Messages' });
    const transcript = screen.getByRole('log');
    expect(transcript).toHaveAttribute('aria-busy', 'true');
    expect(viewport.getAttribute('style')).toBe(
      'scrollbar-color: auto; scrollbar-gutter: auto; scrollbar-width: auto; -webkit-mask-image: none; mask-image: none;',
    );

    const assistant = container.querySelector('[data-slot="message"][data-author="assistant"]');
    const user = container.querySelector('[data-slot="message"][data-author="user"]');
    expect(assistant).toHaveAttribute('aria-label', 'Fraia AI');
    expect(assistant?.querySelector('[data-slot="bubble"]')).toHaveAttribute('data-variant', 'ghost');
    expect(user).toHaveAttribute('aria-label', 'You');
    expect(user?.querySelector('[data-slot="bubble"]')).toHaveAttribute('data-variant', 'default');
    expect(screen.getByText('Check the support conditions before analysis.')).toBeVisible();
    expect(screen.getByText('Keep the pinned bases.')).toBeVisible();
    expect(screen.getByRole('status')).toHaveTextContent('Fraia AI is thinking');
    expect(screen.getByRole('button', { name: 'Cancel response' })).toBeVisible();
    expect(container.querySelector('[data-slot="scroll-area-scrollbar"]')).toBeNull();
  });
});
