use a22_application::Item;
use a22_core::normalize::{
    DateTimeEntryInput, InfoEntryInput, ItemCreateInput, ItemUpdateInput, StarUpdate,
};
use a22_core::state::{CreateItemInput, RelationshipInput, UpdateItemInput};
use uuid::Uuid;

const TITLE_LABEL: &str = "title";
const DATE_LABEL: &str = "date";

/// Editable fields owned by the UI until the user explicitly saves them.
///
/// The original entry vectors are retained so fields that are not editable in
/// this phase keep their identities and values when an item is updated.
#[derive(Debug, Clone)]
pub struct ItemDraft {
    pub title: String,
    pub tags: Vec<String>,
    pub date: Option<i64>,
    info_entries: Vec<InfoEntryInput>,
    date_time_entries: Vec<DateTimeEntryInput>,
    date_entry_index: Option<usize>,
    title_entry_id: Option<String>,
    main_info_entry_id: Option<String>,
    relationships: Option<Vec<RelationshipInput>>,
}

impl ItemDraft {
    pub fn empty() -> Self {
        let title_id = new_info_entry_id();
        Self {
            title: String::new(),
            tags: Vec::new(),
            date: None,
            info_entries: vec![InfoEntryInput {
                id: Some(title_id.clone()),
                label: TITLE_LABEL.to_owned(),
                value: String::new(),
            }],
            date_time_entries: Vec::new(),
            date_entry_index: None,
            title_entry_id: Some(title_id.clone()),
            main_info_entry_id: Some(title_id),
            relationships: Some(Vec::new()),
        }
    }

    pub fn from_item(item: &Item) -> Self {
        let mut info_entries = item
            .info_entries
            .iter()
            .map(|entry| InfoEntryInput {
                id: Some(entry.id.clone()),
                label: entry.label.clone(),
                value: entry.value.clone(),
            })
            .collect::<Vec<_>>();
        if info_entries.is_empty() {
            let title_id = new_info_entry_id();
            info_entries.push(InfoEntryInput {
                id: Some(title_id),
                label: TITLE_LABEL.to_owned(),
                value: String::new(),
            });
        }
        let main_info_entry_id = item.main_info_entry_id.clone();
        let title_entry_id = main_info_entry_id
            .clone()
            .or_else(|| info_entries.first().and_then(|entry| entry.id.clone()));
        let title = title_entry_id
            .as_deref()
            .and_then(|id| {
                info_entries
                    .iter()
                    .find(|entry| entry.id.as_deref() == Some(id))
            })
            .or_else(|| info_entries.first())
            .map(|entry| entry.value.clone())
            .unwrap_or_default();
        let date_time_entries = item
            .date_time_entries
            .iter()
            .map(|entry| DateTimeEntryInput {
                label: entry.label.clone(),
                value: Some(entry.value as f64),
            })
            .collect::<Vec<_>>();
        let date = item
            .date_time_entries
            .iter()
            .find(|entry| entry.label == DATE_LABEL)
            .map(|entry| entry.value);
        let date_entry_index = item
            .date_time_entries
            .iter()
            .position(|entry| entry.label == DATE_LABEL);

        Self {
            title,
            tags: item.tags.clone(),
            date,
            info_entries,
            date_time_entries,
            date_entry_index,
            title_entry_id,
            main_info_entry_id,
            relationships: None,
        }
    }

    pub fn from_item_with_relationships(
        item: &Item,
        relationships: Vec<RelationshipInput>,
    ) -> Self {
        let mut draft = Self::from_item(item);
        draft.relationships = Some(relationships);
        draft
    }

    pub fn has_title(&self) -> bool {
        !self.title.trim().is_empty()
    }

    pub fn content_eq(&self, other: &Self) -> bool {
        self.title == other.title
            && self.tags == other.tags
            && self.date == other.date
            && self
                .info_entries
                .iter()
                .map(|entry| (&entry.id, &entry.label, &entry.value))
                .eq(other
                    .info_entries
                    .iter()
                    .map(|entry| (&entry.id, &entry.label, &entry.value)))
            && self
                .date_time_entries
                .iter()
                .map(|entry| (&entry.label, entry.value))
                .eq(other
                    .date_time_entries
                    .iter()
                    .map(|entry| (&entry.label, entry.value)))
            && self.date_entry_index == other.date_entry_index
            && self.title_entry_id == other.title_entry_id
            && self.main_info_entry_id == other.main_info_entry_id
            && self.relationships == other.relationships
    }

    pub fn set_date(&mut self, date: Option<i64>) {
        self.date = date;
    }

    pub fn info_entries(&self) -> &[InfoEntryInput] {
        &self.info_entries
    }

    pub fn date_time_entries(&self) -> &[DateTimeEntryInput] {
        &self.date_time_entries
    }

    pub fn date_entry_index(&self) -> Option<usize> {
        self.date_entry_index
    }

    pub fn update_date_time_label(&mut self, index: usize, label: String) {
        if let Some(entry) = self.date_time_entries.get_mut(index) {
            entry.label = label;
        }
    }

    pub fn add_date_time_entry(&mut self) {
        self.date_time_entries.push(DateTimeEntryInput {
            label: String::new(),
            value: None,
        });
    }

    pub fn remove_date_time_entry(&mut self, index: usize) {
        if self.date_entry_index == Some(index) || index >= self.date_time_entries.len() {
            return;
        }
        self.date_time_entries.remove(index);
        if let Some(date_index) = self.date_entry_index {
            if index < date_index {
                self.date_entry_index = Some(date_index - 1);
            }
        }
    }

    pub fn set_date_time_value(&mut self, index: usize, value: Option<i64>) {
        if let Some(entry) = self.date_time_entries.get_mut(index) {
            entry.value = value.map(|value| value as f64);
        }
    }

    pub fn update_info_entry(&mut self, index: usize, label: String, value: String) {
        let is_title = self
            .info_entries
            .get(index)
            .and_then(|entry| entry.id.as_deref())
            .zip(self.title_entry_id.as_deref())
            .is_some_and(|(entry_id, title_id)| entry_id == title_id);
        if let Some(entry) = self.info_entries.get_mut(index) {
            entry.label = label;
            entry.value = value;
            if is_title {
                self.title = entry.value.clone();
            }
        }
    }

    pub fn add_info_entry(&mut self) {
        self.info_entries.push(InfoEntryInput {
            id: Some(new_info_entry_id()),
            label: String::new(),
            value: String::new(),
        });
    }

    pub fn remove_info_entry(&mut self, index: usize) {
        if self.info_entries.len() <= 1 || index >= self.info_entries.len() {
            return;
        }
        let removed_id = self.info_entries[index].id.clone();
        self.info_entries.remove(index);
        if removed_id.as_deref() == self.main_info_entry_id.as_deref() {
            self.main_info_entry_id = self.info_entries.first().and_then(|entry| entry.id.clone());
        }
        if removed_id.as_deref() == self.title_entry_id.as_deref() {
            self.title_entry_id = self.info_entries.first().and_then(|entry| entry.id.clone());
            self.title = self
                .info_entries
                .first()
                .map(|entry| entry.value.clone())
                .unwrap_or_default();
        }
    }

    pub fn is_main_info_entry(&self, index: usize) -> bool {
        self.info_entries
            .get(index)
            .and_then(|entry| entry.id.as_deref())
            .zip(self.main_info_entry_id.as_deref())
            .is_some_and(|(entry_id, main_id)| entry_id == main_id)
    }

    pub fn set_main_info_entry(&mut self, index: usize) {
        let Some(id) = self
            .info_entries
            .get(index)
            .and_then(|entry| entry.id.clone())
        else {
            return;
        };
        self.main_info_entry_id = Some(id);
        self.title_entry_id = self.main_info_entry_id.clone();
        self.title = self
            .info_entries
            .get(index)
            .map(|entry| entry.value.clone())
            .unwrap_or_default();
    }

    pub fn relationships(&self) -> Option<&[RelationshipInput]> {
        self.relationships.as_deref()
    }

    pub fn add_relationship(&mut self, target_id: String) {
        if let Some(relationships) = self.relationships.as_mut() {
            relationships.push(RelationshipInput {
                target_id,
                r#type: "related".to_owned(),
            });
        }
    }

    pub fn update_relationship_type(&mut self, index: usize, relationship_type: String) {
        if let Some(relationship) = self
            .relationships
            .as_mut()
            .and_then(|relationships| relationships.get_mut(index))
        {
            relationship.r#type = relationship_type;
        }
    }

    pub fn set_relationship_target(&mut self, index: usize, target_id: String) {
        if let Some(relationship) = self
            .relationships
            .as_mut()
            .and_then(|relationships| relationships.get_mut(index))
        {
            relationship.target_id = target_id;
        }
    }

    pub fn remove_relationship(&mut self, index: usize) {
        if let Some(relationships) = self.relationships.as_mut() {
            if index < relationships.len() {
                relationships.remove(index);
            }
        }
    }

    pub fn to_create_input(&self) -> CreateItemInput {
        CreateItemInput {
            item: ItemCreateInput {
                tags: self.tags.clone(),
                info_entries: self.info_entries_for_create(),
                date_time_entries: self.date_time_entries_for_save(),
                main_info_entry_id: self.main_info_entry_id.clone(),
            },
            relationships: Vec::new(),
        }
    }

    pub fn to_update_input(&self) -> UpdateItemInput {
        UpdateItemInput {
            item: ItemUpdateInput {
                tags: Some(self.tags.clone()),
                info_entries: Some(self.info_entries_for_save()),
                date_time_entries: Some(self.date_time_entries_for_save()),
                main_info_entry_id: self
                    .main_info_entry_id
                    .clone()
                    .map_or(StarUpdate::Clear, StarUpdate::Set),
            },
            relationships: self.relationships.clone(),
        }
    }

    fn info_entries_for_create(&self) -> Vec<InfoEntryInput> {
        self.entries_with_title_value()
    }

    fn info_entries_for_save(&self) -> Vec<InfoEntryInput> {
        self.entries_with_title_value()
    }

    fn entries_with_title_value(&self) -> Vec<InfoEntryInput> {
        self.info_entries
            .iter()
            .map(|entry| InfoEntryInput {
                id: entry.id.clone(),
                label: entry.label.clone(),
                value: if entry.id.as_deref() == self.title_entry_id.as_deref() {
                    self.title.trim().to_owned()
                } else {
                    entry.value.clone()
                },
            })
            .collect()
    }

    fn date_time_entries_for_save(&self) -> Vec<DateTimeEntryInput> {
        let mut entries = Vec::with_capacity(self.date_time_entries.len() + 1);
        let mut date_written = false;
        for (index, entry) in self.date_time_entries.iter().enumerate() {
            if self.date_entry_index == Some(index) {
                if let Some(date) = self.date {
                    entries.push(DateTimeEntryInput {
                        label: DATE_LABEL.to_owned(),
                        value: Some(date as f64),
                    });
                }
                date_written = true;
            } else {
                entries.push(entry.clone());
            }
        }
        if !date_written {
            if let Some(date) = self.date {
                entries.push(DateTimeEntryInput {
                    label: DATE_LABEL.to_owned(),
                    value: Some(date as f64),
                });
            }
        }
        entries
    }
}

fn new_info_entry_id() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use a22_application::{DateTimeEntry, InfoEntry, Item};

    fn item() -> Item {
        Item {
            id: "item-1".to_owned(),
            main_info_entry_id: Some("info-1".to_owned()),
            tags: vec!["one".to_owned()],
            info_entries: vec![
                InfoEntry {
                    id: "info-1".to_owned(),
                    label: TITLE_LABEL.to_owned(),
                    value: "Original".to_owned(),
                },
                InfoEntry {
                    id: "info-2".to_owned(),
                    label: "note".to_owned(),
                    value: "Keep me".to_owned(),
                },
            ],
            date_time_entries: vec![
                DateTimeEntry {
                    label: "created".to_owned(),
                    value: 123,
                },
                DateTimeEntry {
                    label: DATE_LABEL.to_owned(),
                    value: 456,
                },
            ],
            created_at: "created".to_owned(),
            updated_at: "updated".to_owned(),
        }
    }

    #[test]
    fn empty_draft_converts_to_create_payload() {
        let mut draft = ItemDraft::empty();
        draft.title = " New item ".to_owned();
        draft.tags = vec!["tag".to_owned()];
        draft.set_date(Some(100));

        let input = draft.to_create_input();
        assert_eq!(input.item.tags, vec!["tag"]);
        assert!(input.item.info_entries[0].id.is_some());
        assert_eq!(input.item.info_entries[0].value, "New item");
        assert_eq!(input.item.main_info_entry_id, input.item.info_entries[0].id);
        assert_eq!(input.item.date_time_entries[0].value, Some(100.0));
    }

    #[test]
    fn hydration_preserves_entry_ids_and_unedited_fields() {
        let mut draft = ItemDraft::from_item(&item());
        draft.title = "Changed".to_owned();
        draft.tags = vec!["two".to_owned()];
        draft.set_date(Some(789));

        let input = draft.to_update_input();
        let entries = input.item.info_entries.expect("info entries");
        assert_eq!(entries[0].id.as_deref(), Some("info-1"));
        assert_eq!(entries[0].value, "Changed");
        assert_eq!(entries[1].id.as_deref(), Some("info-2"));
        assert_eq!(entries[1].value, "Keep me");
        assert_eq!(
            input.item.main_info_entry_id,
            StarUpdate::Set("info-1".to_owned())
        );
        assert_eq!(input.relationships, None);

        let dates = input.item.date_time_entries.expect("date entries");
        assert_eq!(dates[0].label, "created");
        assert_eq!(dates[0].value, Some(123.0));
        assert_eq!(dates[1].label, DATE_LABEL);
        assert_eq!(dates[1].value, Some(789.0));
    }

    #[test]
    fn dynamic_date_time_rows_preserve_order_and_special_date() {
        let mut draft = ItemDraft::from_item(&item());
        draft.update_date_time_label(0, "created-at".to_owned());
        draft.set_date_time_value(0, Some(789));
        draft.add_date_time_entry();
        draft.update_date_time_label(2, "reviewed".to_owned());
        draft.set_date_time_value(2, Some(900));

        let input = draft.to_update_input();
        let dates = input.item.date_time_entries.expect("date entries");
        assert_eq!(
            dates
                .iter()
                .map(|entry| entry.label.as_str())
                .collect::<Vec<_>>(),
            vec!["created-at", DATE_LABEL, "reviewed"]
        );
        assert_eq!(dates[0].value, Some(789.0));
        assert_eq!(dates[1].value, Some(456.0));
        assert_eq!(dates[2].value, Some(900.0));
    }

    #[test]
    fn empty_dynamic_date_time_row_is_kept_in_draft_payload_until_normalization() {
        let mut draft = ItemDraft::empty();
        draft.add_date_time_entry();
        draft.update_date_time_label(0, "reviewed".to_owned());

        let input = draft.to_create_input();
        assert_eq!(input.item.date_time_entries.len(), 1);
        assert_eq!(input.item.date_time_entries[0].label, "reviewed");
        assert_eq!(input.item.date_time_entries[0].value, None);
    }

    #[test]
    fn invalid_title_is_detected_before_submission() {
        let mut draft = ItemDraft::empty();
        assert!(!draft.has_title());
        draft.title = "  title  ".to_owned();
        assert!(draft.has_title());
    }

    #[test]
    fn edit_preserves_an_unstarred_item() {
        let mut source = item();
        source.main_info_entry_id = None;

        let mut draft = ItemDraft::from_item(&source);
        let unchanged = draft.to_update_input();
        assert_eq!(unchanged.item.main_info_entry_id, StarUpdate::Clear);

        draft.update_info_entry(0, TITLE_LABEL.to_owned(), "Changed".to_owned());
        draft.remove_info_entry(0);
        let updated = draft.to_update_input();
        assert_eq!(updated.item.main_info_entry_id, StarUpdate::Clear);
        assert_eq!(
            updated.item.info_entries.expect("info entries")[0].value,
            "Keep me"
        );
    }

    #[test]
    fn dynamic_entries_star_and_relationships_stay_in_the_update_payload() {
        let mut draft = ItemDraft::from_item_with_relationships(
            &item(),
            vec![RelationshipInput {
                target_id: "item-2".to_owned(),
                r#type: "related".to_owned(),
            }],
        );
        draft.update_info_entry(1, "description".to_owned(), "Changed note".to_owned());
        draft.set_main_info_entry(1);
        draft.add_info_entry();
        draft.update_info_entry(2, "extra".to_owned(), "Value".to_owned());
        draft.update_relationship_type(0, "contact".to_owned());

        let input = draft.to_update_input();
        let entries = input.item.info_entries.expect("info entries");
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[1].label, "description");
        assert_eq!(entries[2].value, "Value");
        assert_eq!(
            input.item.main_info_entry_id,
            StarUpdate::Set("info-2".to_owned())
        );
        assert_eq!(
            input.relationships.expect("relationships")[0].r#type,
            "contact"
        );
    }
}
