import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import {
  ChatTranscript,
  ChatTranscriptActivity,
  ChatTranscriptCancel,
  ChatTranscriptMessage,
} from '@/components/chat/ChatTranscript';

describe('ChatTranscript', () => {
  it('composes the official chat primitives and preserves their registry appearance', () => {
    const onCancel = vi.fn();
    const { container } = render(
      <div className="h-96">
        <ChatTranscript busy defaultScrollPosition="start">
          <ChatTranscriptMessage author="assistant" messageId="assistant-1">
            Check the support conditions before analysis.
          </ChatTranscriptMessage>
          <ChatTranscriptMessage author="user" messageId="user-1">
            Keep the pinned bases.
          </ChatTranscriptMessage>
          <ChatTranscriptMessage
            author="assistant"
            messageId="assistant-static"
            scrollAnchor={false}
            details={<div data-testid="assistant-details">Choose when to continue.</div>}
          >
            Choose when to continue.
          </ChatTranscriptMessage>
          <ChatTranscriptActivity label="Fraia AI is thinking" messageId="assistant-activity">
            <ChatTranscriptCancel onClick={onCancel} />
          </ChatTranscriptActivity>
        </ChatTranscript>
      </div>,
    );

    const viewport = screen.getByRole('region', { name: 'Messages' });
    const transcript = screen.getByRole('log');
    expect(transcript).toHaveAttribute('aria-busy', 'true');
    expect(container.querySelector('[data-slot="message-scroller"]')).toHaveAttribute('data-default-scroll-position', 'start');
    expect(viewport).not.toHaveAttribute('style');
    expect(viewport).toHaveClass('scroll-fade-b', 'scrollbar-thin', 'scrollbar-gutter-stable');

    const assistant = container.querySelector('[data-slot="message"][data-author="assistant"]');
    const user = container.querySelector('[data-slot="message"][data-author="user"]');
    expect(assistant?.closest('[data-message-id="assistant-1"]')).toHaveAttribute('data-scroll-anchor', 'true');
    expect(user?.closest('[data-message-id="user-1"]')).toHaveAttribute('data-scroll-anchor', 'false');
    expect(container.querySelector('[data-message-id="assistant-static"]')).toHaveAttribute('data-scroll-anchor', 'false');
    expect(screen.getByTestId('assistant-details').closest('[data-slot="message-content"]')).not.toBeNull();
    expect(screen.getByTestId('assistant-details').closest('[data-slot="bubble-content"]')).toBeNull();
    expect(assistant).toHaveAttribute('aria-label', 'Fraia AI');
    expect(assistant?.querySelector('[data-slot="bubble"]')).toHaveAttribute('data-variant', 'ghost');
    expect(user).toHaveAttribute('aria-label', 'You');
    expect(user?.querySelector('[data-slot="bubble"]')).toHaveAttribute('data-variant', 'default');
    expect(screen.getByText('Check the support conditions before analysis.')).toBeVisible();
    expect(screen.getByText('Keep the pinned bases.')).toBeVisible();
    expect(screen.getByRole('status')).toHaveTextContent('Fraia AI is thinking');
    expect(screen.getByRole('status').closest('[data-message-id="assistant-activity"]')).toHaveAttribute('data-scroll-anchor', 'false');
    expect(screen.getByRole('button', { name: 'Cancel response' })).toBeVisible();
    expect(container.querySelector('[data-slot="scroll-area-scrollbar"]')).toBeNull();
  });
});
