//! Pure Rust projection for the current application's base report.
//!
//! The projection mirrors the source application's `BasicReportPage` without
//! carrying any UI, browser storage, or printing concerns into the domain
//! layer. The GPUI report window consumes this deterministic document.

use std::collections::{HashMap, HashSet};

use a22_db::{DbData, InfoEntry, Item};

pub const BASIC_REPORT_COLUMN_COUNT: usize = 3;
// Keep enough vertical headroom for wrapped values and relationship labels:
// three columns render about forty rows each before a page needs to grow.
pub const BASIC_REPORT_ROWS_PER_PAGE: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportBullet {
    Email,
    Phone,
    Social,
    Default,
}

impl ReportBullet {
    pub fn glyph(self) -> char {
        match self {
            Self::Email => '✉',
            Self::Phone => '☎',
            Self::Social => '⌘',
            Self::Default => '•',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportPart {
    pub text: String,
    pub is_fat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicReportItem {
    pub id: String,
    pub main: ReportPart,
    pub others: Vec<ReportPart>,
    pub relationships: Vec<String>,
    pub bullet: ReportBullet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BasicReportRow {
    GroupDivider,
    Item(BasicReportItem),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicReportPage {
    pub rows: Vec<BasicReportRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicReportDocument {
    pub pages: Vec<BasicReportPage>,
    pub column_count: usize,
    pub rows_per_page: usize,
}

/// Builds the deterministic base-report document.
///
/// `tag_order` follows the report settings contract: an item belongs to the
/// first matching tag group only, and unclaimed items are placed in the
/// remaining group. An empty order keeps all items in one group.
pub fn build_basic_report(data: &DbData, tag_order: &[String]) -> BasicReportDocument {
    let items_by_id = data
        .items
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let links_by_item = data.item_links.iter().fold(
        HashMap::<&str, Vec<&a22_db::ItemLink>>::new(),
        |mut links_by_item, link| {
            links_by_item
                .entry(link.item_id.as_str())
                .or_default()
                .push(link);
            links_by_item
        },
    );
    let groups = group_item_indices(&data.items, tag_order);
    let mut rows = Vec::new();

    for (group_index, group) in groups.iter().enumerate() {
        if group_index > 0 {
            rows.push(BasicReportRow::GroupDivider);
        }
        rows.extend(group.iter().map(|&item_index| {
            BasicReportRow::Item(build_report_item(
                &data.items[item_index],
                &items_by_id,
                links_by_item
                    .get(data.items[item_index].id.as_str())
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
            ))
        }));
    }

    let mut pages = rows
        .chunks(BASIC_REPORT_ROWS_PER_PAGE)
        .map(|page| BasicReportPage {
            rows: page.to_vec(),
        })
        .collect::<Vec<_>>();
    if pages.is_empty() {
        pages.push(BasicReportPage { rows: Vec::new() });
    }
    if pages.len() == 1 {
        pages.push(BasicReportPage { rows: Vec::new() });
    }

    BasicReportDocument {
        pages,
        column_count: BASIC_REPORT_COLUMN_COUNT,
        rows_per_page: BASIC_REPORT_ROWS_PER_PAGE,
    }
}

fn group_item_indices(items: &[Item], tag_order: &[String]) -> Vec<Vec<usize>> {
    if tag_order.is_empty() {
        return vec![(0..items.len()).collect()];
    }

    let normalized_order = tag_order
        .iter()
        .map(|tag| tag.trim().to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    let mut claimed = HashSet::new();
    let mut groups = Vec::new();

    for tag in normalized_order {
        let group = items
            .iter()
            .enumerate()
            .filter(|(index, item)| {
                !claimed.contains(index)
                    && item
                        .tags
                        .iter()
                        .any(|item_tag| item_tag.trim().eq_ignore_ascii_case(&tag))
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        if group.is_empty() {
            continue;
        }
        claimed.extend(group.iter().copied());
        groups.push(group);
    }

    let remaining = items
        .iter()
        .enumerate()
        .filter_map(|(index, _)| (!claimed.contains(&index)).then_some(index))
        .collect::<Vec<_>>();
    if !remaining.is_empty() {
        groups.push(remaining);
    }
    groups
}

fn build_report_item(
    item: &Item,
    items_by_id: &HashMap<&str, &Item>,
    links: &[&a22_db::ItemLink],
) -> BasicReportItem {
    let main_index = resolve_main_entry_index(item);
    let main_entry = main_index.and_then(|index| item.info_entries.get(index));
    let main_text = main_entry
        .map(|entry| entry.value.clone())
        .unwrap_or_else(|| "-".to_owned());
    let others = item
        .info_entries
        .iter()
        .enumerate()
        .filter(|(index, _)| Some(*index) != main_index)
        .map(|(_, entry)| report_part(entry))
        .collect();
    let relationships = links
        .iter()
        .filter(|link| link.item_id == item.id)
        .map(|link| {
            items_by_id
                .get(link.target_id.as_str())
                .map(|target| item_display_value(target))
                .unwrap_or_else(|| link.target_id.clone())
        })
        .collect::<Vec<_>>();

    BasicReportItem {
        id: item.id.clone(),
        main: ReportPart {
            text: main_text.clone(),
            is_fat: main_entry.is_some_and(|entry| is_fat_label(&entry.label)),
        },
        others,
        relationships,
        bullet: resolve_bullet(&main_text),
    }
}

fn report_part(entry: &InfoEntry) -> ReportPart {
    ReportPart {
        text: entry.value.clone(),
        is_fat: is_fat_label(&entry.label),
    }
}

fn resolve_main_entry_index(item: &Item) -> Option<usize> {
    item.main_info_entry_id
        .as_deref()
        .and_then(|id| item.info_entries.iter().position(|entry| entry.id == id))
        .or_else(|| {
            item.info_entries
                .iter()
                .position(|entry| entry.label.trim().eq_ignore_ascii_case("title"))
        })
        .or_else(|| (!item.info_entries.is_empty()).then_some(0))
}

fn item_display_value(item: &Item) -> String {
    resolve_main_entry_index(item)
        .and_then(|index| item.info_entries.get(index))
        .map(|entry| entry.value.clone())
        .unwrap_or_else(|| "-".to_owned())
}

fn is_fat_label(label: &str) -> bool {
    label.trim().eq_ignore_ascii_case("fat")
}

fn resolve_bullet(value: &str) -> ReportBullet {
    if is_email(value) {
        ReportBullet::Email
    } else if is_phone(value) {
        ReportBullet::Phone
    } else if is_social(value) {
        ReportBullet::Social
    } else {
        ReportBullet::Default
    }
}

fn is_email(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.chars().any(char::is_whitespace) {
        return false;
    }
    let Some(at) = value.rfind('@') else {
        return false;
    };
    at > 0 && at + 1 < value.len() && value[at + 1..].contains('.') && !value[..at].contains('@')
}

fn is_phone(value: &str) -> bool {
    let compact = value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '+')
        .collect::<String>();
    let digits = compact.strip_prefix('+').unwrap_or(&compact);
    compact.len() >= 7
        && compact.len() <= 16
        && !digits.is_empty()
        && digits.chars().all(|character| character.is_ascii_digit())
        && (compact.starts_with('+') || compact.chars().all(|character| character.is_ascii_digit()))
}

fn is_social(value: &str) -> bool {
    let value = value.trim().to_lowercase();
    if value.is_empty() {
        return false;
    }
    if let Some(handle) = value.strip_prefix('@') {
        return !handle.is_empty()
            && handle
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character));
    }
    [
        "t.me",
        "telegram",
        "instagram.com",
        "facebook.com",
        "fb.com",
        "vk.com",
        "vkontakte",
        "linkedin.com",
        "twitter.com",
        "x.com",
        "youtube.com",
        "youtu.be",
        "whatsapp",
        "wa.me",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use a22_db::{DbData, ItemLink};

    fn entry(label: &str, value: &str) -> InfoEntry {
        entry_id(&format!("{label}-{value}"), label, value)
    }

    fn entry_id(id: &str, label: &str, value: &str) -> InfoEntry {
        InfoEntry {
            id: id.to_owned(),
            label: label.to_owned(),
            value: value.to_owned(),
        }
    }

    fn item(id: &str, tags: &[&str], entries: Vec<InfoEntry>, main: Option<&str>) -> Item {
        Item {
            id: id.to_owned(),
            main_info_entry_id: main.map(str::to_owned),
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            info_entries: entries,
            date_time_entries: Vec::new(),
            created_at: "2026-01-01T00:00:00.000Z".to_owned(),
            updated_at: "2026-01-01T00:00:00.000Z".to_owned(),
        }
    }

    fn data(items: Vec<Item>) -> DbData {
        DbData {
            items,
            ..DbData::default()
        }
    }

    fn report_items(document: &BasicReportDocument) -> Vec<&BasicReportItem> {
        document
            .pages
            .iter()
            .flat_map(|page| page.rows.iter())
            .filter_map(|row| match row {
                BasicReportRow::Item(item) => Some(item),
                BasicReportRow::GroupDivider => None,
            })
            .collect()
    }

    #[test]
    fn display_entry_prefers_star_then_title_then_first() {
        let starred = item(
            "starred",
            &[],
            vec![
                entry("description", "first"),
                entry("title", "title"),
                entry_id("star", "title", "starred"),
            ],
            Some("star"),
        );
        let titled = item(
            "titled",
            &[],
            vec![entry("description", "first"), entry("title", "title")],
            None,
        );
        let first = item(
            "first",
            &[],
            vec![entry("description", "first"), entry("note", "second")],
            None,
        );

        let document = build_basic_report(&data(vec![starred, titled, first]), &[]);
        let items = report_items(&document);

        assert_eq!(
            items
                .iter()
                .map(|item| item.main.text.as_str())
                .collect::<Vec<_>>(),
            ["starred", "title", "first"]
        );
    }

    #[test]
    fn bullets_and_fat_parts_match_source_rules() {
        let document = build_basic_report(
            &data(vec![
                item(
                    "email",
                    &[],
                    vec![entry("title", "person@example.com")],
                    None,
                ),
                item("phone", &[], vec![entry("title", "+381641234567")], None),
                item("social", &[], vec![entry("title", "@a22r")], None),
                item("fat", &[], vec![entry("fat", "Important")], None),
            ]),
            &[],
        );
        let items = report_items(&document);

        assert_eq!(items[0].bullet, ReportBullet::Email);
        assert_eq!(items[1].bullet, ReportBullet::Phone);
        assert_eq!(items[2].bullet, ReportBullet::Social);
        assert_eq!(items[3].bullet, ReportBullet::Default);
        assert!(items[3].main.is_fat);
    }

    #[test]
    fn grouping_claims_each_item_by_first_matching_tag() {
        let document = build_basic_report(
            &data(vec![
                item(
                    "multi",
                    &["personal", "work"],
                    vec![entry("title", "multi")],
                    None,
                ),
                item(
                    "personal",
                    &["personal"],
                    vec![entry("title", "personal")],
                    None,
                ),
                item(
                    "remaining",
                    &["other"],
                    vec![entry("title", "remaining")],
                    None,
                ),
            ]),
            &["work".to_owned(), "personal".to_owned()],
        );

        let rows = &document.pages[0].rows;
        assert!(matches!(
            &rows[0],
            BasicReportRow::Item(item) if item.id == "multi"
        ));
        assert!(matches!(&rows[1], BasicReportRow::GroupDivider));
        assert!(matches!(
            &rows[2],
            BasicReportRow::Item(item) if item.id == "personal"
        ));
        assert!(matches!(&rows[3], BasicReportRow::GroupDivider));
        assert!(matches!(
            &rows[4],
            BasicReportRow::Item(item) if item.id == "remaining"
        ));
        assert_eq!(report_items(&document).len(), 3);
    }

    #[test]
    fn report_resolves_relationship_labels_and_keeps_three_columns() {
        let source = item("source", &[], vec![entry("title", "Source")], None);
        let target = item("target", &[], vec![entry("title", "Target")], None);
        let mut data = data(vec![source, target]);
        data.item_links.push(ItemLink {
            id: "link".to_owned(),
            item_id: "source".to_owned(),
            target_id: "target".to_owned(),
            r#type: "related".to_owned(),
            created_at: "2026-01-01T00:00:00.000Z".to_owned(),
            updated_at: "2026-01-01T00:00:00.000Z".to_owned(),
        });

        let document = build_basic_report(&data, &[]);
        let items = report_items(&document);

        assert_eq!(document.column_count, BASIC_REPORT_COLUMN_COUNT);
        assert_eq!(items[0].relationships, ["Target"]);
    }

    #[test]
    fn pagination_preserves_rows_and_keeps_two_page_minimum() {
        let items = (0..BASIC_REPORT_ROWS_PER_PAGE + 1)
            .map(|index| {
                item(
                    &format!("item-{index}"),
                    &[],
                    vec![entry("title", &format!("Item {index}"))],
                    None,
                )
            })
            .collect();
        let document = build_basic_report(&data(items), &[]);

        assert_eq!(document.pages.len(), 2);
        assert_eq!(
            report_items(&document).len(),
            BASIC_REPORT_ROWS_PER_PAGE + 1
        );
        assert_eq!(document.pages[0].rows.len(), BASIC_REPORT_ROWS_PER_PAGE);
        assert_eq!(document.pages[1].rows.len(), 1);
    }

    #[test]
    fn pagination_keeps_all_items_in_a_three_hundred_item_dataset() {
        let items = (0..300)
            .map(|index| {
                item(
                    &format!("item-{index}"),
                    &[],
                    vec![entry("title", &format!("Item {index}"))],
                    None,
                )
            })
            .collect();
        let document = build_basic_report(&data(items), &[]);

        assert_eq!(document.pages.len(), 3);
        assert_eq!(report_items(&document).len(), 300);
    }
}
