import type { Sprint, Epic, Mission } from '../ipc/types';
import { CreateSprintModal } from './CreateSprintModal';
import { CreateEpicModal } from './CreateEpicModal';
import { CreateIssueModal } from './CreateIssueModal';
import { EditEpicModal } from './EditEpicModal';
import { EditSprintModal } from './EditSprintModal';
import { ConfirmCompleteSprintModal } from './ConfirmCompleteSprintModal';
import { MissionModal } from './MissionModal';

interface IssueManagerModalsProps {
  sprintModalOpen: boolean;
  setSprintModalOpen: (open: boolean) => void;
  missionModalOpen: boolean;
  setMissionModalOpen: (open: boolean) => void;
  epicModalOpen: boolean;
  setEpicModalOpen: (open: boolean) => void;
  editMission: Mission | null;
  setEditMission: (mission: Mission | null) => void;
  issueModalEpicId: number | null;
  setIssueModalEpicId: (id: number | null) => void;
  editEpic: Epic | null;
  setEditEpic: (epic: Epic | null) => void;
  editSprint: Sprint | null;
  setEditSprint: (sprint: Sprint | null) => void;
  completeSprintTarget: Sprint | null;
  setCompleteSprintTarget: (sprint: Sprint | null) => void;
  sprints: Sprint[];
}

export function IssueManagerModals({
  sprintModalOpen,
  setSprintModalOpen,
  missionModalOpen,
  setMissionModalOpen,
  epicModalOpen,
  setEpicModalOpen,
  editMission,
  setEditMission,
  issueModalEpicId,
  setIssueModalEpicId,
  editEpic,
  setEditEpic,
  editSprint,
  setEditSprint,
  completeSprintTarget,
  setCompleteSprintTarget,
  sprints,
}: IssueManagerModalsProps) {
  return (
    <>
      <MissionModal
        open={missionModalOpen}
        onClose={() => {
          setMissionModalOpen(false);
          setEditMission(null);
        }}
        mission={editMission ?? undefined}
      />
      <CreateSprintModal
        open={sprintModalOpen}
        onClose={() => setSprintModalOpen(false)}
      />
      <CreateEpicModal
        open={epicModalOpen}
        onClose={() => setEpicModalOpen(false)}
      />
      <CreateIssueModal
        open={issueModalEpicId != null}
        onClose={() => setIssueModalEpicId(null)}
        defaultEpicId={issueModalEpicId ?? undefined}
      />
      <EditEpicModal
        epic={editEpic}
        onClose={() => setEditEpic(null)}
      />
      <EditSprintModal
        sprint={editSprint}
        onClose={() => setEditSprint(null)}
      />
      {completeSprintTarget && (
        <ConfirmCompleteSprintModal
          isOpen={!!completeSprintTarget}
          onClose={() => setCompleteSprintTarget(null)}
          sprint={completeSprintTarget}
          sprints={sprints}
        />
      )}
    </>
  );
}
