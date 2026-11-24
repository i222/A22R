use std::collections::HashMap;

use a22_application::{Item, ItemLink};
use a22_ui::{theme, Button, Size, Variant};
use gpui_kit::{
    canvas, div, prelude::*, px, rgb, Bounds, Context, DragMoveEvent, Empty, Entity, EventEmitter,
    IntoElement, MouseButton, PathBuilder, Pixels, Point, Render, Window,
};

use crate::i18n::{self, MessageKey};

const GRAPH_MIN_WIDTH: f32 = 760.0;
const GRAPH_HEIGHT: f32 = 520.0;
const NODE_WIDTH: f32 = 178.0;
const NODE_HEIGHT: f32 = 62.0;
const GRAPH_MARGIN: f32 = 26.0;

const INCOMING_EDGE_COLOR: u32 = 0x8b5cf6;
const OUTGOING_EDGE_COLOR: u32 = 0x0ea5e9;

#[derive(Debug, Clone)]
pub enum RelationshipGraphEvent {
    Select(String),
}

#[derive(Debug, Clone)]
struct GraphNode {
    id: String,
    label: String,
    is_focus: bool,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

#[derive(Debug, Clone)]
struct GraphEdge {
    source: usize,
    target: usize,
    label: String,
    incoming: bool,
    lane: f32,
}

#[derive(Debug, Clone)]
struct GraphNodeDrag {
    id: String,
}

pub struct RelationshipGraph {
    focus_id: Option<String>,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    viewport_width: f32,
    viewport_height: f32,
    alpha: f32,
    physics_active: bool,
    dragged_node: Option<String>,
    drag_offset: (f32, f32),
}

impl RelationshipGraph {
    pub fn new() -> Self {
        Self {
            focus_id: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            viewport_width: GRAPH_MIN_WIDTH,
            viewport_height: GRAPH_HEIGHT,
            alpha: 0.0,
            physics_active: false,
            dragged_node: None,
            drag_offset: (0.0, 0.0),
        }
    }

    pub fn set_data(
        &mut self,
        items: &[Item],
        links: &[ItemLink],
        focus_id: Option<&str>,
        cx: &mut Context<Self>,
    ) {
        let Some(focus_id) = focus_id else {
            self.clear();
            cx.notify();
            return;
        };

        let items_by_id = items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect::<HashMap<_, _>>();
        if !items_by_id.contains_key(focus_id) {
            self.clear();
            cx.notify();
            return;
        }

        let mut visible_ids = vec![focus_id.to_owned()];
        let mut raw_edges = Vec::new();
        for link in links {
            let outgoing = link.item_id == focus_id;
            let incoming = link.target_id == focus_id;
            if !outgoing && !incoming {
                continue;
            }

            let neighbor_id = if outgoing {
                &link.target_id
            } else {
                &link.item_id
            };
            if neighbor_id == focus_id || !items_by_id.contains_key(neighbor_id.as_str()) {
                continue;
            }
            if !visible_ids.iter().any(|id| id == neighbor_id) {
                visible_ids.push(neighbor_id.clone());
            }
            raw_edges.push((
                link.item_id.clone(),
                link.target_id.clone(),
                link.r#type.clone(),
                incoming,
            ));
        }

        let mut nodes = visible_ids
            .iter()
            .filter_map(|id| {
                let item = items_by_id.get(id.as_str())?;
                Some(GraphNode {
                    id: id.clone(),
                    label: crate::item_display_value(item),
                    is_focus: id == focus_id,
                    x: 0.0,
                    y: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                })
            })
            .collect::<Vec<_>>();
        let node_indexes = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id.clone(), index))
            .collect::<HashMap<_, _>>();

        let mut edge_counts = HashMap::<(usize, usize), usize>::new();
        for (source, target, _, _) in &raw_edges {
            let (Some(&source), Some(&target)) =
                (node_indexes.get(source), node_indexes.get(target))
            else {
                continue;
            };
            *edge_counts
                .entry((source.min(target), source.max(target)))
                .or_default() += 1;
        }

        let mut edge_seen = HashMap::<(usize, usize), usize>::new();
        let mut edges = Vec::new();
        for (source, target, label, incoming) in raw_edges {
            let (Some(&source), Some(&target)) =
                (node_indexes.get(&source), node_indexes.get(&target))
            else {
                continue;
            };
            let key = (source.min(target), source.max(target));
            let seen = edge_seen.entry(key).or_default();
            let count = edge_counts[&key];
            let lane = (*seen as f32 - (count.saturating_sub(1) as f32 / 2.0)) * 16.0;
            *seen += 1;
            edges.push(GraphEdge {
                source,
                target,
                label,
                incoming,
                lane,
            });
        }

        self.focus_id = Some(focus_id.to_owned());
        self.nodes = std::mem::take(&mut nodes);
        self.edges = edges;
        self.dragged_node = None;
        self.reset_layout();
        cx.notify();
    }

    pub fn clear(&mut self) {
        self.focus_id = None;
        self.nodes.clear();
        self.edges.clear();
        self.dragged_node = None;
        self.physics_active = false;
        self.alpha = 0.0;
    }

    fn set_viewport(&mut self, width: f32, height: f32) -> bool {
        let width = width.max(NODE_WIDTH + GRAPH_MARGIN * 2.0);
        let height = height.max(NODE_HEIGHT + GRAPH_MARGIN * 2.0);
        if (self.viewport_width - width).abs() < 1.0 && (self.viewport_height - height).abs() < 1.0
        {
            return false;
        }

        self.viewport_width = width;
        self.viewport_height = height;
        if let Some(focus) = self.nodes.iter_mut().find(|node| node.is_focus) {
            focus.x = width / 2.0;
            focus.y = height / 2.0;
        }
        self.physics_active = self.nodes.len() > 1 && !self.edges.is_empty();
        self.alpha = if self.physics_active { 1.0 } else { 0.0 };
        true
    }

    fn reset_layout(&mut self) {
        let center_x = self.viewport_width / 2.0;
        let center_y = self.viewport_height / 2.0;
        let neighbor_count = self.nodes.len().saturating_sub(1);
        let radius = (self.viewport_width.min(self.viewport_height) * 0.4).clamp(128.0, 230.0);

        let mut neighbor_index = 0;
        for node in &mut self.nodes {
            node.vx = 0.0;
            node.vy = 0.0;
            if node.is_focus {
                node.x = center_x;
                node.y = center_y;
                continue;
            }

            let angle = -std::f32::consts::FRAC_PI_2
                + std::f32::consts::TAU * neighbor_index as f32 / neighbor_count.max(1) as f32;
            node.x = center_x + angle.cos() * radius;
            node.y = center_y + angle.sin() * radius;
            neighbor_index += 1;
        }

        self.dragged_node = None;
        self.alpha = if neighbor_count > 0 && !self.edges.is_empty() {
            1.0
        } else {
            0.0
        };
        self.physics_active = self.alpha > 0.0;
    }

    fn reset_layout_from_button(&mut self, cx: &mut Context<Self>) {
        self.reset_layout();
        cx.notify();
    }

    fn begin_drag(&mut self, id: String, offset: Point<Pixels>) {
        if self
            .nodes
            .iter()
            .find(|node| node.id == id)
            .is_some_and(|node| node.is_focus)
        {
            return;
        }
        self.dragged_node = Some(id);
        self.drag_offset = (f32::from(offset.x), f32::from(offset.y));
        self.physics_active = false;
        self.alpha = 0.0;
    }

    fn drag_move(&mut self, event: &DragMoveEvent<GraphNodeDrag>, cx: &mut Context<Self>) {
        let Some(id) = self.dragged_node.as_deref() else {
            return;
        };
        let pointer_x = f32::from(event.event.position.x - event.bounds.left());
        let pointer_y = f32::from(event.event.position.y - event.bounds.top());
        let offset = self.drag_offset;
        let width = self.viewport_width;
        let height = self.viewport_height;

        if let Some(node) = self.nodes.iter_mut().find(|node| node.id == id) {
            node.x = pointer_x - offset.0 + NODE_WIDTH / 2.0;
            node.y = pointer_y - offset.1 + NODE_HEIGHT / 2.0;
            node.x = clamp_coordinate(node.x, NODE_WIDTH / 2.0 + GRAPH_MARGIN, width);
            node.y = clamp_coordinate(node.y, NODE_HEIGHT / 2.0 + GRAPH_MARGIN, height);
            node.vx = 0.0;
            node.vy = 0.0;
            cx.notify();
        }
    }

    fn end_drag(&mut self) {
        self.dragged_node = None;
    }

    fn advance_physics(&mut self) {
        if !self.physics_active || self.nodes.len() < 2 {
            return;
        }

        let alpha = self.alpha;
        const REPULSION: f32 = 5200.0;
        const LINK_DISTANCE: f32 = 190.0;
        const LINK_STRENGTH: f32 = 0.055;
        const CENTER_STRENGTH: f32 = 0.012;
        const DAMPING: f32 = 0.72;

        for left in 0..self.nodes.len() {
            for right in (left + 1)..self.nodes.len() {
                let mut dx = self.nodes[right].x - self.nodes[left].x;
                let mut dy = self.nodes[right].y - self.nodes[left].y;
                if dx.abs() < 0.1 {
                    dx = if left % 2 == 0 { 0.1 } else { -0.1 };
                }
                if dy.abs() < 0.1 {
                    dy = if right % 2 == 0 { 0.1 } else { -0.1 };
                }
                let distance_squared = (dx * dx + dy * dy).max(64.0);
                let distance = distance_squared.sqrt();
                let force = REPULSION / distance_squared * alpha;
                let fx = dx / distance * force;
                let fy = dy / distance * force;

                if !self.nodes[left].is_focus {
                    self.nodes[left].vx -= fx;
                    self.nodes[left].vy -= fy;
                }
                if !self.nodes[right].is_focus {
                    self.nodes[right].vx += fx;
                    self.nodes[right].vy += fy;
                }
            }
        }

        for edge in &self.edges {
            let source_x = self.nodes[edge.source].x;
            let source_y = self.nodes[edge.source].y;
            let target_x = self.nodes[edge.target].x;
            let target_y = self.nodes[edge.target].y;
            let mut dx = target_x - source_x;
            let mut dy = target_y - source_y;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);
            dx /= distance;
            dy /= distance;
            let force = (distance - LINK_DISTANCE) * LINK_STRENGTH * alpha;
            let fx = dx * force;
            let fy = dy * force;

            if !self.nodes[edge.source].is_focus {
                self.nodes[edge.source].vx += fx;
                self.nodes[edge.source].vy += fy;
            }
            if !self.nodes[edge.target].is_focus {
                self.nodes[edge.target].vx -= fx;
                self.nodes[edge.target].vy -= fy;
            }
        }

        let center_x = self.viewport_width / 2.0;
        let center_y = self.viewport_height / 2.0;
        for node in &mut self.nodes {
            if node.is_focus {
                node.x = center_x;
                node.y = center_y;
                node.vx = 0.0;
                node.vy = 0.0;
                continue;
            }
            node.vx += (center_x - node.x) * CENTER_STRENGTH * alpha;
            node.vy += (center_y - node.y) * CENTER_STRENGTH * alpha;
            node.vx *= DAMPING;
            node.vy *= DAMPING;
            node.x += node.vx;
            node.y += node.vy;
            node.x = clamp_coordinate(node.x, NODE_WIDTH / 2.0 + GRAPH_MARGIN, self.viewport_width);
            node.y = clamp_coordinate(
                node.y,
                NODE_HEIGHT / 2.0 + GRAPH_MARGIN,
                self.viewport_height,
            );
        }

        self.alpha *= 0.94;
        if self.alpha < 0.012 {
            self.alpha = 0.0;
            self.physics_active = false;
        }
    }

    fn render_node(
        &self,
        node: &GraphNode,
        graph: Entity<Self>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let t = theme(cx);
        let node_id = node.id.clone();
        let title = node.label.clone();
        let primary = t.primary().hsla();
        let border = if node.is_focus {
            primary
        } else {
            t.border().hsla()
        };
        let background = if node.is_focus {
            t.primary().alpha(0.14).hsla()
        } else {
            t.surface().hsla()
        };
        let accent = if node.is_focus {
            primary
        } else if self
            .edges
            .iter()
            .any(|edge| edge.source == self.node_index(&node.id) && edge.incoming)
        {
            rgb(INCOMING_EDGE_COLOR).into()
        } else {
            rgb(OUTGOING_EDGE_COLOR).into()
        };
        let subtitle = if node.is_focus {
            i18n::text(MessageKey::RelationshipGraphCurrent)
        } else {
            i18n::text(MessageKey::RelationshipGraphNeighbor)
        };
        let drag_id = node_id.clone();
        let click_id = node_id.clone();
        let mut card = div()
            .id(format!("relationship-graph-node-{node_id}"))
            .absolute()
            .left(px(node.x - NODE_WIDTH / 2.0))
            .top(px(node.y - NODE_HEIGHT / 2.0))
            .w(px(NODE_WIDTH))
            .h(px(NODE_HEIGHT))
            .p(px(10.0))
            .flex()
            .flex_col()
            .justify_center()
            .gap(px(3.0))
            .rounded_lg()
            .border_1()
            .border_color(border)
            .bg(background)
            .shadow_sm()
            .cursor_grab()
            .hover(|style| style.border_color(primary))
            .role(gpui_kit::Role::Button)
            .aria_label(title.clone())
            .aria_selected(node.is_focus)
            .on_click(cx.listener(move |_, _, _, cx| {
                cx.emit(RelationshipGraphEvent::Select(click_id.clone()));
                cx.stop_propagation();
            }));

        if !node.is_focus {
            card = card.on_drag(
                GraphNodeDrag { id: drag_id },
                move |drag, offset, _window, cx| {
                    let graph = graph.clone();
                    let id = drag.id.clone();
                    graph.update(cx, |graph, _| graph.begin_drag(id, offset));
                    cx.new(|_| Empty)
                },
            );
        }

        card.child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(accent))
                .child(
                    div()
                        .min_w_0()
                        .text_sm()
                        .text_color(t.text().hsla())
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .child(title),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(t.dimmed().hsla())
                .child(subtitle),
        )
    }

    fn node_index(&self, id: &str) -> usize {
        self.nodes
            .iter()
            .position(|node| node.id == id)
            .unwrap_or_default()
    }
}

impl Default for RelationshipGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter<RelationshipGraphEvent> for RelationshipGraph {}

impl Render for RelationshipGraph {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.advance_physics();
        if self.physics_active {
            window.request_animation_frame();
        }

        let t = theme(cx);
        let graph = cx.entity().clone();
        let graph_for_measure = graph.clone();
        let nodes = self.nodes.clone();
        let edges = self.edges.clone();
        let border = t.border().hsla();
        let body = t.body().hsla();
        let surface = t.surface().hsla();
        let dimmed = t.dimmed().hsla();
        let text = t.text().hsla();
        let grid = t.border().alpha(0.28).hsla();
        let empty = self.nodes.len() <= 1 || self.edges.is_empty();

        let mut surface_element = div()
            .id("relationship-graph-surface")
            .relative()
            .w_full()
            .min_w(px(GRAPH_MIN_WIDTH))
            .h(px(GRAPH_HEIGHT))
            .overflow_hidden()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .bg(body)
            .on_drag_move(
                cx.listener(|graph, event: &DragMoveEvent<GraphNodeDrag>, _, cx| {
                    graph.drag_move(event, cx);
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|graph, _, _, _| graph.end_drag()),
            );

        let edge_nodes = nodes.clone();
        let edge_links = edges.clone();
        surface_element = surface_element.child(
            canvas(
                move |bounds, _, cx| {
                    graph_for_measure.update(cx, |graph, cx| {
                        if graph.set_viewport(
                            f32::from(bounds.size.width),
                            f32::from(bounds.size.height),
                        ) {
                            cx.notify();
                        }
                    });
                },
                move |bounds, _, window, _| {
                    paint_grid(bounds, window, grid);
                    paint_edges(bounds, &edge_nodes, &edge_links, window);
                },
            )
            .absolute()
            .inset_0(),
        );

        for edge in &edges {
            let source = &nodes[edge.source];
            let target = &nodes[edge.target];
            let dx = target.x - source.x;
            let dy = target.y - source.y;
            let distance = (dx * dx + dy * dy).sqrt().max(1.0);
            let normal_x = -dy / distance;
            let normal_y = dx / distance;
            let mid_x = (source.x + target.x) / 2.0 + normal_x * edge.lane;
            let mid_y = (source.y + target.y) / 2.0 + normal_y * edge.lane;
            let label_color = if edge.incoming {
                rgb(INCOMING_EDGE_COLOR)
            } else {
                rgb(OUTGOING_EDGE_COLOR)
            };
            surface_element = surface_element.child(
                div()
                    .absolute()
                    .left(px(mid_x - 36.0))
                    .top(px(mid_y - 9.0))
                    .w(px(72.0))
                    .h(px(18.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_full()
                    .bg(surface.alpha(0.92))
                    .text_xs()
                    .text_color(label_color)
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(edge.label.clone()),
            );
        }

        for node in &nodes {
            surface_element = surface_element.child(self.render_node(node, graph.clone(), cx));
        }

        if empty {
            surface_element = surface_element.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom(px(18.0))
                    .flex()
                    .justify_center()
                    .text_sm()
                    .text_color(dimmed)
                    .child(i18n::text(MessageKey::RelationshipGraphEmpty)),
            );
        }

        let reset_graph = graph.clone();
        let reset_action = Button::new(
            "relationship-graph-reset",
            i18n::text(MessageKey::RelationshipGraphReset),
        )
        .size(Size::Xs)
        .variant(Variant::Subtle)
        .on_click(move |_, _, cx| {
            reset_graph.update(cx, |graph, cx| graph.reset_layout_from_button(cx));
        });
        let legend = div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .text_xs()
            .text_color(dimmed)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(rgb(INCOMING_EDGE_COLOR)),
                    )
                    .child(i18n::text(MessageKey::IncomingRelationshipsLabel)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .w(px(7.0))
                            .h(px(7.0))
                            .rounded_full()
                            .bg(rgb(OUTGOING_EDGE_COLOR)),
                    )
                    .child(i18n::text(MessageKey::OutgoingRelationshipsLabel)),
            );

        div()
            .flex()
            .flex_col()
            .min_w(px(GRAPH_MIN_WIDTH))
            .gap(px(10.0))
            .text_color(text)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .child(i18n::text(MessageKey::DetailsRelationshipsTab)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(dimmed)
                                    .child(i18n::text(MessageKey::RelationshipGraphHint)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.0))
                            .child(legend)
                            .child(reset_action),
                    ),
            )
            .child(surface_element)
    }
}

fn clamp_coordinate(value: f32, minimum: f32, extent: f32) -> f32 {
    let maximum = (extent - minimum).max(minimum);
    value.clamp(minimum.min(maximum), maximum)
}

fn paint_grid(bounds: Bounds<Pixels>, window: &mut Window, color: gpui_kit::Hsla) {
    let width = f32::from(bounds.size.width);
    let height = f32::from(bounds.size.height);
    let step = 32.0;
    let mut x = step;
    while x < width {
        let mut path = PathBuilder::stroke(px(1.0));
        path.move_to(gpui_kit::point(bounds.origin.x + px(x), bounds.origin.y));
        path.line_to(gpui_kit::point(
            bounds.origin.x + px(x),
            bounds.origin.y + px(height),
        ));
        if let Ok(path) = path.build() {
            window.paint_path(path, color);
        }
        x += step;
    }
    let mut y = step;
    while y < height {
        let mut path = PathBuilder::stroke(px(1.0));
        path.move_to(gpui_kit::point(bounds.origin.x, bounds.origin.y + px(y)));
        path.line_to(gpui_kit::point(
            bounds.origin.x + px(width),
            bounds.origin.y + px(y),
        ));
        if let Ok(path) = path.build() {
            window.paint_path(path, color);
        }
        y += step;
    }
}

fn paint_edges(
    bounds: Bounds<Pixels>,
    nodes: &[GraphNode],
    edges: &[GraphEdge],
    window: &mut Window,
) {
    for edge in edges {
        let source = &nodes[edge.source];
        let target = &nodes[edge.target];
        let dx = target.x - source.x;
        let dy = target.y - source.y;
        let distance = (dx * dx + dy * dy).sqrt().max(1.0);
        let normal_x = -dy / distance;
        let normal_y = dx / distance;
        let source_center = GraphPoint {
            x: source.x + normal_x * edge.lane,
            y: source.y + normal_y * edge.lane,
        };
        let target_center = GraphPoint {
            x: target.x + normal_x * edge.lane,
            y: target.y + normal_y * edge.lane,
        };
        let start = rectangle_edge_point(source_center, target_center);
        let end = rectangle_edge_point(target_center, source_center);
        let origin = bounds.origin;
        let start = gpui_kit::point(origin.x + px(start.x), origin.y + px(start.y));
        let end = gpui_kit::point(origin.x + px(end.x), origin.y + px(end.y));
        let color = if edge.incoming {
            rgb(INCOMING_EDGE_COLOR)
        } else {
            rgb(OUTGOING_EDGE_COLOR)
        };

        let mut line = PathBuilder::stroke(px(1.7));
        line.move_to(start);
        line.line_to(end);
        if let Ok(path) = line.build() {
            window.paint_path(path, color);
        }

        let angle = dy.atan2(dx);
        let arrow_size = 7.0;
        let tip = end;
        let left = gpui_kit::point(
            tip.x - px(arrow_size * (angle - 0.48).cos()),
            tip.y - px(arrow_size * (angle - 0.48).sin()),
        );
        let right = gpui_kit::point(
            tip.x - px(arrow_size * (angle + 0.48).cos()),
            tip.y - px(arrow_size * (angle + 0.48).sin()),
        );
        let mut arrow = PathBuilder::fill();
        arrow.move_to(tip);
        arrow.line_to(left);
        arrow.line_to(right);
        arrow.close();
        if let Ok(path) = arrow.build() {
            window.paint_path(path, color);
        }
    }
}

#[derive(Clone, Copy)]
struct GraphPoint {
    x: f32,
    y: f32,
}

fn rectangle_edge_point(from: GraphPoint, to: GraphPoint) -> GraphPoint {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    if dx.abs() < 0.01 && dy.abs() < 0.01 {
        return from;
    }
    let scale_x = if dx.abs() > 0.01 {
        NODE_WIDTH / 2.0 / dx.abs()
    } else {
        f32::INFINITY
    };
    let scale_y = if dy.abs() > 0.01 {
        NODE_HEIGHT / 2.0 / dy.abs()
    } else {
        f32::INFINITY
    };
    let scale = scale_x.min(scale_y);
    GraphPoint {
        x: from.x + dx * scale,
        y: from.y + dy * scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a22_application::InfoEntry;
    use std::collections::HashSet;

    fn item(id: &str, title: &str) -> Item {
        Item {
            id: id.to_owned(),
            main_info_entry_id: Some(format!("{id}-title")),
            tags: Vec::new(),
            info_entries: vec![InfoEntry {
                id: format!("{id}-title"),
                label: "title".to_owned(),
                value: title.to_owned(),
            }],
            date_time_entries: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn rectangle_edge_point_stops_at_card_boundary() {
        let point = rectangle_edge_point(
            GraphPoint { x: 100.0, y: 100.0 },
            GraphPoint { x: 300.0, y: 100.0 },
        );
        assert_eq!(point.x, 189.0);
        assert_eq!(point.y, 100.0);
    }

    #[test]
    fn graph_projection_keeps_only_direct_neighbors() {
        let items = [
            item("center", "Center"),
            item("neighbor", "Neighbor"),
            item("far", "Far"),
        ];
        let links = [
            ItemLink {
                id: "center-neighbor".to_owned(),
                item_id: "center".to_owned(),
                target_id: "neighbor".to_owned(),
                r#type: "related".to_owned(),
                created_at: String::new(),
                updated_at: String::new(),
            },
            ItemLink {
                id: "neighbor-far".to_owned(),
                item_id: "neighbor".to_owned(),
                target_id: "far".to_owned(),
                r#type: "related".to_owned(),
                created_at: String::new(),
                updated_at: String::new(),
            },
        ];
        let items_by_id = items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect::<HashMap<_, _>>();
        let visible = links
            .iter()
            .filter(|link| link.item_id == "center" || link.target_id == "center")
            .flat_map(|link| [link.item_id.as_str(), link.target_id.as_str()])
            .filter(|id| items_by_id.contains_key(id))
            .collect::<HashSet<_>>();

        assert_eq!(visible, HashSet::from(["center", "neighbor"]));
        assert!(!visible.contains("far"));
    }
}
