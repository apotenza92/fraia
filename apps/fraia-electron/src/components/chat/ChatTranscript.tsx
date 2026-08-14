import type { ReactNode } from 'react';
import type { MessageScrollerDefaultScrollPosition } from '@shadcn/react/message-scroller';
import { Spinner } from '@/components/ui/spinner';
import { Bubble, BubbleContent } from '@/components/ui/bubble';
import { Button } from '@/components/ui/button';
import { Marker, MarkerContent, MarkerIcon } from '@/components/ui/marker';
import { Message, MessageContent } from '@/components/ui/message';
import {
  MessageScroller,
  MessageScrollerButton,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerProvider,
  MessageScrollerViewport,
} from '@/components/ui/message-scroller';

export function ChatTranscript({
  children,
  busy = false,
  defaultScrollPosition = 'last-anchor',
}: {
  children: ReactNode;
  busy?: boolean;
  defaultScrollPosition?: MessageScrollerDefaultScrollPosition;
}) {
  return (
    <MessageScrollerProvider
      autoScroll
      defaultScrollPosition={defaultScrollPosition}
      scrollPreviousItemPeek={0}
    >
      <MessageScroller data-default-scroll-position={defaultScrollPosition} data-purpose="conversation-scroll-region">
        <MessageScrollerViewport className="max-h-full" data-testid="conversation-transcript-viewport">
          <MessageScrollerContent aria-busy={busy} className="gap-3 p-3 pb-16">
            {children}
          </MessageScrollerContent>
        </MessageScrollerViewport>
        <MessageScrollerButton />
      </MessageScroller>
    </MessageScrollerProvider>
  );
}

export function ChatTranscriptMessage({
  author,
  children,
  details,
  messageId,
  scrollAnchor,
  testId,
}: {
  author: 'assistant' | 'user';
  children: ReactNode;
  details?: ReactNode;
  messageId: string;
  scrollAnchor?: boolean;
  testId?: string;
}) {
  const userMessage = author === 'user';
  return (
    <MessageScrollerItem messageId={messageId} scrollAnchor={scrollAnchor ?? !userMessage}>
      <Message
        align={userMessage ? 'end' : 'start'}
        aria-label={userMessage ? 'You' : 'Fraia AI'}
        data-author={author}
        data-testid={testId}
      >
        <MessageContent>
          <Bubble align={userMessage ? 'end' : 'start'} variant={userMessage ? 'default' : 'ghost'}>
            <BubbleContent>{children}</BubbleContent>
          </Bubble>
          {details ? <div className="p-px">{details}</div> : null}
        </MessageContent>
      </Message>
    </MessageScrollerItem>
  );
}

export function ChatTranscriptPanel({
  children,
  messageId,
  scrollAnchor = false,
}: {
  children: ReactNode;
  messageId: string;
  scrollAnchor?: boolean;
}) {
  return (
    <MessageScrollerItem messageId={messageId} scrollAnchor={scrollAnchor}>
      <Message aria-label="Fraia AI" data-author="assistant">
        <MessageContent className="p-px">{children}</MessageContent>
      </Message>
    </MessageScrollerItem>
  );
}

export function ChatTranscriptActivity({
  children,
  label,
  messageId,
}: {
  children: ReactNode;
  label: string;
  messageId: string;
}) {
  return (
    <MessageScrollerItem messageId={messageId} scrollAnchor={false}>
      <Message aria-label="Fraia AI status">
        <MessageContent>
          <Marker role="status">
            <MarkerIcon><Spinner /></MarkerIcon>
            <MarkerContent>{label}</MarkerContent>
          </Marker>
          <div className="flex flex-col gap-2">
            {children}
          </div>
        </MessageContent>
      </Message>
    </MessageScrollerItem>
  );
}

export function ChatTranscriptCancel({ onClick }: { onClick: () => void }) {
  return (
    <Button onClick={onClick} variant="secondary" size="sm">
      Cancel response
    </Button>
  );
}
