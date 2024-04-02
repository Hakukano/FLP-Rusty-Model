#[macro_use]
extern crate flp_rusty_model;

#[macro_use]
extern crate serial_test;

#[derive(Clone, RustyModel)]
#[rusty_model(service = "user_service", has_many = ["post"])]
struct User {
    id: u64,
    #[rusty_model(findable)]
    name: String,
    #[rusty_model(findable)]
    age: u8,
}

#[derive(Clone, RustyModel)]
#[rusty_model(service = "post_service", belongs_to = ["user"])]
struct Post {
    id: String,
    #[rusty_model(findable)]
    title: String,
    #[rusty_model(findable)]
    user_id: u64,
}

mod user_service {
    use std::collections::HashMap;

    use once_cell::sync::Lazy;
    use parking_lot::RwLock;

    use super::User;

    pub type SaveError = String;
    pub type DestroyError = String;

    pub fn all() -> Vec<User> {
        INSTANCE.read().values().cloned().collect()
    }

    pub fn find(id: &u64) -> Option<User> {
        INSTANCE.read().get(id).cloned()
    }

    pub fn save(user: &User) -> Result<(), SaveError> {
        INSTANCE.write().insert(user.id, user.clone());
        Ok(())
    }

    pub fn destroy(id: &u64) -> Result<(), DestroyError> {
        INSTANCE.write().remove(id);
        Ok(())
    }

    pub fn clear() {
        *INSTANCE.write() = HashMap::new();
    }

    static INSTANCE: Lazy<RwLock<HashMap<u64, User>>> = Lazy::new(RwLock::default);
}

mod post_service {
    use std::collections::HashMap;

    use once_cell::sync::Lazy;
    use parking_lot::RwLock;

    use super::Post;

    pub type SaveError = String;
    pub type DestroyError = String;

    pub fn all() -> Vec<Post> {
        INSTANCE.read().values().cloned().collect()
    }

    pub fn find(id: &str) -> Option<Post> {
        INSTANCE.read().get(id).cloned()
    }

    pub fn save(post: &Post) -> Result<(), SaveError> {
        INSTANCE.write().insert(post.id.clone(), post.clone());
        Ok(())
    }

    pub fn destroy(id: &str) -> Result<(), DestroyError> {
        INSTANCE.write().remove(id);
        Ok(())
    }

    pub fn clear() {
        *INSTANCE.write() = HashMap::new();
    }
    static INSTANCE: Lazy<RwLock<HashMap<String, Post>>> = Lazy::new(RwLock::default);
}

fn setup() {
    User {
        id: 1,
        name: "Test User".to_string(),
        age: 20,
    }
    .save()
    .unwrap();
    Post {
        id: "1234abcd".to_string(),
        title: "Test post".to_string(),
        user_id: 1,
    }
    .save()
    .unwrap();
}

fn cleanup() {
    post_service::clear();
    user_service::clear();
}

#[test]
#[serial]
fn find() {
    setup();
    let user = User::find(&1).map(|user| user.name);
    assert_eq!(user, Some("Test User".to_string()));
    cleanup();
}

#[test]
#[serial]
fn all() {
    setup();
    let user = User::all().into_iter().next().map(|user| user.name);
    assert_eq!(user, Some("Test User".to_string()));
    cleanup();
}

#[test]
#[serial]
fn save() {
    setup();
    User {
        id: 2,
        name: "Test User2".to_string(),
        age: 40,
    }
    .save()
    .unwrap();
    let user = User::find(&2).map(|user| user.name);
    assert_eq!(user, Some("Test User2".to_string()));
    cleanup();
}

#[test]
#[serial]
fn destroy() {
    setup();
    let user = User::find(&1).unwrap();
    user.destroy().unwrap();
    assert_eq!(User::all().len(), 0);
    assert_eq!(Post::all().len(), 0);
    cleanup();
}

#[test]
#[serial]
fn find_by() {
    setup();
    let user = User::find_by_age(&20)
        .into_iter()
        .next()
        .map(|user| user.name);
    assert_eq!(user, Some("Test User".to_string()));
    cleanup();
}

#[test]
#[serial]
fn children() {
    setup();
    let user = User::find(&1).unwrap();
    let post = user.post_list().into_iter().next().map(|post| post.title);
    assert_eq!(post, Some("Test post".to_string()));
    cleanup();
}

#[test]
#[serial]
fn parent() {
    setup();
    let post = Post::find(&"1234abcd".to_string()).unwrap();
    let user = post.user().map(|user| user.name);
    assert_eq!(user, Some("Test User".to_string()));
    cleanup();
}
