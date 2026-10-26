use std::ffi::CStr;

use clack_extensions::context_menu::{ContextMenuBuilder, ContextMenuError, Item};
use rift_plugin_params::ClapId;

use crate::MenuContext;

const FIRST_ACTION_ID: u32 = 1;

pub type ActionFn = fn(&mut MenuContext);
pub type IsEnabledFn = fn(&MenuContext) -> bool;
pub type IsCheckedFn = IsEnabledFn;

#[derive(Debug, Clone, Copy)]
pub struct MenuItem {
    label: &'static CStr,
    kind: MenuKind,
}

#[derive(Debug, Clone, Copy)]
enum MenuKind {
    Entry {
        enabled: Option<IsEnabledFn>,
        action: ActionFn,
    },
    CheckEntry {
        enabled: Option<IsEnabledFn>,
        checked: IsCheckedFn,
        action: ActionFn,
    },
    Separator,
    Submenu {
        enabled: Option<IsEnabledFn>,
        items: &'static [MenuItem],
    },
    Title {
        enabled: Option<IsEnabledFn>,
    },
}

impl MenuItem {
    pub const fn entry(label: &'static CStr, action: ActionFn) -> Self {
        Self {
            label,
            kind: MenuKind::Entry {
                enabled: None,
                action,
            },
        }
    }

    pub const fn check(
        label: &'static CStr,
        checked: fn(&MenuContext) -> bool,
        action: ActionFn,
    ) -> Self {
        Self {
            label,
            kind: MenuKind::CheckEntry {
                checked,
                action,
                enabled: None,
            },
        }
    }
    pub const fn separator() -> Self {
        Self {
            label: c"",
            kind: MenuKind::Separator,
        }
    }

    pub const fn submenu(label: &'static CStr, items: &'static [Self]) -> Self {
        Self {
            label,
            kind: MenuKind::Submenu {
                enabled: None,
                items,
            },
        }
    }

    pub const fn title(label: &'static CStr) -> Self {
        Self {
            label,
            kind: MenuKind::Title { enabled: None },
        }
    }

    pub const fn enabled_when(mut self, f: IsEnabledFn) -> Self {
        match &mut self.kind {
            MenuKind::Title { enabled, .. }
            | MenuKind::Entry { enabled, .. }
            | MenuKind::CheckEntry { enabled, .. }
            | MenuKind::Submenu { enabled, .. } => *enabled = Some(f),
            _ => (),
        };
        self
    }

    /// Adds this item, and eventually submenu items into the builder.
    /// Stops on failure.
    fn add_to(
        &self,
        builder: &mut ContextMenuBuilder,
        ctx: &MenuContext,
        next_id: &mut u32,
    ) -> Result<(), ContextMenuError> {
        match self.kind {
            MenuKind::Separator => builder.add_item(&Item::Separator),
            MenuKind::Entry { enabled, .. } => {
                let action_id = take_action_id(next_id);
                builder.add_item(&Item::Entry {
                    label: self.label,
                    enabled: enabled.is_none_or(|f| f(ctx)),
                    action_id,
                })
            }
            MenuKind::CheckEntry {
                checked, enabled, ..
            } => {
                let action_id = take_action_id(next_id);
                builder.add_item(&Item::CheckEntry {
                    label: self.label,
                    enabled: enabled.is_none_or(|f| f(ctx)),
                    checked: checked(ctx),
                    action_id,
                })
            }
            MenuKind::Submenu { enabled, items } => {
                builder.add_item(&Item::BeginSubmenu {
                    label: self.label,
                    enabled: enabled.is_none_or(|f| f(ctx)),
                })?;
                for item in items {
                    item.add_to(builder, ctx, next_id)?;
                }
                builder.add_item(&Item::EndSubmenu)
            }
            MenuKind::Title { enabled } => builder.add_item(&Item::Title {
                title: self.label,
                enabled: enabled.is_none_or(|f| f(ctx)),
            }),
        }
    }

    /// Finds the action registered under `target_id`.
    fn action_for(&self, target_id: ClapId, next_id: &mut u32) -> Option<ActionFn> {
        match self.kind {
            MenuKind::Separator | MenuKind::Title { .. } => None,
            MenuKind::Entry { action, .. } | MenuKind::CheckEntry { action, .. } => {
                let id = take_action_id(next_id);
                (id == target_id).then_some(action)
            }
            MenuKind::Submenu { items, .. } => items
                .iter()
                .find_map(|item| item.action_for(target_id, next_id)),
        }
    }
}

/// Hands out the next action id, advancing the counter.
fn take_action_id(next_id: &mut u32) -> ClapId {
    let id = ClapId::new(*next_id);
    *next_id += 1;
    id
}

/// Fills `builder` with `items`, one unique action id per actionable entry.
pub(crate) fn build_context_menu(
    items: &[MenuItem],
    builder: &mut ContextMenuBuilder,
    ctx: &MenuContext,
) -> Result<(), ContextMenuError> {
    let mut next_id = FIRST_ACTION_ID;
    for item in items {
        item.add_to(builder, ctx, &mut next_id)?;
    }
    Ok(())
}

/// Returns the action registered under `action_id`, if the menu has one.
pub(crate) fn resolve_action(items: &[MenuItem], action_id: ClapId) -> Option<ActionFn> {
    let mut next_id = FIRST_ACTION_ID;
    items
        .iter()
        .find_map(|item| item.action_for(action_id, &mut next_id))
}

#[cfg(test)]
mod tests {
    use clack_extensions::context_menu::{ContextMenuBuilderImpl, ItemKind};

    use super::*;

    fn menu_ctx() -> MenuContext {
        MenuContext::builder().maybe_param(None).build()
    }

    fn noop(_: &mut MenuContext) {}
    fn checked(_: &MenuContext) -> bool {
        true
    }

    /// Records what `build_context_menu` hands to the host.
    #[derive(Default)]
    struct Recorder {
        items: Vec<(ItemKind, Option<ClapId>)>,
    }

    impl ContextMenuBuilderImpl for Recorder {
        fn supports(&mut self, _: ItemKind) -> bool {
            true
        }

        fn add_item(&mut self, item: Item) -> Result<(), ContextMenuError> {
            let action_id = match item {
                Item::Entry { action_id, .. } | Item::CheckEntry { action_id, .. } => {
                    Some(action_id)
                }
                _ => None,
            };
            self.items.push((item.kind(), action_id));
            Ok(())
        }
    }

    static SUB: &[MenuItem] = &[MenuItem::entry(c"Sub A", noop), MenuItem::separator()];

    static NESTED_MENU: &[MenuItem] = &[
        MenuItem::title(c"Header"),
        MenuItem::entry(c"A", noop),
        MenuItem::check(c"Check", checked, noop),
        MenuItem::separator(),
        MenuItem::submenu(c"Sub", SUB),
    ];

    #[test]
    fn flattens_in_order_with_unique_ids() {
        let mut recorder = Recorder::default();
        let mut builder = ContextMenuBuilder::new(&mut recorder);
        build_context_menu(NESTED_MENU, &mut builder, &menu_ctx()).unwrap();

        let kinds: Vec<ItemKind> = recorder.items.iter().map(|(kind, _)| *kind).collect();
        assert_eq!(
            kinds,
            vec![
                ItemKind::Title,
                ItemKind::Entry,
                ItemKind::CheckEntry,
                ItemKind::Separator,
                ItemKind::BeginSubmenu,
                ItemKind::Entry,
                ItemKind::Separator,
                ItemKind::EndSubmenu,
            ],
            "structural items must survive the flattening"
        );

        let ids: Vec<u32> = recorder
            .items
            .iter()
            .filter_map(|(_, id)| *id)
            .map(ClapId::get)
            .collect();
        assert_eq!(ids, vec![1, 2, 3], "each entry needs its own action id");
    }

    #[test]
    fn resolves_every_id_it_hands_out() {
        use std::sync::atomic::{AtomicU32, Ordering};

        static CALLED: AtomicU32 = AtomicU32::new(0);

        fn act_a(_: &mut MenuContext) {
            CALLED.store(1, Ordering::SeqCst);
        }
        fn act_b(_: &mut MenuContext) {
            CALLED.store(2, Ordering::SeqCst);
        }

        static MENU: &[MenuItem] = &[
            MenuItem::title(c"Header"),
            MenuItem::entry(c"A", act_a),
            MenuItem::separator(),
            MenuItem::entry(c"B", act_b),
        ];

        let mut ctx = menu_ctx();

        resolve_action(MENU, ClapId::new(1)).expect("id 1 is `A`")(&mut ctx);
        assert_eq!(CALLED.load(Ordering::SeqCst), 1);

        resolve_action(MENU, ClapId::new(2)).expect("id 2 is `B`")(&mut ctx);
        assert_eq!(CALLED.load(Ordering::SeqCst), 2);

        assert!(resolve_action(MENU, ClapId::new(3)).is_none());
    }
}
