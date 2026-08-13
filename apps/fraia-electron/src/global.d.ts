import type {
  ConversationCreateRequest,
  ConversationAnalysisRequest,
  ConversationComparisonRequest,
  ConversationComparisonResponse,
  ConversationEvidenceResponse,
  ConversationForkRequest,
  ConversationMessageRequest,
  ConversationFactsUpdateRequest,
  ConversationProposalActionRequest,
  ConversationProposalRequest,
  ConversationTransportState,
  ConversationWorkingCopyCommitRequest,
  ConversationWorkingCopyOperationRequest,
  ConversationWorkingCopyOpenRequest,
} from '@/lib/conversationWorkspace';

export {};
declare global {
  interface Window {
    fraia: Record<string, (...args: any[]) => Promise<any>> & {
      applicationMetadata?: () => Promise<{
        channel: 'stable' | 'beta';
        productName: string;
        userDataDirectoryName: string;
      }>;
      pickProjectFile: () => Promise<string | null>;
      createUntitledProject: () => Promise<string>;
      conversationCreate?: (request: ConversationCreateRequest) => Promise<ConversationTransportState>;
      conversationConverse?: (request: ConversationMessageRequest) => Promise<ConversationTransportState>;
      conversationFacts?: (request: ConversationFactsUpdateRequest) => Promise<ConversationTransportState>;
      conversationAnalyse?: (request: ConversationAnalysisRequest) => Promise<ConversationEvidenceResponse>;
      conversationCompare?: (request: ConversationComparisonRequest) => Promise<ConversationComparisonResponse>;
      conversationPropose?: (request: ConversationProposalRequest) => Promise<unknown>;
      conversationAccept?: (request: ConversationProposalActionRequest) => Promise<{
        revisionId: string;
        snapshotId: string;
        parentRevisionId: string | null;
        author: 'agent' | 'manual' | 'system' | 'user';
        agentProvenance?: { provider: string; model: string; turnId: string } | null;
      }>;
      conversationReject?: (request: ConversationProposalActionRequest) => Promise<unknown>;
      conversationFork?: (request: ConversationForkRequest) => Promise<ConversationTransportState>;
      conversationWorkingCopyOpen?: (request: ConversationWorkingCopyOpenRequest) => Promise<{
        workingCopyId: string;
        sourceRevisionId: string;
        sourceSnapshotId: string;
      }>;
      conversationWorkingCopyApply?: (request: ConversationWorkingCopyOperationRequest) => Promise<unknown>;
      conversationWorkingCopyCommit?: (request: ConversationWorkingCopyCommitRequest) => Promise<{
        revisionId: string;
        snapshotId: string;
        parentRevisionId: string | null;
        author: 'agent' | 'manual' | 'system' | 'user';
        agentProvenance?: { provider: string; model: string; turnId: string } | null;
      }>;
      updateStatus?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      checkForUpdates?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      setUpdateFrequency?: (frequency: import('@/lib/updateStatus').UpdateFrequency) => Promise<import('@/lib/updateStatus').UpdateStatus>;
      installUpdate?: () => Promise<import('@/lib/updateStatus').UpdateStatus>;
      onUpdateStatus?: (listener: (status: import('@/lib/updateStatus').UpdateStatus) => void) => () => void;
      onOpenUpdateDialog?: (listener: () => void) => () => void;
      defaultProjectDir: () => Promise<string>;
      setThemeSource?: (themeSource: 'light' | 'dark' | 'system') => Promise<{ ok: boolean; themeSource?: string }>;
      reloadWindow?: () => Promise<{ ok: boolean }>;
      forceReloadWindow?: () => Promise<{ ok: boolean }>;
      quitApp?: () => Promise<{ ok: boolean }>;
    };
  }
}
