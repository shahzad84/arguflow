use crate::data::models::{PasswordReset, Pool, User};
use crate::handlers::register_handler::hash_password;
use crate::{diesel::prelude::*, errors::ServiceError};
use actix_web::web;
use crate::services::email_service::send_password_reset;

pub fn reset_user_password(
    password_reset_id: String,
    password: String,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    let password_reset = get_password_reset_query(password_reset_id, &pool)?;

    // check if password reset is expired
    if password_reset.expires_at < chrono::Local::now().naive_local() {
        return Err(ServiceError::BadRequest("Password Reset Expired".into()));
    }

    reset_user_password_query(password_reset, password, &pool)?;

    Ok(())
}

pub fn send_password_reset_email(
    user_email: String,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let user: User = users
        .filter(email.eq(user_email))
        .first::<User>(&mut conn)
        .map_err(|_db_error| ServiceError::BadRequest("User Does Not Exist".into()))?;

    
    let password_reset = create_password_reset_query(user.email, pool)?;

    send_password_reset(&password_reset)?;

    Ok(())
}

fn create_password_reset_query(
    email: String,
    pool: &web::Data<Pool>,
) -> Result<PasswordReset, crate::errors::ServiceError> {
    use crate::data::schema::password_resets::dsl::password_resets;

    let mut conn = pool.get().unwrap();

    let new_password_reset = PasswordReset::from(email);
    // log the password reset
    log::info!("New Password Reset: {:?}", &new_password_reset);

    let inserted_password_reset = diesel::insert_into(password_resets)
        .values(&new_password_reset)
        .get_result(&mut conn)?;

        log::info!("Inserted Password Reset: {:?}", &inserted_password_reset);

    Ok(inserted_password_reset)
}

fn get_password_reset_query(
    password_reset_id: String,
    pool: &web::Data<Pool>,
) -> Result<PasswordReset, ServiceError> {
    use crate::data::schema::password_resets::dsl::*;

    let mut conn = pool.get().unwrap();

    let password_reset_id = uuid::Uuid::try_parse(&password_reset_id)?;

    let password_reset = password_resets
        .find(password_reset_id)
        .first(&mut conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid Password Reset".into()))?;

    Ok(password_reset)
}

fn reset_user_password_query(
    password_reset: PasswordReset,
    password: String,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let password: String = hash_password(&password)?;

    diesel::update(users.find(password_reset.email))
        .set(hash.eq(password))
        .execute(&mut conn)?;

    Ok(())
}