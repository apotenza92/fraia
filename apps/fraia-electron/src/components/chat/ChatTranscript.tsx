import type { CSSProperties, ReactNode } from 'react';
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

const nativeScrollbarStyle: CSSProperties = {
  scrollbarColor: 'auto',
  scrollbarGutter: 'auto',
  scrollbarWidth: 'auto',
  WebkitMaskImage: 'none',
  maskImage: 'none',
};

export function ChatTranscript({
  children,
  busy = false,
}: {
  children: ReactNode;
  busy?: boolean;
}) {
  return (
    <MessageScrollerProvider
      autoScroll
      defaultScrollPosition="last-anchor"
      scrollPreviousItemPeek={40}
    >
      <MessageScroller>
        <MessageScrollerViewport style={nativeScrollbarStyle}>
          <MessageScrollerContent aria-busy={busy} className="gap-3 p-3">
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
  messageId,
}: {
  author: 'assistant' | 'user';
  children: ReactNode;
  messageId: string;
}) {
  const userMessage = author === 'user';
  return (
    <MessageScrollerItem messageId={messageId} scrollAnchor={userMessage}>
      <Message
        align={userMessage ? 'end' : 'start'}
        aria-label={userMessage ? 'You' : 'Fraia AI'}
        data-author={author}
      >
        <MessageContent>
          <Bubble align={userMessage ? 'end' : 'start'} variant={userMessage ? 'default' : 'ghost'}>
            <BubbleContent>{children}</BubbleContent>
          </Bubble>
        </MessageContent>
      </Message>
    </MessageScrollerItem>
  );
}

export function ChatTranscriptActivity({
  children,
  label,
}: {
  children: ReactNode;
  label: string;
}) {
  return (
    <MessageScrollerItem>
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
