import { useQuery } from '@tanstack/react-query';
import { sprintList } from '../ipc/invoke';

export function useSprints() {
  return useQuery({
    queryKey: ['sprints'],
    queryFn: () => sprintList(),
  });
}
