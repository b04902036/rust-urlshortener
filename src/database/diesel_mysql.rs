use rocket::fairing::AdHoc;
use rocket_db_pools::diesel::MysqlPool;
use rocket_db_pools::Database;

#[derive(Database)]
#[database("diesel_mysql")]
pub struct Db(MysqlPool);

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("diesel_mysql starting", |rocket| async {
        rocket.attach(Db::init())
    })
}
