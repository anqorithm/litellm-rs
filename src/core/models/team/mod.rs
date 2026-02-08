//! Team models for the Gateway
//!
//! This module defines team-related data structures.

mod billing;
mod entity;
mod invitation;
mod member;
mod settings;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use billing::{
    BillingAddress, BillingPlan, BillingStatus, PaymentMethod, PaymentMethodType, TeamBilling,
};
pub use entity::{Team, TeamStatus, TeamVisibility};
pub use invitation::{InvitationStatus, TeamInvitation};
pub use member::{MemberStatus, TeamMember, TeamRole};
pub use settings::{
    ApiAccessSettings, ChannelType, NotificationChannel, PasswordPolicy, TeamNotificationSettings,
    TeamSecuritySettings, TeamSettings,
};
