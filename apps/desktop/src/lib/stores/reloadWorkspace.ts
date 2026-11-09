import { activityStore } from './activities';
import { contactStore } from './contacts';
import { dealStore } from './deals';
import { organizationStore } from './organizations';
import { settingsStore } from './settings';
import { reviewCountsStore } from './reviewCounts';

/**
 * Reload in-memory stores after the SQLite file is replaced (backup restore
 * or import rollback restore).
 */
export async function reloadWorkspaceAfterDataReplace(): Promise<void> {
  contactStore.selectedContact = null;
  dealStore.selectedDeal = null;
  organizationStore.selectedOrganization = null;

  await settingsStore.loadSettings();
  await Promise.all([
    contactStore.loadContacts(),
    dealStore.loadDeals(),
    dealStore.loadPipelineBoard(),
    dealStore.loadPipelineSummary(),
    activityStore.loadActivities(),
    activityStore.loadUpcoming(),
    organizationStore.loadOrganizations(),
    reviewCountsStore.refresh(),
  ]);
}
