//! Search and filter — Rust port of `SourceAppReadOnly/src/lib/item/store/itemStore.ts`
//! (`matchesItemSearch`, list filter) and `src/models/item/helpers.ts`
//! (`hasAllTags`, `getItemDisplayValue`). Pure functions over an explicit dataset.
//!
//! Stage 2 deviation per `docs/db-design.md` §10.4: the separate starred-label field
//! (`mainInfoField`) is not searched — the starred entry's label+value are already
//! covered by the entries loop, so the old line was redundant. Display-entry
//! resolution (star → `title` → first) keys off `mainInfoEntryId` instead of a label.

use std::collections::HashMap;

use crate::types::{DbData, InfoEntry, Item, ItemLink, ItemTimerLink, Timer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSort {
    Storage,
    TitleAscending,
    UpdatedNewest,
    CreatedNewest,
}

/// Lookup tables for one filter pass: built once from the dataset and shared by
/// every item (TS builds the same `Map`s per list render).
pub struct SearchContext<'a> {
    timers_by_id: HashMap<&'a str, &'a Timer>,
    items_by_id: HashMap<&'a str, &'a Item>,
    item_links_by_item: HashMap<&'a str, Vec<&'a ItemLink>>,
    item_timer_links_by_item: HashMap<&'a str, Vec<&'a ItemTimerLink>>,
}

impl<'a> SearchContext<'a> {
    pub fn new(data: &'a DbData) -> Self {
        let timers_by_id = data
            .timers
            .iter()
            .map(|timer| (timer.id.as_str(), timer))
            .collect();
        let items_by_id = data
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let mut item_links_by_item = HashMap::new();
        for link in &data.item_links {
            item_links_by_item
                .entry(link.item_id.as_str())
                .or_insert_with(Vec::new)
                .push(link);
        }
        let mut item_timer_links_by_item = HashMap::new();
        for link in &data.item_timer_links {
            item_timer_links_by_item
                .entry(link.item_id.as_str())
                .or_insert_with(Vec::new)
                .push(link);
        }
        Self {
            timers_by_id,
            items_by_id,
            item_links_by_item,
            item_timer_links_by_item,
        }
    }
}

/// Case-insensitive substring search over the same field set as TS
/// `matchesItemSearch`; an empty/whitespace-only query matches everything.
pub fn matches_item_search(item: &Item, ctx: &SearchContext, raw_query: &str) -> bool {
    let query = raw_query.trim().to_lowercase();
    matches_item_search_normalized(item, ctx, &query)
}

fn matches_item_search_normalized(item: &Item, ctx: &SearchContext, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let matches = |value: &str| value.to_lowercase().contains(query);

    // Stage 2: no `mainInfoField` entry here (db-design §10.4).
    if matches(&item.id)
        || matches(&item.created_at)
        || matches(&item.updated_at)
        || item
            .info_entries
            .iter()
            .any(|entry| matches(&entry.label) || matches(&entry.value))
        || item.date_time_entries.iter().any(|entry| {
            let value = entry.value.to_string();
            matches(&entry.label) || matches(&value)
        })
    {
        return true;
    }

    if let Some(links) = ctx.item_links_by_item.get(item.id.as_str()) {
        for link in links {
            if matches(&link.r#type) || matches(&link.target_id) {
                return true;
            }
            if let Some(target) = ctx.items_by_id.get(link.target_id.as_str()) {
                let target_name = item_display_value(target);
                if matches(&target_name) {
                    return true;
                }
            }
        }
    }

    if let Some(links) = ctx.item_timer_links_by_item.get(item.id.as_str()) {
        for link in links {
            let Some(timer) = ctx.timers_by_id.get(link.timer_id.as_str()) else {
                continue;
            };
            if matches(&timer.name)
                || matches(&timer.base_at)
                || matches(&timer.description)
                || matches(if timer.is_active {
                    "active"
                } else {
                    "inactive"
                })
                || matches(&link.last_checked_at)
            {
                return true;
            }
        }
    }

    false
}

/// TS `hasAllTags`: an empty selection matches everything.
pub fn has_all_tags(item: &Item, selected_tags: &[String]) -> bool {
    let normalized_tags = selected_tags
        .iter()
        .map(|tag| tag.to_lowercase())
        .collect::<Vec<_>>();
    has_all_tags_normalized(item, &normalized_tags)
}

fn has_all_tags_normalized(item: &Item, selected_tags: &[String]) -> bool {
    selected_tags.iter().all(|tag| {
        item.tags
            .iter()
            .any(|item_tag| item_tag.to_lowercase() == *tag)
    })
}

/// Store list filter (TS `items.filter(...)`): AND of the tag and search
/// predicates, storage order preserved.
pub fn filter_items(data: &DbData, query: &str, selected_tags: &[String]) -> Vec<Item> {
    let ctx = SearchContext::new(data);
    let query = query.trim().to_lowercase();
    let selected_tags = selected_tags
        .iter()
        .map(|tag| tag.to_lowercase())
        .collect::<Vec<_>>();
    data.items
        .iter()
        .filter(|item| {
            has_all_tags_normalized(item, &selected_tags)
                && matches_item_search_normalized(item, &ctx, &query)
        })
        .cloned()
        .collect()
}

/// Sort a filtered item list while retaining a deterministic ID tie-breaker.
pub fn sort_items(mut items: Vec<Item>, sort: ItemSort) -> Vec<Item> {
    if sort == ItemSort::Storage {
        return items;
    }

    items.sort_by(|left, right| {
        let ordering = match sort {
            ItemSort::Storage => std::cmp::Ordering::Equal,
            ItemSort::TitleAscending => item_display_value(left)
                .to_lowercase()
                .cmp(&item_display_value(right).to_lowercase()),
            ItemSort::UpdatedNewest => right.updated_at.cmp(&left.updated_at),
            ItemSort::CreatedNewest => right.created_at.cmp(&left.created_at),
        };
        ordering.then_with(|| left.id.cmp(&right.id))
    });
    items
}

/// TS `resolveDisplayInfoEntry`: the starred entry (by id), else the first
/// `title`-labelled entry (case-insensitive), else the first entry.
fn resolve_display_info_entry(item: &Item) -> Option<&InfoEntry> {
    if let Some(star_id) = item.main_info_entry_id.as_deref() {
        if let Some(entry) = item.info_entries.iter().find(|entry| entry.id == star_id) {
            return Some(entry);
        }
    }
    item.info_entries
        .iter()
        .find(|entry| entry.label.to_lowercase() == "title")
        .or_else(|| item.info_entries.first())
}

/// TS `getItemDisplayValue`: the display entry's value, or '-' when the item has none.
fn item_display_value(item: &Item) -> String {
    resolve_display_info_entry(item)
        .map(|entry| entry.value.clone())
        .unwrap_or_else(|| '-'.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_item(id: &str, tags: &[&str], note_value: Option<&str>) -> Item {
        Item {
            id: id.to_string(),
            main_info_entry_id: None,
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            info_entries: note_value
                .map(|value| {
                    vec![InfoEntry {
                        id: format!("{id}-e1"),
                        label: "note".to_string(),
                        value: value.to_string(),
                    }]
                })
                .unwrap_or_default(),
            date_time_entries: Vec::new(),
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn has_all_tags_semantics() {
        let item = test_item("i1", &["a", "b"], None);
        assert!(has_all_tags(&item, &[]));
        assert!(has_all_tags(&item, &["a".to_string()]));
        assert!(has_all_tags(&item, &["a".to_string(), "b".to_string()]));
        assert!(!has_all_tags(&item, &["c".to_string()]));

        let mixed_case = test_item("i2", &["Work"], None);
        assert!(has_all_tags(&mixed_case, &["work".to_string()]));
    }

    #[test]
    fn filter_items_is_and_of_search_and_tags_in_storage_order() {
        let data = DbData {
            items: vec![
                test_item("x", &["t1"], Some("foo")),
                test_item("y", &["t2"], Some("bar")),
                test_item("z", &["t2"], Some("baz")),
            ],
            ..Default::default()
        };

        let tags = vec!["t2".to_string()];
        let filtered = filter_items(&data, "b", &tags);
        let ids: Vec<&str> = filtered.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["y", "z"]);

        let filtered = filter_items(&data, "", &[]);
        let ids: Vec<&str> = filtered.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["x", "y", "z"]);

        // Search matches y and z; only y carries tag t1.
        let filtered = filter_items(&data, "b", &["t1".to_string()]);
        let ids: Vec<&str> = filtered.iter().map(|item| item.id.as_str()).collect();
        assert!(ids.is_empty());
    }

    #[test]
    fn display_value_falls_back_title_then_first() {
        let none = test_item("i0", &[], None);
        assert_eq!(item_display_value(&none), "-");

        let mut titled = test_item("i1", &[], None);
        titled.info_entries = vec![
            InfoEntry {
                id: "e1".into(),
                label: "other".into(),
                value: "first".into(),
            },
            InfoEntry {
                id: "e2".into(),
                label: "Title".into(),
                value: "titled".into(),
            },
        ];
        assert_eq!(item_display_value(&titled), "titled");

        let mut starred = titled.clone();
        starred.info_entries.insert(
            0,
            InfoEntry {
                id: "e0".into(),
                label: "custom".into(),
                value: "starred".into(),
            },
        );
        starred.main_info_entry_id = Some("e0".to_string());
        assert_eq!(item_display_value(&starred), "starred");

        // Dangling star id falls through to the title entry (TS: no label match → same).
        starred.main_info_entry_id = Some("missing".to_string());
        assert_eq!(item_display_value(&starred), "titled");
    }

    #[test]
    fn sort_items_uses_requested_order_and_id_tie_breaker() {
        let mut first = test_item("b", &[], Some("Same"));
        first.updated_at = "2026-01-02T00:00:00.000Z".to_string();
        first.created_at = "2026-01-01T00:00:00.000Z".to_string();
        let mut second = test_item("a", &[], Some("Same"));
        second.updated_at = first.updated_at.clone();
        second.created_at = "2026-01-03T00:00:00.000Z".to_string();
        let mut third = test_item("c", &[], Some("Another"));
        third.updated_at = "2026-01-03T00:00:00.000Z".to_string();

        let sorted = sort_items(
            vec![first.clone(), second.clone(), third.clone()],
            ItemSort::TitleAscending,
        );
        assert_eq!(
            sorted
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["c", "a", "b"]
        );

        let sorted = sort_items(vec![first, second, third], ItemSort::CreatedNewest);
        assert_eq!(
            sorted
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b", "c"]
        );
    }
}
