use crate::state::Role;

pub fn get_key_for_role<'a>(role: Role) -> &'a str {
    match role {
        Role::DefaultAdmin => "1",
        Role::ClaimIssuer => "2",
        Role::Minter => "3",
        Role::Blacklisted => "4",
    }
}
