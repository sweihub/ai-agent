use uuid::Uuid;

pub fn random_uuid() -> String {
    Uuid::new_v4().to_string()
}
