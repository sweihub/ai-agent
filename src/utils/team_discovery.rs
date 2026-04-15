use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub status: MemberStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberStatus {
    Online,
    Away,
    Busy,
    Offline,
}

pub struct TeamDiscovery {
    members: HashMap<String, TeamMember>,
}

impl TeamDiscovery {
    pub fn new() -> Self {
        Self {
            members: HashMap::new(),
        }
    }

    pub fn register(&mut self, member: TeamMember) {
        self.members.insert(member.id.clone(), member);
    }

    pub fn unregister(&mut self, id: &str) {
        self.members.remove(id);
    }

    pub fn get_member(&self, id: &str) -> Option<&TeamMember> {
        self.members.get(id)
    }

    pub fn list_online(&self) -> Vec<&TeamMember> {
        self.members
            .values()
            .filter(|m| m.status == MemberStatus::Online)
            .collect()
    }

    pub fn update_status(&mut self, id: &str, status: MemberStatus) {
        if let Some(member) = self.members.get_mut(id) {
            member.status = status;
        }
    }
}

impl Default for TeamDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_discovery() {
        let mut discovery = TeamDiscovery::new();

        discovery.register(TeamMember {
            id: "1".to_string(),
            name: "Alice".to_string(),
            status: MemberStatus::Online,
        });

        let member = discovery.get_member("1").unwrap();
        assert_eq!(member.name, "Alice");
    }
}
