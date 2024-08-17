use chrono::NaiveDateTime;
use diesel::{deserialize::Queryable, prelude::Insertable, Selectable};
use serde::{Deserialize, Serialize};

use crate::database::databse::get_dbo;
use crate::schema::message;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Clone, Debug)]
#[diesel(table_name = message)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Message {
    pub id: i32,
    pub message_id: String,
    pub lc_channel: String,
    pub created_at: NaiveDateTime,
}

impl Message {
    pub fn new(lc_channel: String, message_id: String) -> Self {
        Self {
            id: 0,
            message_id,
            lc_channel,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn insert(data: Message) -> Result<(), diesel::result::Error> {
        let conn = &mut get_dbo();
        diesel::insert_into(message::table)
            .values(data)
            .execute(conn)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<(), diesel::result::Error> {
        let conn = &mut get_dbo();
        diesel::delete(message::table)
            .filter(message::id.eq(self.id))
            .execute(conn)?;
        Ok(())
    }

    pub fn update(data: Message) -> Result<(), diesel::result::Error> {
        let conn = &mut get_dbo();
        diesel::update(message::table)
            .filter(message::id.eq(data.id))
            .set((
                message::message_id.eq(data.message_id),
                message::lc_channel.eq(data.lc_channel),
            ))
            .execute(conn)?;
        Ok(())
    }

    pub fn find_all() -> Result<Vec<Self>, diesel::result::Error> {
        let conn = &mut get_dbo();
        message::table.load(conn)
    }

    pub fn find_by_channel_id(channel_id: String) -> Result<Self, diesel::result::Error> {
        let conn = &mut get_dbo();
        message::table
            .filter(message::message_id.eq(channel_id))
            .first(conn)
    }
}
