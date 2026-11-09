use serde::{Deserialize, Serialize};

use crate::{result::CrmResult, storage};

use super::CrmCore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationMigrationPreflight {
    pub legacy_organization_contacts: i64,
    pub contacts_with_org_id_missing_organization_id: i64,
    pub contacts_with_invalid_legacy_org_id_links: i64,
    pub contacts_with_invalid_normalized_organization_id_links: i64,
    pub backup_restore_baseline_available: bool,
}

impl CrmCore {
    pub fn normalization_migration_preflight(&self) -> CrmResult<NormalizationMigrationPreflight> {
        let counts =
            storage::migration_readiness::get_normalization_migration_readiness(&self.db.conn)?;

        Ok(NormalizationMigrationPreflight {
            legacy_organization_contacts: counts.legacy_organization_contacts,
            contacts_with_org_id_missing_organization_id: counts
                .contacts_with_org_id_missing_organization_id,
            contacts_with_invalid_legacy_org_id_links: counts
                .contacts_with_invalid_legacy_org_id_links,
            contacts_with_invalid_normalized_organization_id_links: counts
                .contacts_with_invalid_normalized_organization_id_links,
            backup_restore_baseline_available: counts.backup_restore_baseline_available,
        })
    }
}
