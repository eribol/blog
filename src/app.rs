use actix_session::Session;
use actix_web::{Error, error, http, HttpResponse, web};
use tera::{Context, Tera};
use serde::*;
use validator::{Validate};
use bcrypt::bcrypt;
use bcrypt::DEFAULT_COST;

#[derive(Deserialize, Validate)]
pub struct LoginUser{
    #[validate(email)]
    email: String,
    password: String
}
#[derive(Debug, Deserialize, Validate)]
pub struct SigninUser{
    #[validate(email)]
    email: String,
    #[validate(length(min=5))]
    username: String,
    #[validate(must_match = "password2", length(min=6))]
    password: String,
    password2: String
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct User{
    id: i32,
    email: String,
    username: String,
    password: String
}
pub async fn index(tmpl: web::Data<Tera>, session: Session) -> Result<HttpResponse, Error> {
    let mut ctx = Context::new();
    if let Some(user) = session.get::<String>("user")?{
        ctx.insert("user", &user);
    }
    let a = tmpl.render("index.html", &ctx).map_err(error::ErrorInternalServerError)?;
    //.map_err("Err");
    Ok(HttpResponse::Ok().body(a))
}

pub async fn login(tmpl: web::Data<Tera>, session: Session) -> Result<HttpResponse, Error>{
    if let Some(_) = session.get::<String>("user")?{
        return Ok(redirect("/"))
    }
    let ctx = Context::new();
    let a = tmpl.render("login.html", &ctx).map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(a))
}

pub async fn post_login(tmpl: web::Data<Tera>, form: web::Form<LoginUser>, session: Session, conn: web::Data<sqlx::SqlitePool>) -> Result<HttpResponse, Error>{
    let login_form = form.into_inner();
    if let Ok(_) = login_form.validate(){
        let user: User = sqlx::query_as("select * from users where email = $1")
            .bind(&login_form.email)
            .fetch_one(&**conn).await.expect("AA");
        if let Ok(_) = bcrypt::verify(&login_form.password, &user.password){
            session.insert("user", &login_form.email)?;
            return Ok(redirect("/"))
        }
    }
    Ok(redirect("/login"))
}

pub async fn signin(tmpl: web::Data<Tera>, session: Session) -> Result<HttpResponse, Error>{
    if let Some(_) = session.get::<String>("user")?{
        return Ok(redirect("/"))
    }
    let ctx = Context::new();
    let a = tmpl.render("signin.html", &ctx).map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().body(a))
}

pub async fn post_signin(
    _tmpl: web::Data<Tera>,
    form: web::Form<SigninUser>,
    session: Session,
    conn: web::Data<sqlx::SqlitePool>) -> Result<HttpResponse, Error>{
    //let ctx = Context::new();
    let user = form.into_inner();
    if let Ok(_) = user.validate(){
        let add_user = sqlx::query("insert into users (username, email, password) values($1,$2,$3)")
            .bind(&user.username)
            .bind(&user.email)
            .bind(&bcrypt::hash(&user.password, DEFAULT_COST).expect("Şifreleme hatalı")).execute(&**conn).await;
        match add_user{
            Ok(_) => {
                session.insert("user", &user.username)?;
                return Ok(redirect("/"))
            }
            Err(_) => {
                return Ok(redirect("/signin"))
            }
        }
    }
    Ok(redirect("/signin"))
}
pub async fn logout(session: Session) -> Result<HttpResponse, Error>{
    session.purge();
    Ok(redirect("/"))
}


fn redirect(location: &str)-> HttpResponse{
    HttpResponse::Found()
        .append_header((http::header::LOCATION, location))
        .finish()
}