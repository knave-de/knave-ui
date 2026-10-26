use super::collection::COLLECTION_ROW_HEIGHT as ROW;
use super::*;
use crate::{DisplayCommand, ImageStyle, ShapePaint, TextWrap, Transform2D};

pub(super) fn paint_collection(
    list: &mut DisplayList,
    node: &Node,
    collection: &Collection,
    computed: &Style,
    layout: Layout,
) {
    let area = layout.insets().apply(node.bounds);
    let clip = node.clip.and_then(|clip| clip.intersection(area));
    let header = collection.header_height();
    let content = Rect::new(
        area.x,
        area.y + header,
        area.width,
        (area.height - header).max(0.0),
    );
    let content_clip = clip.and_then(|clip| clip.intersection(content));
    let mut style = computed.text.clone();
    style.wrap = TextWrap::None;
    if let CollectionKind::Table { headers } = &collection.kind {
        let width = area.width / headers.len() as f32;
        for (column, header) in headers.iter().enumerate() {
            let text = if let Some((sorted, descending)) =
                collection.sort.filter(|(sorted, _)| *sorted == column)
            {
                let _ = sorted;
                format!("{header} {}", if descending { "▾" } else { "▴" })
            } else {
                header.clone()
            };
            list.commands.push(DisplayCommand::Text {
                bounds: Rect::new(area.x + width * column as f32, area.y, width, ROW),
                text,
                style: style.clone(),
                transform: Transform2D::IDENTITY,
                clip,
            });
        }
    }
    let columns = collection.columns();
    let cell_width = area.width / columns as f32;
    let start = (node.scroll / ROW).floor().max(0.0) as usize * columns;
    let end = (((node.scroll + content.height) / ROW).ceil().max(0.0) as usize)
        .saturating_add(1)
        .saturating_mul(columns)
        .min(collection.visible_items().len());
    for position in start.min(end)..end {
        let (index, depth) = collection.visible_items()[position];
        let item = &collection.items[index];
        let bounds = Rect::new(
            area.x + (position % columns) as f32 * cell_width,
            content.y + (position / columns) as f32 * ROW - node.scroll,
            cell_width,
            ROW,
        );
        let item_clip = content_clip.and_then(|clip| clip.intersection(bounds));
        if item_clip.is_none() {
            continue;
        }
        if collection.selected == Some(item.id) {
            list.commands.push(DisplayCommand::Shape {
                bounds,
                paint: ShapePaint::fill(computed.selected),
                transform: Transform2D::IDENTITY,
                clip: item_clip,
            });
        }
        let mut style = style.clone();
        if !item.enabled || !node.element.enabled {
            style.color.alpha = 105;
        }
        let mut label_bounds = bounds;
        if let CollectionKind::Table { headers } = &collection.kind {
            let width = bounds.width / headers.len() as f32;
            for (column, text) in std::iter::once(&item.label)
                .chain(item.cells.iter())
                .take(headers.len())
                .enumerate()
            {
                list.commands.push(DisplayCommand::Text {
                    bounds: Rect::new(bounds.x + column as f32 * width, bounds.y, width, ROW),
                    text: text.clone(),
                    style: style.clone(),
                    transform: Transform2D::IDENTITY,
                    clip: item_clip.and_then(|clip| {
                        clip.intersection(Rect::new(
                            bounds.x + column as f32 * width,
                            bounds.y,
                            width,
                            ROW,
                        ))
                    }),
                });
            }
            continue;
        }
        label_bounds.x += depth as f32 * 16.0;
        label_bounds.width = (bounds.x + bounds.width - label_bounds.x).max(0.0);
        if let Some(image) = &item.icon {
            list.commands.push(DisplayCommand::Image {
                bounds: Rect::new(label_bounds.x, bounds.y + 6.0, 24.0, 24.0),
                image: image.clone(),
                style: ImageStyle::default(),
                transform: Transform2D::IDENTITY,
                clip: item_clip,
            });
            label_bounds.x += 28.0;
            label_bounds.width = (label_bounds.width - 28.0).max(0.0);
        }
        let text = if matches!(collection.kind, CollectionKind::Tree)
            && collection.has_children(item.id)
        {
            format!("{} {}", if item.expanded { "▾" } else { "▸" }, item.label)
        } else {
            item.label.clone()
        };
        list.commands.push(DisplayCommand::Text {
            bounds: label_bounds,
            text,
            style,
            transform: Transform2D::IDENTITY,
            clip: item_clip,
        });
    }
}
