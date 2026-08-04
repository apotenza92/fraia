import { Fragment, useId } from 'react';
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

function StageLabel({
  label,
  gateReason,
}: {
  label: string;
  gateReason: string | null;
}) {
  const descriptionId = useId();
  const labelElement = (
    <span
      aria-disabled="true"
      aria-describedby={gateReason ? descriptionId : undefined}
      className="text-muted-foreground/70"
    >
      {label}
    </span>
  );

  if (!gateReason) return labelElement;

  return (
    <>
      <Tooltip>
        <TooltipTrigger render={labelElement} />
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
  const currentIndex = WORKFLOW_STAGES.indexOf(currentStage);
  const stageStateById = new Map(stages.map((stage) => [stage.stage, stage] as const));

  return (
    <Breadcrumb
      aria-label="Design workflow"
      className={cn('grid shrink-0 select-none grid-cols-1 pt-1.5', className)}
    >
      <div
        data-slot="workflow-stage-scroll"
        className="min-w-0 overflow-x-auto px-2 pb-1.5"
      >
        <BreadcrumbList className="mx-auto w-max min-w-full flex-nowrap justify-center whitespace-nowrap">
          {WORKFLOW_STAGES.map((stage, index) => {
            const label = WORKFLOW_STAGE_LABELS[stage];
            const stageState = stageStateById.get(stage);
            const isCurrent = stage === currentStage;
            const isAvailable = stageState?.available;

            return (
              <Fragment key={stage}>
                {index > 0 ? <BreadcrumbSeparator /> : null}
                <BreadcrumbItem>
                  {isCurrent ? (
                    <BreadcrumbPage aria-current="step">{label}</BreadcrumbPage>
                  ) : isAvailable ? (
                    <BreadcrumbLink
                      render={<Button type="button" variant="link" size="sm" />}
                      onClick={() => onNavigate(stage)}
                    >
                      {label}
                    </BreadcrumbLink>
                  ) : (
                    <StageLabel
                      label={label}
                      gateReason={stageState?.available ? null : stageState?.gateReason ?? null}
                    />
                  )}
                </BreadcrumbItem>
              </Fragment>
            );
          })}
        </BreadcrumbList>
      </div>

      <span className="sr-only" aria-live="polite" aria-atomic="true">
        Step {currentIndex + 1} of {WORKFLOW_STAGES.length}: {WORKFLOW_STAGE_LABELS[currentStage]}
      </span>
      <Separator />
    </Breadcrumb>
  );
}
