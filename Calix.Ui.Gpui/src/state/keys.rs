//! The keyboard's registry: every action the user can trigger, and the
//! binding that triggers it.
//!
//! The registry is the single source of truth. The handler asks it which
//! action a key press means; the help overlay is *derived* from it. A key map
//! written by hand in a help panel is a lie the moment a binding changes —
//! this one cannot drift, because there is only one list.
//!
//! The gesture is the launcher's: the help appears when the user holds the
//! modifier alone, and the surface carries no key glyph at all.

use crate::state::session::{NavScope, Place};

/// A binding: a key, and the modifiers it needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: &'static str,
    /// The platform's own modifier: ⌘ on macOS, Ctrl elsewhere.
    pub platform: bool,
    pub alt: bool,
    pub shift: bool,
}

impl KeyBinding {
    pub const fn new(key: &'static str) -> Self {
        Self {
            key,
            platform: false,
            alt: false,
            shift: false,
        }
    }

    pub const fn platform(key: &'static str) -> Self {
        Self {
            key,
            platform: true,
            alt: false,
            shift: false,
        }
    }

    pub const fn alt(key: &'static str) -> Self {
        Self {
            key,
            platform: false,
            alt: true,
            shift: false,
        }
    }

    pub const fn alt_platform(key: &'static str) -> Self {
        Self {
            key,
            platform: true,
            alt: true,
            shift: false,
        }
    }

    /// The binding, written the way it is written on a keycap.
    pub fn label(&self) -> String {
        let mut label = String::new();
        if self.platform {
            label.push_str("⌘");
        }
        if self.alt {
            label.push_str("⌥");
        }
        if self.shift {
            label.push_str("⇧");
        }
        label.push_str(&self.key.to_uppercase());
        label
    }
}

/// An action the user can trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyAction {
    /// The action's id: namespaced, so a surface's actions never collide.
    pub id: &'static str,
    /// What the action does, in the user's words.
    pub label: &'static str,
    /// The group the help overlay files it under.
    pub group: &'static str,
    pub binding: KeyBinding,
}

/// Every action, in the order the help overlay shows them.
pub const ACTIONS: &[KeyAction] = &[
    KeyAction {
        id: "rail.search",
        label: "Search",
        group: "Navigation",
        binding: KeyBinding::platform("1"),
    },
    KeyAction {
        id: "rail.installed",
        label: "Installed",
        group: "Navigation",
        binding: KeyBinding::platform("2"),
    },
    KeyAction {
        id: "rail.updates",
        label: "Updates",
        group: "Navigation",
        binding: KeyBinding::platform("3"),
    },
    KeyAction {
        id: "rail.news",
        label: "News",
        group: "Navigation",
        binding: KeyBinding::platform("4"),
    },
    KeyAction {
        id: "rail.settings",
        label: "Settings",
        group: "Navigation",
        binding: KeyBinding::platform("5"),
    },
    KeyAction {
        id: "scope.previous",
        label: "The previous scope",
        group: "Scope",
        binding: KeyBinding::alt("←"),
    },
    KeyAction {
        id: "scope.next",
        label: "The next scope",
        group: "Scope",
        binding: KeyBinding::alt("→"),
    },
    KeyAction {
        id: "search.focus",
        label: "Focus the search",
        group: "Search",
        binding: KeyBinding::platform("f"),
    },
    KeyAction {
        id: "search.open",
        label: "Focus the search",
        group: "Search",
        binding: KeyBinding::new("/"),
    },
    KeyAction {
        id: "console.toggle",
        label: "The operation log",
        group: "The log",
        binding: KeyBinding::platform("l"),
    },
    KeyAction {
        id: "sidebar.toggle",
        label: "The rail",
        group: "View",
        binding: KeyBinding::alt_platform("c"),
    },
    KeyAction {
        id: "list.next",
        label: "The next package",
        group: "The list",
        binding: KeyBinding::new("↓"),
    },
    KeyAction {
        id: "list.previous",
        label: "The previous package",
        group: "The list",
        binding: KeyBinding::new("↑"),
    },
    KeyAction {
        id: "list.select",
        label: "Select the package",
        group: "The list",
        binding: KeyBinding::new("enter"),
    },
    KeyAction {
        id: "list.clear",
        label: "Clear, then go back",
        group: "The list",
        binding: KeyBinding::new("escape"),
    },
];

/// The action a key press means.
pub fn action_for(key: &str, platform: bool, alt: bool, shift: bool) -> Option<&'static KeyAction> {
    ACTIONS.iter().find(|action| {
        action.binding.key.eq_ignore_ascii_case(key)
            && action.binding.platform == platform
            && action.binding.alt == alt
            && action.binding.shift == shift
    })
}

/// The place a rail action opens, and the scope it opens it on.
///
/// The rail's five entries are bound to ⌘1…⌘5. The action's id names the
/// place, and the scope follows it: the first three entries are the package
/// surface, and each of them has a scope of its own.
pub fn rail_target(id: &str) -> Option<(Place, Option<NavScope>)> {
    match id {
        "rail.search" => Some((Place::Packages, Some(NavScope::Results))),
        "rail.installed" => Some((Place::Packages, Some(NavScope::Installed))),
        "rail.updates" => Some((Place::Packages, Some(NavScope::Updates))),
        "rail.news" => Some((Place::News, None)),
        "rail.settings" => Some((Place::Settings, None)),
        _ => None,
    }
}

/// The groups, in the order the help overlay shows them.
pub fn groups() -> Vec<&'static str> {
    let mut groups: Vec<&'static str> = Vec::new();
    for action in ACTIONS {
        if !groups.contains(&action.group) {
            groups.push(action.group);
        }
    }
    groups
}

/// The actions of a group.
pub fn actions_in(group: &str) -> Vec<&'static KeyAction> {
    ACTIONS.iter().filter(|action| action.group == group).collect()
}

/// How long the modifier is held before the help appears.
pub const HELP_HOLD: std::time::Duration = std::time::Duration::from_millis(3000);

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn the_registry_is_the_source_of_the_bindings() {
        // The handler asks the registry; the help is derived from it. The two
        // can never disagree, because there is only one list.
        let action = action_for("f", true, false, false).expect("⌘F is bound");
        assert_eq!(action.id, "search.focus");
        // The same key without the modifier is nothing.
        assert!(action_for("f", false, false, false).is_none());
        // And the same key with another modifier is nothing.
        assert!(action_for("f", true, true, false).is_none());
    }

    #[test]
    fn every_action_has_a_label_a_group_and_an_id() {
        let mut ids: Vec<&str> = Vec::new();
        for action in ACTIONS {
            assert!(!action.id.is_empty());
            assert!(!action.label.is_empty());
            assert!(!action.group.is_empty());
            assert!(!ids.contains(&action.id), "the ids are unique");
            ids.push(action.id);
        }
        // And every id is reachable by its own binding.
        for action in ACTIONS {
            assert!(action_for(
                action.binding.key,
                action.binding.platform,
                action.binding.alt,
                action.binding.shift
            )
            .is_some());
        }
    }

    #[test]
    fn the_rail_actions_open_the_places_they_name() {
        assert_eq!(
            rail_target("rail.search"),
            Some((Place::Packages, Some(NavScope::Results)))
        );
        assert_eq!(
            rail_target("rail.updates"),
            Some((Place::Packages, Some(NavScope::Updates)))
        );
        assert_eq!(rail_target("rail.news"), Some((Place::News, None)));
        assert!(rail_target("sidebar.toggle").is_none());
    }

    #[test]
    fn the_groups_keep_their_order_and_their_actions() {
        let groups = groups();
        assert_eq!(groups[0], "Navigation");
        assert_eq!(groups.len(), 6);
        let navigation = actions_in("Navigation");
        assert_eq!(navigation.len(), 5);
        assert_eq!(navigation[0].id, "rail.search");
    }

    #[test]
    fn the_bindings_are_written_the_way_a_keycap_is() {
        assert_eq!(KeyBinding::platform("f").label(), "⌘F");
        assert_eq!(KeyBinding::alt_platform("c").label(), "⌘⌥C");
        assert_eq!(KeyBinding::alt("←").label(), "⌥←");
        assert_eq!(KeyBinding::new("/").label(), "/");
    }
}
