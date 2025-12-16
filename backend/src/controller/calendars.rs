use rootcause::Report;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use uuid::Uuid;

use crate::entity::calendar;

pub struct CalendarData {
    pub title: String,
    pub colour: String,
}

pub async fn create_calendar_for_user(
    db: &DatabaseConnection,
    user_id: Uuid,
    data: CalendarData,
) -> Result<(), Report> {
    let calendar = calendar::ActiveModel {
        id: Set(Uuid::new_v4()),
        title: Set(data.title),
        colour: Set(data.colour),
        owner: Set(user_id),
        c_tag: Set(Uuid::new_v4()),
        ..Default::default()
    };

    calendar.insert(db).await?;
    Ok(())
}
