use arrow::datatypes::DataType;
use re_ui::UiExt as _;
use re_ui::list_item::PropertyContent;

/// Show an ui describing an Arrow `DataType`.
///
/// This is basically a string type name and an optional closure describing nested fields.
pub struct DataTypeUi<'a> {
    /// A short name for the type, e.g. "struct", "list", "int32", etc.
    ///
    /// Doesn't contain info about nested fields.
    pub type_name: String,

    /// A closure for showing a list item with info about nested fields.
    ///
    /// Only set if the type actually has nested fields.
    #[expect(clippy::type_complexity)]
    pub content: Option<Box<dyn FnOnce(&mut egui::Ui) + 'a>>,
}

impl<'a> DataTypeUi<'a> {
    pub fn new(data_type: &'a DataType) -> Self {
        match data_type {
            DataType::Struct(fields) => DataTypeUi {
                type_name: "Struct".to_owned(),
                content: Some(Box::new(move |ui| {
                    for field in fields {
                        data_type_field_ui(ui, field);
                    }
                })),
            },
            DataType::List(field) => DataTypeUi {
                type_name: "List".to_owned(),
                content: Some(Box::new(move |ui| {
                    data_type_field_ui(ui, field);
                })),
            },
            DataType::ListView(field) => DataTypeUi {
                type_name: "ListView".to_owned(),
                content: Some(Box::new(move |ui| {
                    data_type_field_ui(ui, field);
                })),
            },
            DataType::FixedSizeList(field, size) => DataTypeUi {
                type_name: format!("FixedSizeList({size})"),
                content: Some(Box::new(move |ui| {
                    data_type_field_ui(ui, field);
                })),
            },
            DataType::LargeList(field) => DataTypeUi {
                type_name: "LargeList".to_owned(),
                content: Some(Box::new(move |ui| {
                    data_type_field_ui(ui, field);
                })),
            },
            DataType::LargeListView(field) => DataTypeUi {
                type_name: "LargeListView".to_owned(),
                content: Some(Box::new(move |ui| {
                    data_type_field_ui(ui, field);
                })),
            },
            DataType::Union(fields, mode) => {
                let label = match mode {
                    arrow::datatypes::UnionMode::Sparse => "Union(Sparse)",
                    arrow::datatypes::UnionMode::Dense => "Union(Dense)",
                };
                DataTypeUi {
                    type_name: label.to_owned(),
                    content: Some(Box::new(move |ui| {
                        for (_, field) in fields.iter() {
                            data_type_field_ui(ui, field);
                        }
                    })),
                }
            }
            DataType::Dictionary(_k, _v) => DataTypeUi {
                type_name: "Dictionary".to_owned(),
                content: None,
            },
            DataType::Map(a, _) => DataTypeUi {
                type_name: "Map".to_owned(),
                content: Some(Box::new(move |ui| {
                    data_type_field_ui(ui, a);
                })),
            },
            DataType::RunEndEncoded(a, b) => DataTypeUi {
                type_name: "RunEndEncoded".to_owned(),
                content: Some(Box::new(move |ui| {
                    data_type_field_ui(ui, a);
                    data_type_field_ui(ui, b);
                })),
            },
            non_nested => DataTypeUi {
                type_name: non_nested.to_string(),
                content: None,
            },
        }
    }

    /// Show the data type as a `list_item`.
    ///
    /// The root item has the label "Data type".
    pub fn list_item_ui(self, ui: &mut egui::Ui) {
        let content = PropertyContent::new("Data type")
            .value_text(self.type_name)
            .show_only_when_collapsed(false);
        if let Some(datatype_ui) = self.content {
            ui.list_item().show_hierarchical_with_children(
                ui,
                ui.make_persistent_id("data_type_ui_root"),
                false,
                content,
                datatype_ui,
            );
        } else {
            ui.list_item().show_hierarchical(ui, content);
        }
    }
}

fn data_type_field_ui(ui: &mut egui::Ui, field: &arrow::datatypes::Field) {
    ui.spacing_mut().item_spacing.y = 0.0;
    let item = ui.list_item();

    let data_type_ui = DataTypeUi::new(field.data_type());

    let text = if field.is_nullable() {
        data_type_ui.type_name.clone()
    } else {
        // This follows the notation set by arrow-rs.
        // If we change this, we should probably change
        // arrow-rs and datafusion to match.
        format!("non-null {}", data_type_ui.type_name)
    };

    let property = PropertyContent::new(field.name())
        .value_text(text)
        .show_only_when_collapsed(false);

    if let Some(content) = data_type_ui.content {
        item.show_hierarchical_with_children(
            ui,
            ui.unique_id().with(field.name()),
            true,
            property,
            content,
        );
    } else {
        item.show_hierarchical(ui, property);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datatype_labels_match_table_formatting() {
        for datatype in [
            DataType::Boolean,
            DataType::Int32,
            DataType::Utf8,
            DataType::LargeUtf8,
            DataType::Binary,
            DataType::LargeBinary,
            DataType::Timestamp(arrow::datatypes::TimeUnit::Nanosecond, None),
        ] {
            let field = arrow::datatypes::Field::new("value", datatype.clone(), true);
            assert_eq!(
                DataTypeUi::new(&datatype).type_name,
                re_arrow_util::format_field_datatype(&field),
            );
        }
    }
}
