//! # BundlesDB
//! Database handler for all database types
use super::{
    cachedb::CacheDB,
    sql::{self, Database, DatabaseOpts},
};

use sqlx::{Column, Row};

use crate::utility;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Clone)]
pub struct AppData {
    pub db: BundlesDB,
    pub http_client: awc::Client,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Default API return value
pub struct DefaultReturn<T> {
    pub success: bool,
    pub message: String,
    pub payload: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseReturn {
    pub data: HashMap<String, String>,
}

// Paste and Group require the type of their metadata to be specified so it can be converted if needed
#[derive(Debug, Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Paste<M> {
    // selectors
    pub custom_url: String,
    pub id: String,
    pub group_name: String,
    // passwords
    pub edit_password: String,
    // dates
    pub pub_date: u128,
    pub edit_date: u128,
    // ...
    pub content: String,
    pub content_html: String, // rendered paste content
    //                           storing the rendered content in the database will save like 100ms when loading pastes!
    // ...
    pub metadata: M,
    pub views: usize,
}

#[derive(Debug, Default, sqlx::FromRow, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasteIdentifier {
    pub custom_url: String,
    pub id: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PasteMetadata {
    pub owner: String,
    pub private_source: String,
    // optionals
    pub title: Option<String>,
    pub description: Option<String>,
    pub favicon: Option<String>,
    pub embed_color: Option<String>,
    pub view_password: Option<String>,
    pub page_template: Option<String>, // handlebars formatted page template
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullPaste<M, U> {
    pub paste: Paste<M>,
    pub user: Option<FullUser<U>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A paste content structure containing an array of [files](AtomicPasteFSFile)
pub struct AtomicPaste {
    // atomic pastes are a plain JSON file system storing HTML, CSS, and JS files only
    // they have the least amount of boilerplate for rendering!
    pub _is_atomic: bool, // this must exist so we know a paste's content is for an atomic paste
    pub files: Vec<AtomicPasteFSFile>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
/// A structure representing a single text file
pub struct AtomicPasteFSFile {
    // store only the bare minimum for the required file types
    pub path: String,
    pub content: String,
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Group<M> {
    // selectors
    pub name: String,
    // passwords
    pub submit_password: String,
    // ...
    pub metadata: M, // JSON Object
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GroupMetadata {
    pub owner: String, // username of owner
}

#[derive(Debug, Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
/// A user object
pub struct UserState<M> {
    // selectors
    pub username: String,
    pub id_hashed: String, // users use their UNHASHED id to login, it is used as their session id too!
    //                        the hashed id is the only id that should ever be public!
    pub role: String,
    // dates
    pub timestamp: u128,
    // ...
    pub metadata: M,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleLevel {
    pub elevation: i32, // this marks the level of the role, 0 should always be member
    // users cannot manage users of a higher elevation than them
    pub name: String,             // role name, shown on user profiles
    pub permissions: Vec<String>, // a vec of user permissions (ex: "ManagePastes")
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullUser<M> {
    pub user: UserState<M>,
    pub level: RoleLevel,
}

#[derive(Default, Clone, sqlx::FromRow, Serialize, Deserialize, PartialEq)]
pub struct RoleLevelLog {
    pub id: String,
    pub level: RoleLevel,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMetadata {
    pub about: String,
    pub avatar_url: Option<String>,
    pub secondary_token: Option<String>,
    pub allow_mail: Option<String>,    // yes/no
    pub nickname: Option<String>,      // user display name
    pub page_template: Option<String>, // profile handlebars template
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Log {
    // selectors
    pub id: String,
    pub logtype: String,
    // dates
    pub timestamp: u128,
    // ...
    pub content: String,
}

#[derive(Debug, Default, sqlx::FromRow, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogIdentifier {
    pub id: String,
}

#[derive(Default, PartialEq, sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Board<M> {
    // selectors
    pub name: String,
    // dates
    pub timestamp: u128,
    // ...
    pub metadata: M,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardMetadata {
    pub owner: String,                      // username of owner
    pub is_private: String, // if the homepage of the board is shown to other users (not owner)
    pub allow_anonymous: Option<String>, // if anonymous users can post
    pub allow_open_posting: Option<String>, // if all users can post on the board (not just owner)
    pub topic_required: Option<String>, // if posts are required to include a topic value
    pub about: Option<String>, // welcome message
    pub tags: Option<String>, // SPACE separated list of tags that identify the board for searches
                            // TODO: we could likely export a list of "valid" tags at some point in the future
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardPostLog {
    pub author: String, // username of owner
    pub content: String,
    pub content_html: String,
    pub topic: Option<String>, // post topic, content is hidden unless expanded if provided
    pub board: String,         // name of board the post is located in
    pub is_hidden: bool,       // if the post is hidden in the feed (does nothing right now)
    pub reply: Option<String>, // the ID of the post we're replying to
    pub pinned: Option<bool>,  // pin the post to the top of the board feed
    pub replies: Option<usize>, // not really managed in the log, just used to show the number of replies this post has
    pub tags: Option<String>,   // same as board tags, just for posts specifically
}

#[derive(Debug, Default, sqlx::FromRow, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoardIdentifier {
    pub name: String,
    pub tags: String,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
// Takes the place of "about" in BoardMetadata, identifies a board as a user mail stream
pub struct UserMailStreamIdentifier {
    pub _is_user_mail_stream: bool, // always going to be true ... cannot be edited into board about ANYWHERE
    pub user1: String,              // username of first user
    pub user2: String,              // username of second user
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserFollow {
    pub user: String,         // the user that is following `is_following`
    pub is_following: String, // use user that `user` is following
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub user: String,    // the user that is being notified
    pub content: String, // notification text
    pub address: String, // notification redirect url
}

// ...
#[derive(Clone)]
#[cfg(feature = "postgres")]
pub struct BundlesDB {
    pub db: Database<sqlx::PgPool>,
    pub options: DatabaseOpts,
    pub cachedb: CacheDB,
}

#[derive(Clone)]
#[cfg(feature = "mysql")]
pub struct BundlesDB {
    pub db: Database<sqlx::MySqlPool>,
    pub options: DatabaseOpts,
    pub cachedb: CacheDB,
}

#[derive(Clone)]
#[cfg(feature = "sqlite")]
pub struct BundlesDB {
    pub db: Database<sqlx::SqlitePool>,
    pub options: DatabaseOpts,
    pub cachedb: CacheDB,
}

impl BundlesDB {
    pub async fn new(options: DatabaseOpts) -> BundlesDB {
        return BundlesDB {
            db: sql::create_db(options.clone()).await,
            options,
            cachedb: CacheDB::new().await,
        };
    }

    pub async fn init(&self) {
        // create tables
        let c = &self.db.client;
        // MAX = 1000000
        // we're just using the same max length for everything because lengths are checked before being sent to db

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Pastes\" (
                custom_url VARCHAR(1000000),
                id VARCHAR(1000000),
                group_name VARCHAR(1000000),
                edit_password VARCHAR(1000000),
                pub_date VARCHAR(1000000),
                edit_date VARCHAR(1000000),
                content VARCHAR(1000000),
                content_html VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Groups\" (
                name VARCHAR(1000000),
                submit_password VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Users\" (
                username VARCHAR(1000000),
                id_hashed VARCHAR(1000000),
                role VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id VARCHAR(1000000),
                logtype VARCHAR(1000000),
                timestamp  VARCHAR(1000000),
                content VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlx::query(
            "CREATE TABLE IF NOT EXISTS \"Boards\" (
                name VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;
    }

    #[cfg(feature = "sqlite")]
    fn textify_row(&self, row: sqlx::sqlite::SqliteRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.get(column.name());
            out.insert(column.name().to_string(), value);
        }

        // return
        return DatabaseReturn { data: out };
    }

    #[cfg(feature = "postgres")]
    fn textify_row(&self, row: sqlx::postgres::PgRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.get(column.name());
            out.insert(column.name().to_string(), value);
        }

        // return
        return DatabaseReturn { data: out };
    }

    #[cfg(feature = "mysql")]
    fn textify_row(&self, row: sqlx::mysql::MySqlRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.try_get::<Vec<u8>, _>(column.name());

            if value.is_ok() {
                // returned bytes instead of text :(
                // we're going to convert this to a string and then add it to the output!
                out.insert(
                    column.name().to_string(),
                    std::str::from_utf8(value.unwrap().as_slice())
                        .unwrap()
                        .to_string(),
                );
            } else {
                // already text
                let value = row.get(column.name());
                out.insert(column.name().to_string(), value);
            }
        }

        // return
        return DatabaseReturn { data: out };
    }

    // users

    // GET
    /// Get a user by their hashed ID
    ///
    /// # Arguments:
    /// * `hashed` - `String` of the user's hashed ID
    pub async fn get_user_by_hashed(
        &self,
        hashed: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        // fetch from database
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&hashed)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        let role = row.get("role").unwrap().to_string();
        if role == "banned" {
            return DefaultReturn {
                success: false,
                message: String::from("User is banned"),
                payload: Option::None,
            };
        }

        // ...
        let meta = row.get("metadata"); // for compatability - users did not have metadata until Bundlrs v0.10.6
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role: role.clone(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: if meta.is_some() {
                meta.unwrap().to_string()
            } else {
                String::new()
            },
        };

        // fetch level from role
        let level = self.get_level_by_role(role).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(FullUser {
                user,
                level: level.payload.level,
            }),
        };
    }

    /// Get a user by their unhashed ID (hashes ID and then calls [`BundlesDB::get_user_by_hashed()`])
    ///
    /// Calls [`BundlesDB::get_user_by_unhashed_st()`] if user is invalid.
    ///
    /// # Arguments:
    /// * `unhashed` - `String` of the user's unhashed ID
    pub async fn get_user_by_unhashed(
        &self,
        unhashed: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        let res = self
            .get_user_by_hashed(utility::hash(unhashed.clone()))
            .await;

        if res.success == false {
            // treat unhashed as a secondary token and try again
            return self.get_user_by_unhashed_st(unhashed).await;
        }

        res
    }

    /// Get a user by their unhashed secondary token
    ///
    /// # Arguments:
    /// * `unhashed` - `String` of the user's unhashed secondary token
    pub async fn get_user_by_unhashed_st(
        &self,
        unhashed: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        // fetch from database
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"metadata\" LIKE ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"metadata\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!(
                "%\"secondary_token\":\"{}\"%",
                crate::utility::hash(unhashed)
            ))
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        let role = row.get("role").unwrap().to_string();
        if role == "banned" {
            return DefaultReturn {
                success: false,
                message: String::from("User is banned"),
                payload: Option::None,
            };
        }

        // ...
        let meta = row.get("metadata"); // for compatability - users did not have metadata until Bundlrs v0.10.6
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role: role.clone(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: if meta.is_some() {
                meta.unwrap().to_string()
            } else {
                String::new()
            },
        };

        // fetch level from role
        let level = self.get_level_by_role(role).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(FullUser {
                user,
                level: level.payload.level,
            }),
        };
    }

    /// Get a user by their username
    ///
    /// # Arguments:
    /// * `username` - `String` of the user's username
    pub async fn get_user_by_username(
        &self,
        username: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        // check in cache
        let cached = self.cachedb.get(format!("user:{}", username)).await;

        if cached.is_some() {
            // ...
            let user = serde_json::from_str::<UserState<String>>(cached.unwrap().as_str()).unwrap();

            // get role
            let role = user.role.clone();
            if role == "banned" {
                // account banned - we're going to act like it simply does not exist
                return DefaultReturn {
                    success: false,
                    message: String::from("User is banned"),
                    payload: Option::None,
                };
            }

            // fetch level from role
            let level = self.get_level_by_role(role.clone()).await;

            // ...
            return DefaultReturn {
                success: true,
                message: String::from("User exists (cache)"),
                payload: Option::Some(FullUser {
                    user,
                    level: level.payload.level,
                }),
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"username\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"username\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&username)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        let role = row.get("role").unwrap().to_string();
        if role == "banned" {
            // account banned - we're going to act like it simply does not exist
            return DefaultReturn {
                success: false,
                message: String::from("User is banned"),
                payload: Option::None,
            };
        }

        // fetch level from role
        let level = self.get_level_by_role(role.clone()).await;

        // store in cache
        let meta = row.get("metadata");
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role,
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: if meta.is_some() {
                meta.unwrap().to_string()
            } else {
                String::new()
            },
        };

        self.cachedb
            .set(
                format!("user:{}", username),
                serde_json::to_string::<UserState<String>>(&user).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists (new)"),
            payload: Option::Some(FullUser {
                user,
                level: level.payload.level,
            }),
        };
    }

    /// Get a [`RoleLevel`] by its `name`
    ///
    /// # Arguments:
    /// * `name` - `String` of the level's role name
    pub async fn get_level_by_role(&self, name: String) -> DefaultReturn<RoleLevelLog> {
        // check if level already exists in cache
        let cached = self.cachedb.get(format!("level:{}", name)).await;

        if cached.is_some() {
            return DefaultReturn {
                success: true,
                message: String::from("Level exists (cache)"),
                payload: serde_json::from_str::<RoleLevelLog>(cached.unwrap().as_str()).unwrap(),
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'level' AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'level' AND \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"name\":\"{}\"%", name))
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: true,
                message: String::from("Level does not exist, using default"),
                payload: RoleLevelLog {
                    id: String::new(),
                    level: RoleLevel {
                        name: String::from("member"),
                        elevation: 0,
                        permissions: Vec::new(),
                    },
                },
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // store in cache
        let id = row.get("id").unwrap().to_string();
        let level = serde_json::from_str::<RoleLevel>(row.get("content").unwrap()).unwrap();

        let level = RoleLevelLog { id, level };
        self.cachedb
            .set(
                format!("level:{}", name),
                serde_json::to_string::<RoleLevelLog>(&level).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Level exists (new)"),
            payload: level,
        };
    }

    // SET
    /// Create a new user given their username. Returns their hashed ID
    ///
    /// # Arguments:
    /// * `username` - `String` of the user's `username`
    pub async fn create_user(&self, username: String) -> DefaultReturn<Option<String>> {
        // make sure user doesn't already exists
        let existing = &self.get_user_by_username(username.clone()).await;
        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User already exists!"),
                payload: Option::None,
            };
        }

        // check username
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&username).iter().len() < 1 {
            return DefaultReturn {
                success: false,
                message: String::from("Username is invalid"),
                payload: Option::None,
            };
        }

        if (username.len() < 2) | (username.len() > 500) {
            return DefaultReturn {
                success: false,
                message: String::from("Username is invalid"),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Users\" VALUES (?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Users\" VALUES ($1, $2, $3, $4, $5)"
        };

        let user_id_unhashed: String = utility::uuid();
        let user_id_hashed: String = utility::hash(user_id_unhashed.clone());
        let timestamp = utility::unix_epoch_timestamp().to_string();

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&username)
            .bind::<&String>(&user_id_hashed)
            .bind::<&String>(&String::from("member")) // default role
            .bind::<&String>(&timestamp)
            .bind::<&String>(
                &serde_json::to_string::<UserMetadata>(&UserMetadata {
                    about: String::new(),
                    avatar_url: Option::None,
                    secondary_token: Option::None,
                    allow_mail: Option::Some(String::from("yes")),
                    nickname: Option::Some(username.clone()),
                    page_template: Option::Some(crate::pages::auth::profile_view_hb_template()),
                })
                .unwrap(),
            )
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: user_id_unhashed,
            payload: Option::Some(user_id_hashed),
        };
    }

    /// Update a [`UserState`]'s metadata by its `username`
    pub async fn edit_user_metadata_by_name(
        &self,
        name: String,
        metadata: UserMetadata,
    ) -> DefaultReturn<Option<String>> {
        // make sure user exists
        let existing = &self.get_user_by_username(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist!"),
                payload: Option::None,
            };
        }

        // update user
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Users\" SET \"metadata\" = ? WHERE \"username\" = ?"
        } else {
            "UPDATE \"Users\" SET (\"metadata\") = ($1) WHERE \"username\" = $2"
        };

        let c = &self.db.client;
        let meta = &serde_json::to_string(&metadata).unwrap();
        let res = sqlx::query(query)
            .bind::<&String>(meta)
            .bind::<&String>(&name)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.cachedb.get(format!("user:{}", name)).await;

        if existing_in_cache.is_some() {
            let mut user =
                serde_json::from_str::<UserState<String>>(&existing_in_cache.unwrap()).unwrap();
            user.metadata = meta.to_string(); // update metadata

            // update cache
            self.cachedb
                .update(
                    format!("user:{}", name),
                    serde_json::to_string::<UserState<String>>(&user).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User updated!"),
            payload: Option::Some(name),
        };
    }

    /// Ban a [`UserState`] by its `username`
    pub async fn ban_user_by_name(&self, name: String) -> DefaultReturn<Option<String>> {
        // make sure user exists
        let existing = &self.get_user_by_username(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist!"),
                payload: Option::None,
            };
        }

        // make sure user role is "member"
        let user = &existing.payload.as_ref().unwrap().user;
        if user.role != "member" {
            return DefaultReturn {
                success: false,
                message: String::from("User must be of role \"member\""),
                payload: Option::None,
            };
        }

        // update user
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Users\" SET \"role\" = ? WHERE \"username\" = ?"
        } else {
            "UPDATE \"Users\" SET (\"role\") = ($1) WHERE \"username\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&str>("banned")
            .bind::<&String>(&name)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // lock user assets
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"metadata\" = ? WHERE \"metadata\" LIKE ?"
        } else {
            "UPDATE \"Pastes\" SET (\"metadata\") = ($1) WHERE \"metadata\" LIKE $2"
        };

        let c = &self.db.client;
        // TODO: some kind of bulk cache update to handle this
        let res = sqlx::query(query)
            .bind::<&String>(
                &serde_json::to_string::<PasteMetadata>(&PasteMetadata {
                    // lock editors out
                    owner: String::new(),
                    private_source: String::from("on"),
                    // optionals
                    title: Option::Some(String::new()),
                    description: Option::Some(String::new()),
                    favicon: Option::None,
                    embed_color: Option::None,
                    view_password: Option::Some(format!(
                        "LOCKED(USER_BANNED)-{}",
                        crate::utility::random_id()
                    )),
                    page_template: Option::Some(String::new()),
                })
                .unwrap(),
            )
            .bind::<&String>(&format!("%\"owner\":\"{name}\"%"))
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.cachedb.get(format!("user:{}", name)).await;

        if existing_in_cache.is_some() {
            let mut user =
                serde_json::from_str::<UserState<String>>(&existing_in_cache.unwrap()).unwrap();
            user.role = String::from("banned"); // update role

            // update cache
            self.cachedb
                .update(
                    format!("user:{}", name),
                    serde_json::to_string::<UserState<String>>(&user).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User banned!"),
            payload: Option::Some(name),
        };
    }

    /*
    /// Create a new [`RoleLevel`] given various properties
    ///
    /// # Arguments:
    /// * `props` - [`RoleLevel`]
    pub async fn create_level(&self, props: &mut RoleLevel) -> DefaultReturn<RoleLevelLog> {
        let p: &mut RoleLevel = props; // borrowed props

        // make sure role does not exist
        let existing: DefaultReturn<RoleLevelLog> = self.get_level_by_role(p.name.to_owned()).await;

        if existing.success {
            return existing;
        }

        // create role
        let res = self
            .create_log(
                String::from("level"),
                serde_json::to_string::<RoleLevel>(p).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: res.success,
            message: res.message,
            payload: RoleLevelLog {
                id: if res.success {
                    res.payload.unwrap()
                } else {
                    String::new()
                },
                level: p.to_owned(),
            },
        };
    }

    /// Delete an existing [`RoleLevel`] given its `name`
    ///
    /// # Arguments:
    /// * `name` - [`RoleLevel`] `name` field
    pub async fn delete_level(&self, props: &mut RoleLevel) -> DefaultReturn<Option<String>> {
        let p: &mut RoleLevel = props; // borrowed props

        // make sure role exists
        let existing: DefaultReturn<RoleLevelLog> = self.get_level_by_role(p.name.to_owned()).await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Level does not exist"),
                payload: Option::None,
            };
        }

        // delete role
        let res = self.delete_log(existing.payload.id.clone()).await;

        // return
        return DefaultReturn {
            success: res.success,
            message: res.message,
            payload: Option::Some(existing.payload.id),
        };
    }
    */

    // logs

    // GET
    /// Get a log by its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn get_log_by_id(&self, id: String) -> DefaultReturn<Option<Log>> {
        // check in cache
        let cached = self.cachedb.get(format!("log:{}", id)).await;

        if cached.is_some() {
            // ...
            let log = serde_json::from_str::<Log>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Log exists (cache)"),
                payload: Option::Some(log),
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&id).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Log does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // store in cache
        let log = Log {
            id: row.get("id").unwrap().to_string(),
            logtype: row.get("logtype").unwrap().to_string(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            content: row.get("content").unwrap().to_string(),
        };

        self.cachedb
            .set(
                format!("log:{}", id),
                serde_json::to_string::<Log>(&log).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste exists"),
            payload: Option::Some(log),
        };
    }

    // SET
    /// Create a log given its type and content
    ///
    /// # Arguments:
    /// * `logtype` - `String` of the log's `logtype`
    /// * `content` - `String` of the log's `content`
    pub async fn create_log(
        &self,
        logtype: String,
        content: String,
    ) -> DefaultReturn<Option<String>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Logs\" VALUES (?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Logs\" VALUES ($1, $2, $3, $4)"
        };

        let log_id: String = utility::random_id();

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&log_id)
            .bind::<String>(logtype)
            .bind::<String>(utility::unix_epoch_timestamp().to_string())
            .bind::<String>(content)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Log created!"),
            payload: Option::Some(log_id),
        };
    }

    /// Edit a log given its ID
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    /// * `content` - `String` of the log's new content
    pub async fn edit_log(&self, id: String, content: String) -> DefaultReturn<Option<String>> {
        // make sure log exists
        let existing = &self.get_log_by_id(id.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Log does not exist!"),
                payload: Option::None,
            };
        }

        // update log
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Logs\" SET \"content\" = ? WHERE \"id\" = ?"
        } else {
            "UPDATE \"Logs\" SET (\"content\") = ($1) WHERE \"id\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&content)
            .bind::<&String>(&id)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.cachedb.get(format!("log:{}", id)).await;

        if existing_in_cache.is_some() {
            let mut log = serde_json::from_str::<Log>(&existing_in_cache.unwrap()).unwrap();

            log.content = content; // update content

            // update cache
            self.cachedb
                .update(
                    format!("log:{}", id),
                    serde_json::to_string::<Log>(&log).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Log updated!"),
            payload: Option::Some(id),
        };
    }

    /// Delete a log given its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn delete_log(&self, id: String) -> DefaultReturn<Option<String>> {
        // make sure log exists
        let existing = &self.get_log_by_id(id.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Log does not exist!"),
                payload: Option::None,
            };
        }

        // update log
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&id).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        self.cachedb.remove(format!("log:{}", id)).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Log deleted!"),
            payload: Option::Some(id),
        };
    }

    // pastes

    /// Count the `view_paste` logs for a specific [`Paste`]
    async fn count_paste_views(&self, custom_url: String) -> usize {
        let c = &self.db.client;

        // count views
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let views_res = sqlx::query(query)
            .bind::<&String>(&format!("{}::%", &custom_url))
            .fetch_all(c)
            .await;

        if views_res.is_err() {
            return 0;
        }

        return views_res.unwrap().len();
    }

    /// Build a [`Paste`] query with information about it
    async fn build_result_from_query(
        &self,
        query: &str,
        selector: &str,
    ) -> DefaultReturn<Option<FullPaste<PasteMetadata, String>>> {
        // check in cache
        let cached = self.cachedb.get(format!("paste:{}", selector)).await;

        if cached.is_some() {
            // ...
            let paste =
                serde_json::from_str::<Paste<PasteMetadata>>(cached.unwrap().as_str()).unwrap();

            // get user
            let user = if paste.metadata.owner.len() > 0 {
                // TODO: maybe don't clone here
                (self
                    .get_user_by_username(paste.clone().metadata.owner)
                    .await)
                    .payload
            } else {
                Option::None
            };

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Paste exists (cache)"),
                payload: Option::Some(FullPaste { paste, user }),
            };
        }

        // fetch from db
        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&selector.to_lowercase())
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // get views
        let views = &self
            .count_paste_views(row.get("custom_url").unwrap().to_owned())
            .await;

        // ...
        let metadata = serde_json::from_str::<PasteMetadata>(row.get("metadata").unwrap()).unwrap();

        let paste = Paste {
            custom_url: row.get("custom_url").unwrap().to_string(),
            id: row.get("id").unwrap().to_string(),
            group_name: row.get("group_name").unwrap().to_string(),
            edit_password: row.get("edit_password").unwrap().to_string(),
            pub_date: row.get("pub_date").unwrap().parse::<u128>().unwrap(),
            edit_date: row.get("edit_date").unwrap().parse::<u128>().unwrap(),
            content: row.get("content").unwrap().to_string(),
            content_html: row.get("content_html").unwrap().to_string(),
            metadata,
            views: views.to_owned(),
        };

        // store in cache
        self.cachedb
            .set(
                format!("paste:{}", paste.custom_url),
                serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
            )
            .await;

        // get user
        let user = if paste.metadata.owner.len() > 0 {
            // TODO: maybe don't clone here
            (self
                .get_user_by_username(paste.clone().metadata.owner)
                .await)
                .payload
        } else {
            Option::None
        };

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste exists (new)"),
            payload: Option::Some(FullPaste { paste, user }),
        };
    }

    // GET
    /// Get a [`Paste`] given its `custom_url`
    ///
    /// # Arguments:
    /// * `url` - `String` of the paste's `custom_url`
    pub async fn get_paste_by_url(
        &self,
        url: String,
    ) -> DefaultReturn<Option<FullPaste<PasteMetadata, String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        return self.build_result_from_query(query, &url).await;
    }

    /// Get a [`Paste`] given its `id`
    ///
    /// # Arguments:
    /// * `id` - `String` of the paste's `id`
    pub async fn get_paste_by_id(
        &self,
        id: String,
    ) -> DefaultReturn<Option<FullPaste<PasteMetadata, String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"id\" = $1"
        };

        return self.build_result_from_query(query, &id).await;
    }

    /// Get all [pastes](Paste) owned by a specific user
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    pub async fn get_pastes_by_owner(
        &self,
        owner: String,
    ) -> DefaultReturn<Option<Vec<PasteIdentifier>>> {
        // check in cache
        let cached = self.cachedb.get(format!("pastes-by-owner:{}", owner)).await;

        if cached.is_some() {
            // ...
            let pastes =
                serde_json::from_str::<Vec<PasteIdentifier>>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: owner,
                payload: Option::Some(pastes),
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<PasteIdentifier> = Vec::new();

        for row in res.unwrap() {
            let row = self.textify_row(row).data;
            full_res.push(PasteIdentifier {
                custom_url: row.get("custom_url").unwrap().to_string(),
                id: row.get("id").unwrap().to_string(),
            });
        }

        // store in cache
        self.cachedb
            .set(
                format!("pastes-by-owner:{}", owner),
                serde_json::to_string::<Vec<PasteIdentifier>>(&full_res).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    /// Get all [pastes](Paste) owned by a specific user (limited)
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_pastes_by_owner_limited(
        &self,
        owner: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<PasteIdentifier>>> {
        let offset = if offset.is_some() { offset.unwrap() } else { 0 };

        // check in cache
        let cached = self
            .cachedb
            .get(format!("pastes-by-owner:{}:offset{}", owner, offset))
            .await;

        if cached.is_some() {
            // ...
            let pastes =
                serde_json::from_str::<Vec<PasteIdentifier>>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: owner,
                payload: Option::Some(pastes),
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE ? ORDER BY \"pub_date\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE $1 ORDER BY \"pub_date\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .bind(offset)
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<PasteIdentifier> = Vec::new();

        for row in res.unwrap() {
            let row = self.textify_row(row).data;
            full_res.push(PasteIdentifier {
                custom_url: row.get("custom_url").unwrap().to_string(),
                id: row.get("id").unwrap().to_string(),
            });
        }

        // store in cache
        self.cachedb
            .set(
                format!("pastes-by-owner:{}:offset{}", owner, offset),
                serde_json::to_string::<Vec<PasteIdentifier>>(&full_res).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    /// Get all atomic [pastes](Paste) owned by a specific user
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    pub async fn get_atomic_pastes_by_owner(
        &self,
        owner: String,
    ) -> DefaultReturn<Option<Vec<PasteIdentifier>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE ? AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Pastes\" WHERE \"metadata\" LIKE $1 AND \"content\" LIKE $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .bind("%\"_is_atomic\":true%")
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<PasteIdentifier> = Vec::new();

        for row in res.unwrap() {
            let row = self.textify_row(row).data;
            full_res.push(PasteIdentifier {
                custom_url: row.get("custom_url").unwrap().to_string(),
                id: row.get("id").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    // SET
    /// Create a new [`Paste`] given various properties
    ///
    /// # Arguments:
    /// * `props` - [`Paste<String>`](Paste)
    /// * `as_user` - The ID of the user creating the paste
    pub async fn create_paste(
        &self,
        props: &mut Paste<String>,
        as_user: Option<String>, // id of paste owner
    ) -> DefaultReturn<Option<Paste<String>>> {
        let p: &mut Paste<String> = props; // borrowed props

        // create default metadata
        let metadata: PasteMetadata = PasteMetadata {
            owner: if as_user.is_some() {
                as_user.clone().unwrap()
            } else {
                String::new()
            },
            private_source: String::from("off"),
            // optionals
            title: Option::Some(String::new()),
            description: Option::Some(String::new()),
            favicon: Option::None,
            embed_color: Option::Some(String::from("#ff9999")),
            view_password: Option::None,
            page_template: Option::Some(crate::pages::paste_view::paste_view_hb_template()),
        };

        // check values

        // (check empty)
        if p.custom_url.is_empty() {
            p.custom_url = utility::random_id().chars().take(10).collect();
        }

        if p.edit_password.is_empty() {
            p.edit_password = utility::random_id().chars().take(10).collect();
        }

        // (check length)
        if (p.custom_url.len() < 2) | (p.custom_url.len() > 500) {
            return DefaultReturn {
                success: false,
                message: String::from("Custom URL is invalid"),
                payload: Option::None,
            };
        }

        if !p.group_name.is_empty() && (p.group_name.len() < 2) | (p.group_name.len() > 500) {
            return DefaultReturn {
                success: false,
                message: String::from("Group Name is invalid"),
                payload: Option::None,
            };
        }

        // check content
        if (p.content.len() < 1) | (p.content.len() > 400_000) {
            return DefaultReturn {
                success: false,
                message: String::from("Content is invalid"),
                payload: Option::None,
            };
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&p.custom_url).iter().len() < 1 {
            return DefaultReturn {
                success: false,
                message: String::from("Custom URL is invalid"),
                payload: Option::None,
            };
        }

        // if we're trying to create a paste in a group, make sure the group exists
        // (create it if it doesn't)
        if !p.group_name.is_empty() {
            let n = &p.group_name;
            let e = &p.edit_password;
            let o = &p.custom_url;

            let existing_group = self.get_group_by_name(n.to_string()).await;

            if !existing_group.success {
                let res = self
                    .create_group(Group {
                        name: n.to_string(),
                        submit_password: e.to_string(), // groups will have the same password as their first paste
                        metadata: GroupMetadata {
                            owner: metadata.clone().owner,
                        },
                    })
                    .await;

                if !res.success {
                    return DefaultReturn {
                        success: false,
                        message: res.message,
                        payload: Option::None,
                    };
                }
            } else {
                // check group password
                if utility::hash(e.to_string()) != existing_group.payload.unwrap().submit_password {
                    return DefaultReturn {
                        success: false,
                        message: String::from("The paste edit password must match the group submit password during creation."),
                        payload: Option::None,
                    };
                }
            }

            // append to custom_url
            p.custom_url = format!("{}/{}", n, o);
        }

        // make sure paste does not exist
        let existing: DefaultReturn<Option<FullPaste<PasteMetadata, String>>> =
            self.get_paste_by_url(p.custom_url.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste already exists!"),
                payload: Option::None,
            };
        }

        // create paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Pastes\" VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Pastes\" VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        };

        let c = &self.db.client;
        let p: &mut Paste<String> = &mut props.clone();
        p.id = utility::random_id();

        let edit_password = &p.edit_password;
        let edit_password_hash = utility::hash(edit_password.to_string());

        let edit_date = &p.edit_date;
        let pub_date = &p.pub_date;

        let res = sqlx::query(query)
            .bind::<&String>(&p.custom_url)
            .bind::<&String>(&p.id)
            .bind::<&String>(&p.group_name)
            .bind::<&String>(&edit_password_hash)
            .bind::<&String>(&pub_date.to_string())
            .bind::<&String>(&edit_date.to_string())
            .bind::<&String>(&p.content)
            .bind::<&String>(&p.content_html)
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // update cache
        if as_user.is_some() {
            self.cachedb
                .remove_starting_with(format!("pastes-by-owner:{}*", as_user.unwrap()))
                .await;
        }

        // return
        let pass = &p.edit_password;
        return DefaultReturn {
            success: true,
            message: pass.to_string(),
            payload: Option::Some(p.to_owned()),
        };
    }

    /// Edit an existing [`Paste`] given its `custom_url`
    pub async fn edit_paste_by_url(
        &self,
        url: String,
        content: String,
        edit_password: String,
        new_url: Option<String>,
        new_edit_password: Option<String>,
        edit_as: Option<String>, // username of account that is editing this paste
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // (parse metadata from existing)
        let existing_metadata = &existing.payload.as_ref().unwrap().paste.metadata;

        // verify password
        // if password hash doesn't match AND edit_as is none OR edit_as != existing_metadata's owner value
        let paste = &existing.payload.clone().unwrap().paste;

        let skip_password_check = edit_as.is_some() && edit_as.unwrap() == existing_metadata.owner;

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // ...
        let edit_password_hash = if new_edit_password.is_some() {
            utility::hash(new_edit_password.unwrap())
        } else {
            // get old password
            let edit_password = &paste.edit_password;
            edit_password.to_owned()
        };

        let custom_url = if new_url.is_some() {
            new_url.as_ref().unwrap()
        } else {
            // get old custom url
            &paste.custom_url
        };

        // if we're changing url, make sure this paste doesn't already exist
        if new_url.is_some() {
            let existing = &self.get_paste_by_url(new_url.clone().unwrap()).await;
            if existing.success {
                return DefaultReturn {
                    success: false,
                    message: String::from("A paste with this URL already exists!"),
                    payload: Option::None,
                };
            }
        }

        // update paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"content\" = ?, \"content_html\" = ?, \"edit_password\" = ?, \"custom_url\" = ?, \"edit_date\" = ? WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"content\", \"content_html\", \"edit_password\", \"custom_url\", \"edit_date\") = ($1, $2, $3, $4, $5) WHERE \"custom_url\" = $6"
        };

        let content_html = &crate::markdown::render::parse_markdown(&content);
        let edit_date = &utility::unix_epoch_timestamp().to_string();

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&content)
            .bind::<&String>(content_html)
            .bind::<&String>(&edit_password_hash)
            .bind::<&String>(&custom_url)
            .bind::<&String>(edit_date) // update edit_date
            .bind::<&String>(&url)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.cachedb.get(format!("paste:{}", url)).await;

        if existing_in_cache.is_some() {
            let mut paste =
                serde_json::from_str::<Paste<PasteMetadata>>(&existing_in_cache.unwrap()).unwrap();

            paste.content = content; // update content
            paste.content_html = content_html.to_string(); // update content_html
            paste.edit_password = edit_password_hash; // update edit_password
            paste.edit_date = edit_date.parse::<u128>().unwrap(); // update edit_date
            paste.custom_url = custom_url.to_string(); // update custom_url

            // update cache
            self.cachedb
                .update(
                    format!("paste:{}", url),
                    serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste updated!"),
            payload: Option::Some(custom_url.to_string()),
        };
    }

    /// Update a [`Paste`]'s metadata by its `custom_url`
    pub async fn edit_paste_metadata_by_url(
        &self,
        url: String,
        metadata: PasteMetadata,
        edit_password: String,
        edit_as: Option<String>, // username of account that is editing this paste
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // (parse metadata from existing)
        let existing_metadata = &existing.payload.as_ref().unwrap().paste.metadata;

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        // verify password
        // if password hash doesn't match AND edit_as is none OR edit_as != existing_metadata's owner value
        let paste = &existing.payload.clone().unwrap().paste;

        // ...skip password check IF the user is the paste owner!
        let skip_password_check = (edit_as.is_some()
            && edit_as.unwrap() == existing_metadata.owner)
            // OR if the user has the "ManagePastes" permission
            | (ua.as_ref().is_some()
                && ua.as_ref().unwrap().is_some()
                && ua.unwrap().unwrap().level.permissions.contains(&String::from("ManagePastes")));

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // update paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Pastes\" SET \"metadata\" = ? WHERE \"custom_url\" = ?"
        } else {
            "UPDATE \"Pastes\" SET (\"metadata\") = ($1) WHERE \"custom_url\" = $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .bind::<&String>(&url)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.cachedb.get(format!("paste:{}", url)).await;

        if existing_in_cache.is_some() {
            let mut paste =
                serde_json::from_str::<Paste<PasteMetadata>>(&existing_in_cache.unwrap()).unwrap();
            paste.metadata = metadata; // update metadata

            // update cache
            self.cachedb
                .update(
                    format!("paste:{}", url),
                    serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste updated!"),
            payload: Option::Some(url),
        };
    }

    /// Count a view to a [`Paste`] given its `custom_url`
    ///
    /// # Arguments:
    /// * `view_as` - The username of the account that viewed the paste
    pub async fn add_view_to_url(
        &self,
        url: &String,
        view_as: &String, // username of account that is viewing this paste
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // check for existing view log
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'view_paste' AND \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("{}::{}", &url, &view_as))
            .fetch_one(c)
            .await;

        if res.is_err() {
            let err = res.err().unwrap();
            let err_message = err.to_string();

            // count view if message says no rows were returned
            if err_message.starts_with("no rows returned") {
                self.create_log(
                    String::from("view_paste"),
                    format!("{}::{}", &url, &view_as),
                )
                .await;

                // update cache
                let existing_in_cache = self.cachedb.get(format!("paste:{}", url)).await;

                if existing_in_cache.is_some() {
                    let mut paste =
                        serde_json::from_str::<Paste<PasteMetadata>>(&existing_in_cache.unwrap())
                            .unwrap();
                    paste.views += 1;

                    // update cache
                    self.cachedb
                        .update(
                            format!("paste:{}", url),
                            serde_json::to_string::<Paste<PasteMetadata>>(&paste).unwrap(),
                        )
                        .await;
                }

                // return
                return DefaultReturn {
                    success: true,
                    message: String::from("View counted!"),
                    payload: Option::Some(url.to_string()),
                };
            }

            // default error return
            return DefaultReturn {
                success: false,
                message: String::from("Failed to check for existing view!"),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("View counted!"),
            payload: Option::Some(url.to_string()),
        };
    }

    /// Delete a [`Paste`] given its `custom_url` and `edit_password`
    pub async fn delete_paste_by_url(
        &self,
        url: String,
        edit_password: String,
        delete_as: Option<String>,
    ) -> DefaultReturn<Option<String>> {
        // make sure paste exists
        let existing = &self.get_paste_by_url(url.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Paste does not exist!"),
                payload: Option::None,
            };
        }

        // (parse metadata from existing)
        let existing_metadata = &existing.payload.as_ref().unwrap().paste.metadata;

        // get edit_as user account
        let ua = if delete_as.is_some() {
            Option::Some(
                self.get_user_by_username(delete_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        // verify password
        let paste = &existing.payload.clone().unwrap().paste;

        // ...skip password check IF the user is the paste owner!
        let skip_password_check = (delete_as.is_some()
                && delete_as.unwrap() == existing_metadata.owner)
                // OR if the user has the "ManagePastes" permission
                | (ua.as_ref().is_some()
                    && ua.as_ref().unwrap().is_some()
                    && ua.unwrap().unwrap().level.permissions.contains(&String::from("ManagePastes")));

        if !skip_password_check && utility::hash(edit_password) != paste.edit_password {
            return DefaultReturn {
                success: false,
                message: String::from("Password invalid"),
                payload: Option::None,
            };
        }

        // delete paste
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = ?"
        } else {
            "DELETE FROM \"Pastes\" WHERE \"custom_url\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&url).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // delete paste views
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("{}::%", &url))
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to delete paste"),
                payload: Option::None,
            };
        }

        // update cache
        self.cachedb.remove(format!("paste:{}", url)).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste deleted!"),
            payload: Option::Some(url),
        };
    }

    // groups

    // GET
    /// Get a [`Group`] by its name
    ///
    /// # Arguments:
    /// * `url` - group name
    pub async fn get_group_by_name(&self, url: String) -> DefaultReturn<Option<Group<String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Groups\" WHERE \"name\" = ?"
        } else {
            "SELECT * FROM \"Groups\" WHERE \"name\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&url).fetch_one(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Group does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Group exists"),
            payload: Option::Some(Group {
                name: row.get("name"),
                submit_password: row.get("submit_password"),
                metadata: row.get("metadata"),
            }),
        };
    }

    // SET
    /// Create a new [`Group`] by its name
    ///
    /// # Arguments:
    /// * `props` - [`Group<GroupMetadata>`](Group)
    pub async fn create_group(&self, props: Group<GroupMetadata>) -> DefaultReturn<Option<String>> {
        let p: &Group<GroupMetadata> = &props; // borrowed props

        // make sure group does not exist
        let existing: DefaultReturn<Option<Group<String>>> =
            self.get_group_by_name(p.name.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Group already exists!"),
                payload: Option::None,
            };
        }

        // create group
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Groups\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Groups\" VALUES ($1, $2, $3)"
        };

        let c = &self.db.client;
        let p: &mut Group<GroupMetadata> = &mut props.clone();

        p.submit_password = utility::hash(p.submit_password.clone());
        let res = sqlx::query(query)
            .bind::<&String>(&p.name)
            .bind::<&String>(&p.submit_password)
            .bind::<&String>(&serde_json::to_string(&p.metadata).unwrap())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Paste created"),
            payload: Option::Some(p.name.to_string()),
        };
    }

    // boards

    // GET
    /// Get a [`Board`] by its name
    ///
    /// # Arguments:
    /// * `url` - board name
    pub async fn get_board_by_name(&self, url: String) -> DefaultReturn<Option<Board<String>>> {
        // check in cache
        let cached = self
            .cachedb
            .get(format!("board:{}", url.to_lowercase()))
            .await;

        if cached.is_some() {
            // ...
            let board = serde_json::from_str::<Board<String>>(cached.unwrap().as_str()).unwrap();

            // return
            return DefaultReturn {
                success: true,
                message: String::from("Board exists (cache)"),
                payload: Option::Some(board),
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"name\" = ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"name\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&url.to_lowercase())
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // store in cache
        let board = Board {
            name: row.get("name").unwrap().to_string(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: row.get("metadata").unwrap().to_string(),
        };

        self.cachedb
            .set(
                format!("board:{}", url.to_lowercase()),
                serde_json::to_string::<Board<String>>(&board).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Board exists (new)"),
            payload: Option::Some(board),
        };
    }

    /// Get a [`UserMailStreamIdentifier`] [`Board`] by its users
    ///
    /// # Arguments:
    /// * `props` - [`UserMailStreamIdentifier`]
    #[allow(dead_code)]
    pub async fn get_mail_stream_by_users(
        &self,
        props: UserMailStreamIdentifier,
    ) -> DefaultReturn<Option<Board<String>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!(
                "%\"about\":\"{}\"%",
                if &self.db._type == "mysql" {
                    serde_json::to_string::<UserMailStreamIdentifier>(&props)
                        .unwrap()
                        .replace("\"", "\\\\\"")
                } else {
                    serde_json::to_string::<UserMailStreamIdentifier>(&props)
                        .unwrap()
                        .replace("\"", "\\\"")
                }
            ))
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Board exists"),
            payload: Option::Some(Board {
                name: row.get("name").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                metadata: row.get("metadata").unwrap().to_string(),
            }),
        };
    }

    /// Get all [`UserMailStreamIdentifier`] [`Board`] by a single participating user
    ///
    /// # Arguments:
    /// * `user` - username of the user
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_user_mail_streams(
        &self,
        user: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Vec<BoardIdentifier>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE ? OR \"metadata\" LIKE ? ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE $1 OR \"metadata\" LIKE $2 ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $3"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<String>(if &self.db._type == "mysql" {
                format!("%\\\\\"user1\\\\\":\\\\\"{}\\\\\"%", user)
            } else {
                format!("%\\\"user1\\\":\\\"{}\\\"%", user)
            })
            .bind::<String>(if &self.db._type == "mysql" {
                format!("%\\\\\"user2\\\\\":\\\\\"{}\\\\\"%", user)
            } else {
                format!("%\\\"user2\\\":\\\"{}\\\"%", user)
            })
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Boards do not exist"),
                payload: Vec::new(),
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<BoardIdentifier> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;

            let metadata =
                serde_json::from_str::<BoardMetadata>(row.get("metadata").unwrap()).unwrap();

            let mailstream =
                serde_json::from_str::<UserMailStreamIdentifier>(&metadata.about.unwrap()).unwrap();

            output.push(BoardIdentifier {
                name: row.get("name").unwrap().to_string(),
                // we're going to use tags to store the name of the other user
                tags: if user == mailstream.user1 {
                    mailstream.user2
                } else {
                    mailstream.user1
                },
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Boards exists"),
            payload: output,
        };
    }

    /// Get public [`Board`]s
    ///
    /// # Arguments:
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_boards(
        &self,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<BoardIdentifier>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" NOT LIKE '%\"is_private\":\"yes\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" NOT LIKE '%\"is_private\":\"yes\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch boards"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<BoardIdentifier> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(BoardIdentifier {
                name: row.get("name").unwrap().to_string(),
                tags: String::new(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Board exists"),
            payload: Option::Some(output),
        };
    }

    /// Get public [`Board`]s by their tags
    ///
    /// # Arguments:
    /// * `tags` - space separated list of tags, tags should use "_" instead of a space in their names, all tags should start with "+"
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_boards_by_tags(
        &self,
        mut tags: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<BoardIdentifier>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE ? AND \"metadata\" NOT LIKE '%\"is_private\":\"yes\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE $1 AND \"metadata\" NOT LIKE '%\"is_private\":\"yes\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        // check tags (reason below)
        for tag in tags.clone().split(" ") {
            if tag.starts_with("+") | (tag.len() == 0) {
                continue;
            }

            tags = tags.replacen(tag, &format!("+{}", tag), 1);
        }

        // ...
        let c = &self.db.client;
        let res = sqlx::query(query)
            // this allows us to search within ANY field of the metadata ...
            // ideally we would use MySQL JSON_VALUE here, but SQLite and PostgreSQL don't support this
            // instead we'll just require that tags start with "+"
            .bind::<&String>(&format!("%{tags}%"))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch boards"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<BoardIdentifier> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            let metadata =
                serde_json::from_str::<BoardMetadata>(&row.get("metadata").unwrap().to_string())
                    .unwrap();

            output.push(BoardIdentifier {
                name: row.get("name").unwrap().to_string(),
                tags: if metadata.tags.is_some() {
                    metadata.tags.unwrap()
                } else {
                    String::new()
                },
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Board exists"),
            payload: Option::Some(output),
        };
    }

    /// Get all posts in a [`Board`] by its name
    ///
    /// # Arguments:
    /// * `url` - board name
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_board_posts(
        &self,
        url: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        // make sure board exists
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_board_by_name(url.to_owned().to_lowercase()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"content\" NOT LIKE '%\"reply\":\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"content\" NOT LIKE '%\"reply\":\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"board\":\"{}\"%", url))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        let mut true_output: Vec<Log> = Vec::new();
        for mut post in output {
            let mut parsed = serde_json::from_str::<BoardPostLog>(&post.content).unwrap();

            // get replies
            let replies = &self.get_post_replies_limited(post.clone().id, false).await;

            if replies.payload.is_some() {
                parsed.replies = Option::Some(replies.payload.as_ref().unwrap().len());

                // update
                post.content = serde_json::to_string::<BoardPostLog>(&parsed).unwrap();
                true_output.push(post);

                continue;
            }

            continue;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts"),
            payload: Option::Some(true_output),
        };
    }

    /// Get all posts in a [`Board`] by its name and their tags
    ///
    /// # Arguments:
    /// * `url` - board name
    /// * `tags` - space separated list of tags, tags should use "_" instead of a space in their names, all tags should start with "+"
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_board_posts_by_tag(
        &self,
        url: String,
        mut tags: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        // make sure board exists
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_board_by_name(url.to_owned().to_lowercase()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"content\" LIKE ? AND \"content\" NOT LIKE '%\"reply\":\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"content\" LIKE $2 AND \"content\" NOT LIKE '%\"reply\":\"%' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $3"
        };

        // check tags
        for tag in tags.clone().split(" ") {
            if tag.starts_with("+") | (tag.len() == 0) {
                continue;
            }

            tags = tags.replacen(tag, &format!("+{}", tag), 1);
        }

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"board\":\"{}\"%", url))
            .bind::<&String>(&format!("%{tags}%"))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        let mut true_output: Vec<Log> = Vec::new();
        for mut post in output {
            let mut parsed = serde_json::from_str::<BoardPostLog>(&post.content).unwrap();

            // get replies
            let replies = &self.get_post_replies_limited(post.clone().id, false).await;

            if replies.payload.is_some() {
                parsed.replies = Option::Some(replies.payload.as_ref().unwrap().len());

                // update
                post.content = serde_json::to_string::<BoardPostLog>(&parsed).unwrap();
                true_output.push(post);

                continue;
            }

            continue;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts"),
            payload: Option::Some(true_output),
        };
    }

    /// Get all pinned posts in a [`Board`] by its name
    ///
    /// # Arguments:
    /// * `url` - board name
    pub async fn get_pinned_board_posts(&self, url: String) -> DefaultReturn<Option<Vec<Log>>> {
        // make sure board exists
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_board_by_name(url.to_owned().to_lowercase()).await;

        if existing.success == false {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"content\" NOT LIKE '%\"reply\":\"%' AND \"content\" LIKE '%\"pinned\":true%' ORDER BY \"timestamp\" DESC LIMIT 50"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"content\" NOT LIKE '%\"reply\":\"%' AND \"content\" LIKE '%\"pinned\":true%' ORDER BY \"timestamp\" DESC LIMIT 50"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"board\":\"{}\"%", url))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts (pinned)"),
            payload: Option::Some(output),
        };
    }

    /// Get all posts in a [`Board`] by its name that are replying to another [`BoardPostLog`]
    ///
    /// # Arguments:
    /// * `id` - post id
    /// * `run_existing_check` - if we should check that the log exists first
    pub async fn get_post_replies(
        &self,
        id: String,
        run_existing_check: bool,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        // make sure message exists
        if run_existing_check != false {
            let existing: DefaultReturn<Option<Log>> = self.get_log_by_id(id.to_owned()).await;

            if existing.success == false {
                return DefaultReturn {
                    success: false,
                    message: String::from("Post does not exist"),
                    payload: Option::None,
                };
            }
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"reply\":\"{}\"%", id))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        let mut true_output: Vec<Log> = Vec::new();
        for mut post in output {
            let mut parsed = serde_json::from_str::<BoardPostLog>(&post.content).unwrap();

            // get replies
            let replies = &self.get_post_replies_limited(post.clone().id, false).await;

            if replies.payload.is_some() {
                parsed.replies = Option::Some(replies.payload.as_ref().unwrap().len());

                // update
                post.content = serde_json::to_string::<BoardPostLog>(&parsed).unwrap();
                true_output.push(post);

                continue;
            }

            continue;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts (replies)"),
            payload: Option::Some(true_output),
        };
    }

    /// Get all posts in a [`Board`] by its name that are replying to another [`BoardPostLog`] (limited form)
    ///
    /// - only includes post id
    ///
    /// # Arguments:
    /// * `id` - post id
    /// * `run_existing_check` - if we should check that the log exists first
    pub async fn get_post_replies_limited(
        &self,
        id: String,
        run_existing_check: bool,
    ) -> DefaultReturn<Option<Vec<LogIdentifier>>> {
        // make sure message exists
        if run_existing_check != false {
            let existing: DefaultReturn<Option<Log>> = self.get_log_by_id(id.to_owned()).await;

            if existing.success == false {
                return DefaultReturn {
                    success: false,
                    message: String::from("Post does not exist"),
                    payload: Option::None,
                };
            }
        }

        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"content\" LIKE ?"
        } else {
            "SELECT \"ID\" FROM \"Logs\" WHERE \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"reply\":\"{}\"%", id))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<LogIdentifier> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(LogIdentifier {
                id: row.get("id").unwrap_or(&String::new()).to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts (limited)"),
            payload: Option::Some(output),
        };
    }

    /// Get all [boards](Board) owned by a specific user
    ///
    /// # Arguments:
    /// * `owner` - `String` of the owner's `username`
    pub async fn get_boards_by_owner(
        &self,
        owner: String,
    ) -> DefaultReturn<Option<Vec<BoardIdentifier>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE ?"
        } else {
            "SELECT * FROM \"Boards\" WHERE \"metadata\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"owner\":\"{}\"%", &owner))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // build res
        let mut full_res: Vec<BoardIdentifier> = Vec::new();

        for row in res.unwrap() {
            let row = self.textify_row(row).data;
            full_res.push(BoardIdentifier {
                name: row.get("name").unwrap().to_string(),
                tags: String::new(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: owner,
            payload: Option::Some(full_res),
        };
    }

    /// Get most recent posts from all [`Boards`](Board)
    ///
    /// # Arguments:
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn fetch_most_recent_posts(
        &self,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        // ...
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'board_post' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'board_post' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Successfully fetched posts"),
            payload: Option::Some(output),
        };
    }

    /// Get the amount of posts a user has created in all boards
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn get_user_posts_count(&self, user: String) -> DefaultReturn<usize> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'board_post'"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'board_post'"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"author\":\"{user}\"%"))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch posts"),
                payload: 0,
            };
        }

        // ...
        let rows = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Posts exist"),
            payload: rows.len(),
        };
    }

    // SET
    /// Create a new [`Board`] given various properties
    ///
    /// # Arguments:
    /// * `props` - [`Board<String>`](Board)
    /// * `as_user` - The ID of the user creating the board
    pub async fn create_board(
        &self,
        props: &mut Board<String>,
        as_user: Option<String>, // id of board owner
    ) -> DefaultReturn<Option<Board<String>>> {
        let p: &mut Board<String> = props; // borrowed props

        // create default metadata
        let metadata: BoardMetadata = BoardMetadata {
            owner: as_user.clone().unwrap(),
            is_private: String::from("no"), // values other than no/yes will show the view password prompt
            allow_anonymous: Option::Some(String::from("yes")),
            allow_open_posting: Option::Some(String::from("yes")),
            topic_required: Option::Some(String::from("no")),
            about: Option::None,
            tags: Option::None,
        };

        // check values

        // (check length)
        if (p.name.len() < 2) | (p.name.len() > 250) {
            return DefaultReturn {
                success: false,
                message: String::from("Name is invalid"),
                payload: Option::None,
            };
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&p.name).iter().len() < 1 {
            return DefaultReturn {
                success: false,
                message: String::from("Name is invalid"),
                payload: Option::None,
            };
        }

        // make sure board does not exist
        let existing: DefaultReturn<Option<Board<String>>> = self
            .get_board_by_name(p.name.to_owned().to_lowercase())
            .await;

        if existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board already exists!"),
                payload: Option::None,
            };
        }

        // create board
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Boards\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Boards\" VALUES ($1, $2, $3)"
        };

        let c = &self.db.client;
        let p: &mut Board<String> = &mut props.clone();
        p.timestamp = utility::unix_epoch_timestamp();

        let res = sqlx::query(query)
            .bind::<&String>(&p.name)
            .bind::<&String>(&p.timestamp.to_string())
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Created board"),
            payload: Option::Some(p.to_owned()),
        };
    }

    /// Create a new [`Board`] given various [properties](UserMailStreamIdentifier)
    ///
    /// # Arguments:
    /// * `props` - [`UserMailStreamIdentifier`]
    pub async fn create_mail_stream(
        &self,
        props: &mut UserMailStreamIdentifier,
    ) -> DefaultReturn<Option<Board<String>>> {
        let p: &mut UserMailStreamIdentifier = props; // borrowed props

        // create default metadata
        let metadata: BoardMetadata = BoardMetadata {
            owner: String::new(),
            is_private: String::from("yes"),
            allow_anonymous: Option::Some(String::from("no")),
            allow_open_posting: Option::Some(String::from("yes")),
            topic_required: Option::Some(String::from("no")), // change to "yes" for a more inbox-like thing
            about: Option::Some(
                serde_json::to_string::<UserMailStreamIdentifier>(&p.clone()).unwrap(),
            ),
            tags: Option::None,
        };

        // check values

        // make sure board does not exist
        let existing: DefaultReturn<Option<Board<String>>> =
            self.get_mail_stream_by_users(props.to_owned()).await;

        if existing.success {
            return DefaultReturn {
                success: true, // return true so client still redirects
                message: String::from("Board already exists!"),
                payload: existing.payload, // return existing board if it does
            };
        }

        // make sure board does not exist
        let existing: DefaultReturn<Option<Board<String>>> = self
            .get_mail_stream_by_users(UserMailStreamIdentifier {
                _is_user_mail_stream: true,
                user1: props.user2.clone(),
                user2: props.user1.clone(),
            })
            .await;

        if existing.success {
            return DefaultReturn {
                success: true, // return true so client still redirects
                message: String::from("Board already exists!"),
                payload: existing.payload, // return existing board if it does
            };
        }

        // make sure other user exists (user1 should be the current user)
        let existing: DefaultReturn<Option<FullUser<String>>> =
            self.get_user_by_username(props.user2.clone()).await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Other user is invalid"),
                payload: Option::None,
            };
        }

        // check user permissions
        let user2_metadata =
            serde_json::from_str::<UserMetadata>(&existing.payload.unwrap().user.metadata).unwrap();

        if user2_metadata.allow_mail.is_some()
            && user2_metadata.allow_mail.as_ref().unwrap() == "no"
        {
            return DefaultReturn {
                success: false,
                message: String::from("User is not accepting mail"),
                payload: Option::None,
            };
        }

        // create board
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "INSERT INTO \"Boards\" VALUES (?, ?, ?)"
        } else {
            "INSERT INTO \"Boards\" VALUES ($1, $2, $3)"
        };

        let c = &self.db.client;
        let p: &mut Board<String> = &mut Board {
            name: format!("inbox-{}", utility::random_id()),
            timestamp: utility::unix_epoch_timestamp(),
            metadata: String::new(),
        };

        let res = sqlx::query(query)
            .bind::<&String>(&p.name)
            .bind::<&String>(&p.timestamp.to_string())
            .bind::<&String>(&serde_json::to_string(&metadata).unwrap())
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: res.err().unwrap().to_string(),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Created board"),
            payload: Option::Some(p.to_owned()),
        };
    }

    /// Create a new post in a given [`Board`]
    ///
    /// # Arguments:
    /// * `props` - [`BoardPostLog`]
    /// * `as_user` - The ID of the user creating the post
    pub async fn create_board_post(
        &self,
        props: &mut BoardPostLog,
        as_user: Option<String>, // username of user posting
        as_role: Option<String>, // role of user posting
    ) -> DefaultReturn<Option<String>> {
        let p: &mut BoardPostLog = props; // borrowed props

        // check values

        // (check length)
        if (p.content.len() < 2) | (p.content.len() > 1_000) {
            return DefaultReturn {
                success: false,
                message: String::from("Content is invalid"),
                payload: Option::None,
            };
        }

        // make sure this board exists
        let existing: DefaultReturn<Option<Board<String>>> = self
            .get_board_by_name(p.board.to_owned().to_lowercase())
            .await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist!"),
                payload: Option::None,
            };
        }

        let board =
            serde_json::from_str::<BoardMetadata>(&existing.payload.unwrap().metadata).unwrap();

        // check board "topic_required" setting
        // if it is set to "yes", make sure we provided a topic AND this is not a reply (replies to not count)
        if board.topic_required.is_some()
            && board.topic_required.unwrap() == "yes"
            && p.reply.is_none()
            && p.topic.is_none()
        {
            return DefaultReturn {
                success: false,
                message: String::from("This board requires a topic to be set before posting"),
                payload: Option::None,
            };
        }

        // check board "allow_anonymous" setting
        if board.allow_anonymous.is_some()
            && board.allow_anonymous.unwrap() == "no"
            && as_user.is_none()
        {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // check board "allow_open_posting" setting
        if board.allow_open_posting.is_some()
            && board.allow_open_posting.unwrap() == String::from("no")
        {
            let can_post = as_user.is_some()
                && ((as_user.as_ref().unwrap() == &board.owner) | (as_role.unwrap() == "staff"));

            if can_post == false {
                return DefaultReturn {
                    success: false,
                    message: String::from("You do not have permission to do this"),
                    payload: Option::None,
                };
            }
        }

        // create post
        let post = BoardPostLog {
            author: if as_user.is_some() {
                as_user.unwrap()
            } else {
                String::from("Anonymous")
            },
            content: p.content.clone(),
            content_html: crate::markdown::render::parse_markdown(&p.content),
            topic: p.topic.clone(),
            board: p.board.clone().to_lowercase(),
            is_hidden: false,
            reply: p.reply.clone(),
            pinned: Option::Some(false),
            replies: Option::None,
            tags: Option::None,
        };

        // return
        self.create_log(
            String::from("board_post"),
            serde_json::to_string::<BoardPostLog>(&post).unwrap(),
        )
        .await
    }

    /// Update a [`Board`]'s metadata by its `custom_url`
    pub async fn edit_board_metadata_by_name(
        &self,
        name: String,
        metadata: BoardMetadata,
        edit_as: Option<String>, // username of account that is editing this board
    ) -> DefaultReturn<Option<String>> {
        // make sure board exists
        let existing = &self.get_board_by_name(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist!"),
                payload: Option::None,
            };
        }

        // get edit_as user account
        let ua = if edit_as.is_some() {
            Option::Some(
                self.get_user_by_username(edit_as.clone().unwrap())
                    .await
                    .payload,
            )
        } else {
            Option::None
        };

        if ua.is_none() {
            return DefaultReturn {
                success: false,
                message: String::from("An account is required to do this"),
                payload: Option::None,
            };
        }

        // make sure we can do this
        let user = ua.unwrap().unwrap();
        let can_edit: bool =
            // using "metadata" here likely prohibits us from changing board owner
            (user.user.username == metadata.owner) | (user.level.permissions.contains(&String::from("ManageBoards")));

        if can_edit == false {
            return DefaultReturn {
                success: false,
                message: String::from(
                    "You do not have permission to manage this board's contents.",
                ),
                payload: Option::None,
            };
        }

        // make sure we're not trying to convert this to a mail stream
        if metadata.about.is_some()
            && metadata
                .about
                .as_ref()
                .unwrap()
                .contains("\"_is_user_mail_stream\":true")
        {
            return DefaultReturn {
                success: false,
                message: String::from("Cannot convert this board into a user mail stream."),
                payload: Option::None,
            };
        }

        // update board
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "UPDATE \"Boards\" SET \"metadata\" = ? WHERE \"name\" = ?"
        } else {
            "UPDATE \"Boards\" SET (\"metadata\") = ($1) WHERE \"name\" = $2"
        };

        let c = &self.db.client;
        let metadata = serde_json::to_string(&metadata).unwrap();
        let res = sqlx::query(query)
            .bind::<&String>(&metadata)
            .bind::<&String>(&name)
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        let existing_in_cache = self.cachedb.get(format!("board:{}", name)).await;

        if existing_in_cache.is_some() {
            let mut board =
                serde_json::from_str::<Board<String>>(&existing_in_cache.unwrap()).unwrap();
            board.metadata = metadata; // update metadata

            // update cache
            self.cachedb
                .update(
                    format!("board:{}", name),
                    serde_json::to_string::<Board<String>>(&board).unwrap(),
                )
                .await;
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Board updated!"),
            payload: Option::Some(name),
        };
    }

    /// Delete a board given its name
    ///
    /// # Arguments:
    /// * `name` - `String` of the board's `name`
    pub async fn delete_board(&self, name: String) -> DefaultReturn<Option<String>> {
        // make sure log exists
        let existing = &self.get_board_by_name(name.clone()).await;
        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("Board does not exist!"),
                payload: Option::None,
            };
        }

        // delete board
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Boards\" WHERE \"name\" = ?"
        } else {
            "DELETE FROM \"Boards\" WHERE \"name\" = $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query).bind::<&String>(&name).execute(c).await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // delete board messages
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"content\" LIKE $1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"board\":\"{}\"%", name))
            .execute(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from(res.err().unwrap().to_string()),
                payload: Option::None,
            };
        }

        // update cache
        self.cachedb.remove(format!("board:{}", name)).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Board deleted!"),
            payload: Option::Some(name),
        };
    }

    // follows

    // GET
    /// Get a [`UserFollow`] by the username of the user following
    ///
    /// # Arguments:
    /// * `user` - username of user following
    /// * `is_following` - the username of the user that `user` is following
    pub async fn get_follow_by_user(
        &self,
        user: String,
        is_following: String,
    ) -> DefaultReturn<Option<Log>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'follow'"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'follow'"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!(
                "%\"user\":\"{user}\",\"is_following\":\"{is_following}\"%"
            ))
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Follow does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.textify_row(row).data;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Follow exists"),
            payload: Option::Some(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            }),
        };
    }

    /// Get the [`UserFollow`]s that are following the given `user`
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_user_followers(
        &self,
        user: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'follow' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'follow' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"is_following\":\"{user}\"%"))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch followers"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Followers exists"),
            payload: Option::Some(output),
        };
    }

    /// Get the [`UserFollow`]s that the given `user` is following
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    /// * `offset` - optional value representing the SQL fetch offset
    pub async fn get_user_following(
        &self,
        user: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'follow' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'follow' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch following"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Following exists"),
            payload: Option::Some(output),
        };
    }

    /// Get the amount of followers a user has
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn get_user_follow_count(&self, user: String) -> DefaultReturn<usize> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'follow'"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'follow'"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"is_following\":\"{user}\"%"))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch followers"),
                payload: 0,
            };
        }

        // ...
        let rows = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Follow exists"),
            payload: rows.len(),
        };
    }

    /// Get the amount of users a user is following
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn get_user_following_count(&self, user: String) -> DefaultReturn<usize> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'follow'"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'follow'"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch following"),
                payload: 0,
            };
        }

        // ...
        let rows = res.unwrap();

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Follow exists"),
            payload: rows.len(),
        };
    }

    // SET
    /// Toggle the following status of `user` on `is_following` ([`UserFollow`])
    ///
    /// # Arguments:
    /// * `props` - [`UserFollow`]
    pub async fn toggle_user_follow(
        &self,
        props: &mut UserFollow,
    ) -> DefaultReturn<Option<String>> {
        let p: &mut UserFollow = props; // borrowed props

        // users cannot be the same
        if p.user == p.is_following {
            return DefaultReturn {
                success: false,
                message: String::from("Cannot follow yourself!"),
                payload: Option::None,
            };
        }

        // make sure both users exist
        let existing: DefaultReturn<Option<FullUser<String>>> =
            self.get_user_by_username(p.user.to_owned()).await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist!"),
                payload: Option::None,
            };
        }

        // make sure both users exist
        let existing: DefaultReturn<Option<FullUser<String>>> =
            self.get_user_by_username(p.is_following.to_owned()).await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User (2) does not exist!"),
                payload: Option::None,
            };
        }

        // check if follow exists
        let existing: DefaultReturn<Option<Log>> = self
            .get_follow_by_user(p.user.to_owned(), p.is_following.to_owned())
            .await;

        if existing.success {
            // delete log and return
            return self.delete_log(existing.payload.unwrap().id).await;
        }

        // return
        self.create_log(
            String::from("follow"),
            serde_json::to_string::<UserFollow>(&p).unwrap(),
        )
        .await
    }

    // notifications

    // GET
    /// Get the [`Notification`]s that belong to the given `user`
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn get_user_notifications(
        &self,
        user: String,
        offset: Option<i32>,
    ) -> DefaultReturn<Option<Vec<Log>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'notification' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'notification' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch notifications"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Notifications exist"),
            payload: Option::Some(output),
        };
    }

    /// Check if the given `user` has an active notification
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn user_has_notification(&self, user: String) -> DefaultReturn<Option<Vec<Log>>> {
        let query: &str = if (self.db._type == "sqlite") | (self.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'notification' LIMIT 1"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'notification' LIMIT 1"
        };

        let c = &self.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .fetch_all(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("Failed to fetch notification"),
                payload: Option::None,
            };
        }

        // ...
        let rows = res.unwrap();

        if rows.len() == 0 {
            return DefaultReturn {
                success: true,
                message: String::from("No"),
                payload: Option::None,
            };
        }

        // return
        return DefaultReturn {
            success: true,
            message: String::from("Yes"),
            payload: Option::None,
        };
    }

    // SET
    /// Create a new [`Notification`] for a given [`UserState`]
    ///
    /// # Arguments:
    /// * `props` - [`Notification`]
    pub async fn push_user_notification(
        &self,
        props: &mut Notification,
    ) -> DefaultReturn<Option<String>> {
        let p: &mut Notification = props; // borrowed props

        // make sure user exists
        let existing: DefaultReturn<Option<FullUser<String>>> =
            self.get_user_by_username(p.user.to_owned()).await;

        if !existing.success {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist!"),
                payload: Option::None,
            };
        }

        // return
        self.create_log(
            String::from("notification"),
            serde_json::to_string::<Notification>(&p).unwrap(),
        )
        .await
    }
}
