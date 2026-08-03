import { Fragment, useId } from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/components/ui/breadcrumb';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { cn } from '@/lib/utils';
import {
  WORKFLOW_STAGE_LABELS,
  WORKFLOW_STAGES,
  type WorkflowStage,
  type WorkflowStageState,
} from '@/lib/workflowJourney';

export type WorkflowStageNavigationState = Pick<
  WorkflowStageState,
  'stage' | 'available' | 'gateReason'
>;

export type WorkflowStageBarProps = {
  currentStage: WorkflowStage;
  stages: readonly WorkflowStageNavigationState[];
  onNavigate: (stage: WorkflowStage) => void;
  className?: string;
};

type EdgeButtonProps = {
  direction: 'previous' | 'next';
  destination: WorkflowStage;
  destinationState: WorkflowStageNavigationState | undefined;
  descriptionId: string;
  onNavigate: (stage: WorkflowStage) => void;
};

function EdgeButton({
  direction,
  destination,
  destinationState,
  descriptionId,
  onNavigate,
}: EdgeButtonProps) {
  const isPrevious = direction === 'previous';
  const label = isPrevious ? 'Previous' : 'Next';
  const isGated = !destinationState?.available;
  const gateReason = isGated
    ? destinationState?.gateReason
      ?? `${WORKFLOW_STAGE_LABELS[destination]} is not available yet.`
    : null;

  const button = (
    <Button
      type="button"
      variant="outline"
      size="sm"
      aria-disabled={isGated || undefined}
      aria-describedby={gateReason ? descriptionId : undefined}
      onClick={() => {
        if (isGated) return;
        onNavigate(destination);
      }}
    >
      {isPrevious ? <ChevronLeft data-icon="inline-start" /> : null}
      {label}
      {!isPrevious ? <ChevronRight data-icon="inline-end" /> : null}
    </Button>
  );

  if (!gateReason) return button;

  return (
    <>
      <Tooltip>
        <TooltipTrigger render={button} />
        <TooltipContent>{gateReason}</TooltipContent>
      </Tooltip>
      <span id={descriptionId} className="sr-only">
        {gateReason}
      </span>
    </>
  );
}

export function WorkflowStageBar({
  currentStage,
  stages,
  onNavigate,
  className,
}: WorkflowStageBarProps) {
  const previousGateDescriptionId = useId();
  const nextGateDescriptionId = useId();
  const currentIndex = WORKFLOW_STAGES.indexOf(currentStage);
  const previousStage = currentIndex > 0 ? WORKFLOW_STAGES[currentIndex - 1] : null;
  const nextStage = currentIndex < WORKFLOW_STAGES.length - 1
    ? WORKFLOW_STAGES[currentIndex + 1]
    : null;
  const stageStateById = new Map(stages.map((stage) => [stage.stage, stage] as const));

  return (
    <Breadcrumb
      aria-label="Design workflow"
      className={cn('grid shrink-0 grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-2 px-2 pt-1.5', className)}
    >
      <div className="pb-1.5">
        {previousStage ? (
          <EdgeButton
            direction="previous"
            destination={previousStage}
            destinationState={stageStateById.get(previousStage)}
            descriptionId={previousGateDescriptionId}
            onNavigate={onNavigate}
          />
        ) : null}
      </div>

      <div
        data-slot="workflow-stage-scroll"
        className="min-w-0 overflow-x-auto pb-1.5"
      >
        <BreadcrumbList className="mx-auto w-max min-w-full flex-nowrap justify-center whitespace-nowrap">
          {WORKFLOW_STAGES.map((stage, index) => {
            const label = WORKFLOW_STAGE_LABELS[stage];
            const stageState = stageStateById.get(stage);
            const isCurrent = stage === currentStage;
            const isEarlier = index < currentIndex;
            const isAvailableEarlier = isEarlier && stageState?.available;

            return (
              <Fragment key={stage}>
                {index > 0 ? <BreadcrumbSeparator /> : null}
                <BreadcrumbItem>
                  {isCurrent ? (
                    <BreadcrumbPage aria-current="step">{label}</BreadcrumbPage>
                  ) : isAvailableEarlier ? (
                    <BreadcrumbLink
                      render={<Button type="button" variant="link" size="sm" />}
                      onClick={() => onNavigate(stage)}
                    >
                      {label}
                    </BreadcrumbLink>
                  ) : (
                    <span aria-disabled="true" className="text-muted-foreground/70">
                      {label}
                    </span>
                  )}
                </BreadcrumbItem>
              </Fragment>
            );
          })}
        </BreadcrumbList>
      </div>

      <div className="justify-self-end pb-1.5">
        {nextStage ? (
          <EdgeButton
            direction="next"
            destination={nextStage}
            destinationState={stageStateById.get(nextStage)}
            descriptionId={nextGateDescriptionId}
            onNavigate={onNavigate}
          />
        ) : null}
      </div>

      <span className="sr-only" aria-live="polite" aria-atomic="true">
        Step {currentIndex + 1} of {WORKFLOW_STAGES.length}: {WORKFLOW_STAGE_LABELS[currentStage]}
      </span>
      <Separator className="col-span-full" />
    </Breadcrumb>
  );
}
