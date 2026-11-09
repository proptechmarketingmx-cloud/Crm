use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::utils::errors::CrmResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyPipelineValue {
    pub currency: String,
    pub total_value: f64,
    pub deal_count: i64,
}

pub struct DashboardCounts {
    pub total_contacts: i64,
    pub total_organizations: i64,
    pub new_contacts_this_month: i64,
    pub new_deals_this_month: i64,
}

pub fn get_dashboard_counts(conn: &Connection, month_prefix: &str) -> CrmResult<DashboardCounts> {
    let total_contacts = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE deleted_at IS NULL AND contact_type = 'person'",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let total_organizations = conn
        .query_row(
            r#"
            SELECT
                (SELECT COUNT(*) FROM contacts
                 WHERE deleted_at IS NULL AND contact_type = 'organization')
                +
                (SELECT COUNT(*) FROM organizations
                 WHERE deleted_at IS NULL)
            "#,
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let new_contacts_this_month = conn
        .query_row(
            "SELECT COUNT(*) FROM contacts WHERE deleted_at IS NULL AND created_at LIKE ?1",
            params![month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let new_deals_this_month = conn
        .query_row(
            "SELECT COUNT(*) FROM deals WHERE deleted_at IS NULL AND created_at LIKE ?1",
            params![month_prefix],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(DashboardCounts {
        total_contacts,
        total_organizations,
        new_contacts_this_month,
        new_deals_this_month,
    })
}

pub fn get_pipeline_value_by_currency(conn: &Connection) -> CrmResult<Vec<CurrencyPipelineValue>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT currency, COALESCE(SUM(value), 0), COUNT(*)
        FROM deals
        WHERE deleted_at IS NULL
          AND stage NOT IN ('Closed Won', 'Closed Lost')
        GROUP BY currency
        ORDER BY ABS(SUM(value)) DESC, currency ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(CurrencyPipelineValue {
            currency: row
                .get::<_, String>(0)
                .unwrap_or_else(|_| "USD".to_string()),
            total_value: row.get::<_, f64>(1).unwrap_or(0.0),
            deal_count: row.get::<_, i64>(2).unwrap_or(0),
        })
    })?;

    let mut values = Vec::new();
    for row in rows {
        values.push(row?);
    }
    Ok(values)
}
