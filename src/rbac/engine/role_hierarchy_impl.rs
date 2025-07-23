use super::RoleId;
use std::collections::{HashMap, VecDeque};

/// walk the hierarchy tree and enumerate all the child roles given a role
pub fn enumerate_role(
    role: RoleId,
    role_hierarchy: &HashMap<RoleId, Vec<RoleId>>, // Role -> ChildRole
) -> Vec<RoleId> {
    let mut roles = Vec::new();
    let mut queue = VecDeque::new();
    queue.push_back(role);

    while let Some(role) = queue.pop_front() {
        roles.push(role);
        if let Some(children) = role_hierarchy.get(&role) {
            for child in children {
                queue.push_back(*child);
            }
        }
    }

    roles
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_enumerate_role() {
        let role_hierarchy = [
            (RoleId(1), vec![RoleId(2)]),
            (RoleId(2), vec![RoleId(3), RoleId(4)]),
            (RoleId(4), vec![RoleId(5)]),
            (RoleId(6), vec![]),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            enumerate_role(RoleId(1), &role_hierarchy),
            [RoleId(1), RoleId(2), RoleId(3), RoleId(4), RoleId(5)]
        );
        assert_eq!(
            enumerate_role(RoleId(2), &role_hierarchy),
            [RoleId(2), RoleId(3), RoleId(4), RoleId(5)]
        );
        assert_eq!(enumerate_role(RoleId(3), &role_hierarchy), [RoleId(3)]);
        assert_eq!(
            enumerate_role(RoleId(4), &role_hierarchy),
            [RoleId(4), RoleId(5)]
        );
        assert_eq!(enumerate_role(RoleId(5), &role_hierarchy), [RoleId(5)]);
        assert_eq!(enumerate_role(RoleId(6), &role_hierarchy), [RoleId(6)]);
        assert_eq!(enumerate_role(RoleId(7), &role_hierarchy), [RoleId(7)]);
    }
}
