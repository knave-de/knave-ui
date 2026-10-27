use super::collection::COLLECTION_ROW_HEIGHT as ROW;
use super::*;
use crate::{DisplayCommand, ImageStyle, ShapePaint, TextWrap, Transform2D};

pub(super) fn paint_collection(
    list: &mut DisplayList,
    scene: &Scene,
    owner: usize,
    collection: &Collection,
    computed: &Style,
    layout: Layout,
) {
    let node = &scene.nodes[owner];
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
            let bounds = Rect::new(area.x + width * column as f32, area.y, width, ROW);
            let state = InteractionState {
                hovered: scene.pointer_inside
                    && scene.hover == Some(node.element.id)
                    && contains(bounds, scene.pointer),
                selected: collection.sort.is_some_and(|(sorted, _)| sorted == column),
                disabled: !node.element.enabled,
                focused: scene.focus == Some(node.element.id),
                ..InteractionState::default()
            };
            let mut base = computed.clone();
            base.text = style.clone();
            base.background = None;
            let part = css::part_style(scene, owner, "table-header", state, base.clone());
            if part.as_ref().is_some_and(|part| {
                part.background_explicit || super::paint::has_borders(&part.computed)
            }) {
                let paint = super::paint::part_background(
                    scene,
                    owner,
                    "table-header",
                    state,
                    &base,
                    ShapePaint::fill(Color::rgba(0, 0, 0, 0)),
                );
                list.commands.push(DisplayCommand::Shape {
                    bounds,
                    paint,
                    transform: Transform2D::IDENTITY,
                    clip,
                });
            }
            let header_style = part.map_or_else(|| style.clone(), |part| part.computed.style.text);
            list.commands.push(DisplayCommand::Text {
                bounds,
                text,
                style: header_style,
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
        let selected = collection.selected == Some(item.id);
        let state = InteractionState {
            hovered: scene.pointer_inside
                && scene.hover == Some(node.element.id)
                && scene.menu.is_none()
                && contains(bounds, scene.pointer),
            selected,
            disabled: !item.enabled || !node.element.enabled,
            focused: selected && scene.focus == Some(node.element.id),
            ..InteractionState::default()
        };
        let mut base = computed.clone();
        base.text = style.clone();
        base.background = None;
        let part = css::part_style(scene, owner, "collection-item", state, base.clone());
        let has_part_shape = part.as_ref().is_some_and(|part| {
            part.background_explicit || super::paint::has_borders(&part.computed)
        });
        if selected || has_part_shape {
            let default = if selected {
                computed
                    .background
                    .as_ref()
                    .and_then(|background| background.fill)
                    .unwrap_or(computed.selected)
            } else {
                Color::rgba(0, 0, 0, 0)
            };
            let paint = super::paint::part_background(
                scene,
                owner,
                "collection-item",
                state,
                &base,
                ShapePaint::fill(default),
            );
            list.commands.push(DisplayCommand::Shape {
                bounds,
                paint,
                transform: Transform2D::IDENTITY,
                clip: item_clip,
            });
        }
        let mut style = part.map_or_else(|| style.clone(), |part| part.computed.style.text);
        if state.disabled {
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
                let cell_bounds = Rect::new(bounds.x + column as f32 * width, bounds.y, width, ROW);
                let cell_clip = item_clip.and_then(|clip| clip.intersection(cell_bounds));
                let mut cell_state = state;
                cell_state.hovered &= contains(cell_bounds, scene.pointer);
                let mut cell_base = computed.clone();
                cell_base.text = style.clone();
                cell_base.background = None;
                let cell =
                    css::part_style(scene, owner, "table-cell", cell_state, cell_base.clone());
                if cell.as_ref().is_some_and(|part| {
                    part.background_explicit || super::paint::has_borders(&part.computed)
                }) {
                    let paint = super::paint::part_background(
                        scene,
                        owner,
                        "table-cell",
                        cell_state,
                        &cell_base,
                        ShapePaint::fill(Color::rgba(0, 0, 0, 0)),
                    );
                    list.commands.push(DisplayCommand::Shape {
                        bounds: cell_bounds,
                        paint,
                        transform: Transform2D::IDENTITY,
                        clip: cell_clip,
                    });
                }
                list.commands.push(DisplayCommand::Text {
                    bounds: cell_bounds,
                    text: text.clone(),
                    style: cell.map_or_else(|| style.clone(), |part| part.computed.style.text),
                    transform: Transform2D::IDENTITY,
                    clip: cell_clip,
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
