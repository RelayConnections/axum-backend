use serde::{Serialize, Deserialize};
use mongodb::{Client, bson, doc};

// Think of a Trait as an Interface
trait CommonModel {
    pub async fn sync() -> Result<bool>; // Get an update from DB -> server
    pub async fn save() -> Result<bool>; // Send an update from server -> DB
    pub async fn remove() -> Result<bool>; // Remove the object from the DB
    pub async fn add() -> Result<bool>; // Add the object to the DB
}

// Structs are just objects containing data
#[derive(Serialize, Deserialize)]
struct UserModel {
    #[serde(rename = "_id")]
    id: bson::oid::ObjectId,
    username: String,
    password: String,
    firstname: String,
    lastname: String,
    isAdmin: bool,
    profilePicture: String,
    coverPicture: String,
    about: String,
    livesIn: String,
    worksAt: String,
    relationship: String,
    country: String,
    followers: Option<Vec<String>>,
    following: Option<Vec<String>>,
}

// Implementation (functions) of CommonModel for UserModel
impl CommonModel for UserModel {
    async fn sync(&mut self, client: Client) -> Result<bool> {
        let users = client.database("SocialMedia").collection::<UserModel>("users");
        let doc = doc!({_id: &self.id});
        let user = users.find_one(doc, Option::None).await?;
        self.username = user.username;
        self.password = user.password;
        self.firstname = user.firstname;
        self.lastname = user.lastname;
        self.isAdmin = user.isAdmin;
        self.profilePicture = user.profilePicture;
        self.coverPicture = user.coverPicture;
        self.about = user.about;
        self.livesIn = user.livesIn;
        self.worksAt = user.worksAt;
        self.relationship = user.relationship;
        self.country = user.country;
        self.followers = user.followers;
        self.following = user.following;
        Ok(true)
    }
}

// Implementation (functions) for UserModel
impl UserModel {
    async fn follow(client: Client, id: String) -> Result<bool> {
        
    }
}