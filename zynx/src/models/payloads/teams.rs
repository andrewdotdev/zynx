// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

//! Types extracted from <https://discord.com/developers/docs/topics/teams>

use serde::{Deserialize, Serialize};

use super::user::APIUser;
/// See <https://discord.com/developers/docs/topics/teams#data-models-team-object>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APITeam {
    /// A hash of the image of the team's icon
    pub icon: Option<String>,
    /// The unique id of the team
    pub id: String,
    /// The members of the team
    pub members: Vec<APITeamMember>,
    /// The name of the team
    pub name: String,
    /// The user id of the current team owner
    pub owner_user_id: String,
}

/// See <https://discord.com/developers/docs/topics/teams#data-models-team-member-object>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APITeamMember {
    /// The user's membership state on the team
    ///
    /// See <https://discord.com/developers/docs/topics/teams#data-models-membership-state-enum>
    pub membership_state: TeamMemberMembershipState,
    /// Will always be `["*"]`
    ///
    /// **Deprecated:** Use [`APITeamMember::role`](crate::models::payloads::teams::APITeamMember::role) instead.
    pub permissions: Vec<String>,
    /// The id of the parent team of which they are a member
    pub team_id: String,
    /// The avatar, discriminator, id, and username of the user
    ///
    /// See <https://discord.com/developers/docs/resources/user#user-object>
    pub user: APIUser,
    /// The user's role in the team.
    ///
    /// See <https://discord.com/developers/docs/topics/teams#team-member-roles>
    pub role: TeamMemberRole,
}

/// See <https://discord.com/developers/docs/topics/teams#data-models-membership-state-enum>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamMemberMembershipState {
    Invited = 1,
    Accepted,
}

/// See <https://discord.com/developers/docs/topics/teams#team-member-roles-team-member-role-types>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeamMemberRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "developer")]
    Developer,
    #[serde(rename = "read_only")]
    ReadOnly,
}
