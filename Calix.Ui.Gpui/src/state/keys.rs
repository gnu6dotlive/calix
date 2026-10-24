//! The keyboard's registry: every action the user can trigger, and the
//! binding that triggers it.
//!
//! The registry is the single source of truth. The handler asks it which
//! action a key press means; the help overlay is *derived* from it. A key map
//! written by hand in a help panel is a lie the moment a binding changes —
//! this one cannot drift, because there is only one list.
//!
//! The gesture is the launcher's: the help appears when the user holds the
//! modifier alone (or presses F1), and the surface carries no key glyph at all.
//! The palette (Ctrl+K) lists the same actions, found by their words.
//!
//! The registry feeds GPUI's own keymap (`bindings`): each action is a GPUI
//! action, dispatched through key contexts, so a text field keeps the keys it
//! types and a deeper context wins over the window's — the lesson of Skwir,
//! where every user action is a command the keymap, the palette and the menus
//! share. Nothing compares key names by hand any more.

use crate::state::session::{NavScope, Place};
use gpui::actions;

actions!(
    calix,
    [
        RailSearch,
        RailInstalled,
        RailUpdates,
        RailNews,
        RailSettings,
        ScopePrevious,
        ScopeNext,
        FocusSearch,
        OpenSearch,
        ToggleConsole,
        ToggleSidebar,
        ListNext,
        ListPrevious,
        ListClear,
        ToggleHelp,
        OpenPalette,
    ]
);

/// The window's key context: the actions of the registry are bound in it.
pub const WORKSPACE_CONTEXT: &str = "Workspace";
/// The search field's key context.
pub const FIELD_CONTEXT: &str = "SearchInput";
/// A menu's key context: a menu owns the arrows, Enter and Escape.
pub const MENU_CONTEXT: &str = "MenuSurface";
/// The command palette's key context.
pub const PALETTE_CONTEXT: &str = "CommandPalette";

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

    /// The binding, written the way it is written on a keycap: ⌘⌥F on macOS,
    /// Ctrl+Alt+F elsewhere.
    pub fn label(&self) -> String {
        let mut label = String::new();
        if cfg!(target_os = "macos") {
            if self.platform {
                label.push('⌘');
            }
            if self.alt {
                label.push('⌥');
            }
            if self.shift {
                label.push('⇧');
            }
        } else {
            if self.platform {
                label.push_str("Ctrl+");
            }
            if self.alt {
                label.push_str("Alt+");
            }
            if self.shift {
                label.push_str("Shift+");
            }
        }
        label.push_str(&match self.key {
            "escape" => "Esc".to_string(),
            "enter" => "Enter".to_string(),
            key => key.to_uppercase(),
        });
        label
    }

    /// The binding in GPUI's keystroke syntax (`secondary-f`, `alt-left`):
    /// `secondary` is ⌘ on macOS and Ctrl elsewhere.
    pub fn keystroke(&self) -> String {
        let mut keystroke = String::new();
        if self.platform {
            keystroke.push_str("secondary-");
        }
        if self.alt {
            keystroke.push_str("alt-");
        }
        if self.shift {
            keystroke.push_str("shift-");
        }
        keystroke.push_str(match self.key {
            "←" => "left",
            "→" => "right",
            "↑" => "up",
            "↓" => "down",
            key => key,
        });
        keystroke
    }

    /// Whether the key, pressed with these modifiers, types a character into
    /// a text field (`/`, a letter): such a key belongs to the field there.
    pub fn types_a_character(&self) -> bool {
        !self.platform && !self.alt && self.key.chars().count() == 1 && self.key.is_ascii()
    }

    /// Whether a list or a menu owns this key while it has the focus.
    pub fn is_navigation(&self) -> bool {
        !self.platform && !self.alt && matches!(self.key, "↑" | "↓" | "escape" | "enter")
    }
}

/// An action the user can trigger.
#[derive(Debug, Clone, Copy)]
pub struct KeyAction {
    /// The action's id: namespaced, so a surface's actions never collide.
    pub id: &'static str,
    /// What the action does, in the user's words.
    pub label: &'static str,
    /// The group the help overlay files it under.
    pub group: &'static str,
    pub binding: KeyBinding,
    /// Builds GPUI's binding of this action, for a keystroke and a context.
    pub bind: fn(&str, Option<&str>) -> gpui::KeyBinding,
    /// The action itself, for the palette to dispatch.
    pub action: fn() -> Box<dyn gpui::Action>,
}

/// Every action, in the order the help overlay shows them.
pub const ACTIONS: &[KeyAction] = &[
    KeyAction {
        id: "rail.search",
        label: "Search",
        group: "Navigation",
        binding: KeyBinding::platform("1"),
        bind: |keys, context| gpui::KeyBinding::new(keys, RailSearch, context),
        action: || Box::new(RailSearch),
    },
    KeyAction {
        id: "rail.installed",
        label: "Installed",
        group: "Navigation",
        binding: KeyBinding::platform("2"),
        bind: |keys, context| gpui::KeyBinding::new(keys, RailInstalled, context),
        action: || Box::new(RailInstalled),
    },
    KeyAction {
        id: "rail.updates",
        label: "Updates",
        group: "Navigation",
        binding: KeyBinding::platform("3"),
        bind: |keys, context| gpui::KeyBinding::new(keys, RailUpdates, context),
        action: || Box::new(RailUpdates),
    },
    KeyAction {
        id: "rail.news",
        label: "News",
        group: "Navigation",
        binding: KeyBinding::platform("4"),
        bind: |keys, context| gpui::KeyBinding::new(keys, RailNews, context),
        action: || Box::new(RailNews),
    },
    KeyAction {
        id: "rail.settings",
        label: "Settings",
        group: "Navigation",
        binding: KeyBinding::platform("5"),
        bind: |keys, context| gpui::KeyBinding::new(keys, RailSettings, context),
        action: || Box::new(RailSettings),
    },
    KeyAction {
        id: "scope.previous",
        label: "The previous scope",
        group: "Scope",
        binding: KeyBinding::alt("←"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ScopePrevious, context),
        action: || Box::new(ScopePrevious),
    },
    KeyAction {
        id: "scope.next",
        label: "The next scope",
        group: "Scope",
        binding: KeyBinding::alt("→"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ScopeNext, context),
        action: || Box::new(ScopeNext),
    },
    KeyAction {
        id: "search.focus",
        label: "Focus the search",
        group: "Search",
        binding: KeyBinding::platform("f"),
        bind: |keys, context| gpui::KeyBinding::new(keys, FocusSearch, context),
        action: || Box::new(FocusSearch),
    },
    KeyAction {
        id: "search.open",
        label: "Focus the search",
        group: "Search",
        binding: KeyBinding::new("/"),
        bind: |keys, context| gpui::KeyBinding::new(keys, OpenSearch, context),
        action: || Box::new(OpenSearch),
    },
    KeyAction {
        id: "console.toggle",
        label: "The operation log",
        group: "The log",
        binding: KeyBinding::platform("l"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ToggleConsole, context),
        action: || Box::new(ToggleConsole),
    },
    KeyAction {
        id: "sidebar.toggle",
        label: "The rail",
        group: "View",
        binding: KeyBinding::alt_platform("c"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ToggleSidebar, context),
        action: || Box::new(ToggleSidebar),
    },
    KeyAction {
        id: "list.next",
        label: "The next package",
        group: "The list",
        binding: KeyBinding::new("↓"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ListNext, context),
        action: || Box::new(ListNext),
    },
    KeyAction {
        id: "list.previous",
        label: "The previous package",
        group: "The list",
        binding: KeyBinding::new("↑"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ListPrevious, context),
        action: || Box::new(ListPrevious),
    },
    KeyAction {
        id: "list.clear",
        label: "Clear, then go back",
        group: "The list",
        binding: KeyBinding::new("escape"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ListClear, context),
        action: || Box::new(ListClear),
    },
    KeyAction {
        id: "palette.open",
        label: "Run a command",
        group: "Help",
        binding: KeyBinding::platform("k"),
        bind: |keys, context| gpui::KeyBinding::new(keys, OpenPalette, context),
        action: || Box::new(OpenPalette),
    },
    KeyAction {
        id: "help.toggle",
        label: "The keyboard shortcuts",
        group: "Help",
        binding: KeyBinding::new("f1"),
        bind: |keys, context| gpui::KeyBinding::new(keys, ToggleHelp, context),
        action: || Box::new(ToggleHelp),
    },
];

/// GPUI's keymap, derived from the registry: every action in the window's
/// context; every key that types a character given back to the text field
/// (`NoAction` there), so `/` still types a slash in the search; and the
/// arrows, Enter and Escape given back to a menu, which owns them while it is
/// open (a menu's ↓ used to move the catalogue's selection behind it too).
pub fn bindings() -> Vec<gpui::KeyBinding> {
    let mut bindings: Vec<gpui::KeyBinding> = ACTIONS
        .iter()
        .map(|action| (action.bind)(&action.binding.keystroke(), Some(WORKSPACE_CONTEXT)))
        .collect();
    type Keeps = fn(&KeyBinding) -> bool;
    let given_back: [(&str, Keeps); 2] = [
        (FIELD_CONTEXT, KeyBinding::types_a_character),
        (MENU_CONTEXT, KeyBinding::is_navigation),
    ];
    for (context, keeps) in given_back {
        bindings.extend(
            ACTIONS
                .iter()
                .filter(|action| keeps(&action.binding))
                .map(|action| {
                    gpui::KeyBinding::new(&action.binding.keystroke(), gpui::NoAction, Some(context))
                }),
        );
    }
    bindings
}

/// The actions the palette offers for what is typed, best first.
///
/// Every word typed must start a word of the action's label or group, or be
/// found in its label; an action whose label starts with the text comes
/// first. Nothing typed offers everything, in the registry's order. The
/// palette never offers itself.
pub fn matching(query: &str) -> Vec<&'static KeyAction> {
    let query = query.trim().to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();
    let mut found: Vec<(u8, usize, &'static KeyAction)> = ACTIONS
        .iter()
        .enumerate()
        .filter(|(_, action)| action.id != "palette.open")
        .filter_map(|(order, action)| {
            let label = action.label.to_lowercase();
            let group = action.group.to_lowercase();
            let starts = |word: &str| {
                label
                    .split_whitespace()
                    .chain(group.split_whitespace())
                    .any(|w| w.starts_with(word))
            };
            if !words.iter().all(|word| starts(word) || label.contains(word)) {
                return None;
            }
            let rank = if words.is_empty() || label.starts_with(&query) {
                0
            } else if words.iter().all(|word| starts(word)) {
                1
            } else {
                2
            };
            Some((rank, order, action))
        })
        .collect();
    found.sort_by_key(|(rank, order, _)| (*rank, *order));
    // Two keys for one thing (⌘F and /) are one line: the palette names
    // each thing once, with its first key.
    let mut offered: Vec<&'static KeyAction> = Vec::new();
    for (_, _, action) in found {
        if !offered.iter().any(|o| o.label == action.label) {
            offered.push(action);
        }
    }
    offered
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
    fn the_registry_is_the_source_of_gpuis_keymap() {
        // Every action is bound in the window's context, under the keystroke
        // its binding writes: the help, the palette and the keymap read the
        // one list, so they can never disagree.
        let bindings = bindings();
        for action in ACTIONS {
            let keystroke = gpui::Keystroke::parse(&action.binding.keystroke())
                .expect("every binding parses as a GPUI keystroke");
            assert!(
                bindings.iter().any(|binding| {
                    binding.match_keystrokes(std::slice::from_ref(&keystroke)) == Some(false)
                        && binding.action().partial_eq((action.action)().as_ref())
                }),
                "{} is bound",
                action.id
            );
        }
        assert_eq!(KeyBinding::platform("f").keystroke(), "secondary-f");
        assert_eq!(KeyBinding::alt("←").keystroke(), "alt-left");
        assert_eq!(KeyBinding::new("↓").keystroke(), "down");
    }

    #[test]
    fn a_field_keeps_the_characters_and_a_menu_its_arrows() {
        let given_back = |context: &str, keystroke: &str| {
            let keystroke = gpui::Keystroke::parse(keystroke).unwrap();
            bindings().iter().any(|binding| {
                binding.action().partial_eq(&gpui::NoAction)
                    && binding.match_keystrokes(std::slice::from_ref(&keystroke)) == Some(false)
                    && binding.predicate().is_some_and(|p| p.to_string() == context)
            })
        };
        // `/` types a slash in the search, and focuses it elsewhere.
        assert!(given_back(FIELD_CONTEXT, "/"));
        // ⌘F is no character: it stays the window's in the field.
        assert!(!given_back(FIELD_CONTEXT, "secondary-f"));
        // A menu walks its items with the arrows and closes on Escape.
        assert!(given_back(MENU_CONTEXT, "down"));
        assert!(given_back(MENU_CONTEXT, "up"));
        assert!(given_back(MENU_CONTEXT, "escape"));
        assert!(!given_back(MENU_CONTEXT, "alt-left"));
    }

    /// A window like Calix's: the actions on its root, and inside it a place
    /// that holds the focus under a key context of its own.
    struct Window_ {
        inner: gpui::FocusHandle,
        context: &'static str,
        ran: Vec<&'static str>,
        keys_seen: Vec<String>,
    }

    impl gpui::Render for Window_ {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            use gpui::{InteractiveElement, ParentElement};
            gpui::div()
                .key_context(WORKSPACE_CONTEXT)
                .on_action(cx.listener(|v, _: &ListNext, _, _| v.ran.push("list.next")))
                .on_action(cx.listener(|v, _: &OpenSearch, _, _| v.ran.push("search.open")))
                .on_action(cx.listener(|v, _: &FocusSearch, _, _| v.ran.push("search.focus")))
                .child(
                    gpui::div()
                        .key_context(self.context)
                        .track_focus(&self.inner)
                        .on_key_down(cx.listener(|v, e: &gpui::KeyDownEvent, _, _| {
                            v.keys_seen.push(e.keystroke.key.clone())
                        })),
                )
        }
    }

    fn focused_in<'a>(
        context: &'static str,
        cx: &'a mut gpui::TestAppContext,
    ) -> (gpui::Entity<Window_>, &'a mut gpui::VisualTestContext) {
        cx.update(|cx| cx.bind_keys(bindings()));
        let (view, cx) = cx.add_window_view(|_, cx| Window_ {
            inner: cx.focus_handle(),
            context,
            ran: Vec::new(),
            keys_seen: Vec::new(),
        });
        cx.update(|window, cx| {
            let inner = view.read(cx).inner.clone();
            window.focus(&inner, cx);
        });
        cx.run_until_parked();
        (view, cx)
    }

    #[gpui::test]
    fn an_open_menu_owns_its_arrows(cx: &mut gpui::TestAppContext) {
        let (view, cx) = focused_in(MENU_CONTEXT, cx);
        cx.simulate_keystrokes("down");
        view.read_with(cx, |v, _| {
            assert!(v.ran.is_empty(), "the list behind the menu does not move");
            assert_eq!(v.keys_seen, vec!["down".to_string()], "the menu hears it");
        });
    }

    #[gpui::test]
    fn the_field_types_its_slash_and_the_window_keeps_its_shortcuts(
        cx: &mut gpui::TestAppContext,
    ) {
        let (view, cx) = focused_in(FIELD_CONTEXT, cx);
        cx.simulate_keystrokes("/ secondary-f down");
        view.read_with(cx, |v, _| {
            assert_eq!(v.ran, vec!["search.focus", "list.next"]);
            assert_eq!(v.keys_seen, vec!["/".to_string()]);
        });
    }

    #[gpui::test]
    fn elsewhere_the_slash_opens_the_search(cx: &mut gpui::TestAppContext) {
        let (view, cx) = focused_in("Elsewhere", cx);
        cx.simulate_keystrokes("/");
        view.read_with(cx, |v, _| assert_eq!(v.ran, vec!["search.open"]));
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
    }

    #[test]
    fn the_palette_finds_by_the_words_of_a_label() {
        // Nothing typed: everything, once, in the registry's order, and never
        // the palette itself.
        let all = matching("");
        assert_eq!(all[0].id, "rail.search");
        assert!(all.iter().all(|action| action.id != "palette.open"));
        let focus = all.iter().filter(|a| a.label == "Focus the search").count();
        assert_eq!(focus, 1, "one line for one thing");
        // A word's start finds it, whatever the case.
        assert_eq!(matching("upd")[0].id, "rail.updates");
        assert_eq!(matching("LOG")[0].id, "console.toggle");
        // Every word must be found.
        assert_eq!(matching("next pack")[0].id, "list.next");
        assert!(matching("next zebra").is_empty());
        // A label that starts with the text comes before one that holds it.
        let scope = matching("the next");
        assert_eq!(scope[0].id, "scope.next");
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
        assert_eq!(groups.len(), 7);
        let navigation = actions_in("Navigation");
        assert_eq!(navigation.len(), 5);
        assert_eq!(navigation[0].id, "rail.search");
    }

    #[test]
    fn the_bindings_are_written_the_way_a_keycap_is() {
        if cfg!(target_os = "macos") {
            assert_eq!(KeyBinding::platform("f").label(), "⌘F");
            assert_eq!(KeyBinding::alt_platform("c").label(), "⌘⌥C");
            assert_eq!(KeyBinding::alt("←").label(), "⌥←");
        } else {
            assert_eq!(KeyBinding::platform("f").label(), "Ctrl+F");
            assert_eq!(KeyBinding::alt_platform("c").label(), "Ctrl+Alt+C");
            assert_eq!(KeyBinding::alt("←").label(), "Alt+←");
        }
        assert_eq!(KeyBinding::new("/").label(), "/");
        assert_eq!(KeyBinding::new("escape").label(), "Esc");
    }
}
