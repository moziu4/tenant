use std::time::{Duration, UNIX_EPOCH};

use chrono::{NaiveDate, Utc};
use mongodb::bson::DateTime;

/// Convierte `NaiveDate` a `bson::DateTime` de MongoDB
pub fn naive_date_to_bson_datetime(date: NaiveDate) -> DateTime
{
    // Convertir `NaiveDate` a un `SystemTime`
    let d = date.and_hms_opt(0, 0, 0).unwrap_or_default(); // Hora en 00:00:00
    let unix_time = d.and_utc().timestamp();
    let duration_since_epoch = Duration::from_secs(unix_time as u64);

    // Convertir a `SystemTime`
    let system_time = UNIX_EPOCH + duration_since_epoch;

    // Devolver `bson::DateTime`
    DateTime::from_system_time(system_time)
}

/// Convierte `bson::DateTime` a `NaiveDate` de chrono
pub fn bson_datetime_to_naive_date(bson_dt: DateTime) -> NaiveDate
{
    // Convertir `bson::DateTime` a `SystemTime`
    let system_time = bson_dt.to_system_time();
    // Convertir a `chrono::DateTime<Utc>`
    let chrono_time = chrono::DateTime::<Utc>::from(system_time);
    // Devolver solo la parte de la fecha
    chrono_time.naive_utc().date()
}
