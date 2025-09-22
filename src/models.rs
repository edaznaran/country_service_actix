use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::audit_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuditLog {
    pub id: i32,
    pub action: String,
    pub details: serde_json::Value,
    #[diesel(column_name = createdAt)]
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::country)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Country {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub dial_code: String,
    #[diesel(column_name = createdAt)]
    pub created_at: chrono::NaiveDateTime,
    #[diesel(column_name = updateAt)]
    pub updated_at: chrono::NaiveDateTime,
}
