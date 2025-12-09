use super::pane::PaneId;

/// Direction of a split
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, // panes stacked vertically (one above the other)
    Vertical,   // panes side by side
}

/// A rectangle representing a pane's screen area
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Split this rect horizontally (top/bottom)
    pub fn split_horizontal(&self, ratio: f32) -> (Rect, Rect) {
        let split_y = self.y + (self.height as f32 * ratio) as u16;
        let top = Rect::new(self.x, self.y, self.width, split_y - self.y);
        let bottom = Rect::new(
            self.x,
            split_y,
            self.width,
            self.height - (split_y - self.y),
        );
        (top, bottom)
    }

    /// Split this rect vertically (left/right)
    pub fn split_vertical(&self, ratio: f32) -> (Rect, Rect) {
        let split_x = self.x + (self.width as f32 * ratio) as u16;
        let left = Rect::new(self.x, self.y, split_x - self.x, self.height);
        let right = Rect::new(
            split_x,
            self.y,
            self.width - (split_x - self.x),
            self.height,
        );
        (left, right)
    }
}

/// A node in the layout tree
#[derive(Debug)]
pub enum LayoutNode {
    /// A leaf node containing a pane
    Pane(PaneId),
    /// A split containing two children
    Split {
        direction: SplitDirection,
        ratio: f32,
        first: Box<LayoutNode>,
        second: Box<LayoutNode>,
    },
}

impl LayoutNode {
    /// Calculate the rect for each pane in this subtree
    pub fn calculate_rects(&self, area: Rect) -> Vec<(PaneId, Rect)> {
        match self {
            LayoutNode::Pane(id) => vec![(*id, area)],
            LayoutNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let (first_area, second_area) = match direction {
                    SplitDirection::Horizontal => area.split_horizontal(*ratio),
                    SplitDirection::Vertical => area.split_vertical(*ratio),
                };
                let mut rects = first.calculate_rects(first_area);
                rects.extend(second.calculate_rects(second_area));
                rects
            }
        }
    }

    /// Find the pane ID at the given position in the tree (for focus cycling)
    pub fn collect_pane_ids(&self) -> Vec<PaneId> {
        match self {
            LayoutNode::Pane(id) => vec![*id],
            LayoutNode::Split { first, second, .. } => {
                let mut ids = first.collect_pane_ids();
                ids.extend(second.collect_pane_ids());
                ids
            }
        }
    }

    /// Remove a pane from the layout, returning the new root if it was removed
    pub fn remove_pane(self, target_id: PaneId) -> Option<LayoutNode> {
        match self {
            LayoutNode::Pane(id) if id == target_id => None,
            LayoutNode::Pane(_) => Some(self),
            LayoutNode::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let first_removed = first.remove_pane(target_id);
                let second_removed = (*second).remove_pane(target_id);

                match (first_removed, second_removed) {
                    (None, None) => None,
                    (Some(node), None) | (None, Some(node)) => Some(node),
                    (Some(f), Some(s)) => Some(LayoutNode::Split {
                        direction,
                        ratio,
                        first: Box::new(f),
                        second: Box::new(s),
                    }),
                }
            }
        }
    }
}

/// The layout manager
pub struct Layout {
    pub root: LayoutNode,
}

impl Layout {
    pub fn new(initial_pane: PaneId) -> Self {
        Self {
            root: LayoutNode::Pane(initial_pane),
        }
    }

    pub fn calculate_rects(&self, area: Rect) -> Vec<(PaneId, Rect)> {
        self.root.calculate_rects(area)
    }

    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.root.collect_pane_ids()
    }

    /// Split the given pane, returning the new pane's position in the tree
    pub fn split_pane(&mut self, pane_id: PaneId, new_pane_id: PaneId, direction: SplitDirection) {
        self.root = Self::split_node(
            std::mem::replace(&mut self.root, LayoutNode::Pane(0)),
            pane_id,
            new_pane_id,
            direction,
        );
    }

    fn split_node(
        node: LayoutNode,
        target_id: PaneId,
        new_id: PaneId,
        direction: SplitDirection,
    ) -> LayoutNode {
        match node {
            LayoutNode::Pane(id) if id == target_id => LayoutNode::Split {
                direction,
                ratio: 0.5,
                first: Box::new(LayoutNode::Pane(id)),
                second: Box::new(LayoutNode::Pane(new_id)),
            },
            LayoutNode::Pane(_) => node,
            LayoutNode::Split {
                direction: d,
                ratio,
                first,
                second,
            } => LayoutNode::Split {
                direction: d,
                ratio,
                first: Box::new(Self::split_node(*first, target_id, new_id, direction)),
                second: Box::new(Self::split_node(*second, target_id, new_id, direction)),
            },
        }
    }

    /// Add a pane to the left side of the entire layout
    pub fn add_left_pane(&mut self, new_pane_id: PaneId, ratio: f32) {
        let old_root = std::mem::replace(&mut self.root, LayoutNode::Pane(0));
        self.root = LayoutNode::Split {
            direction: SplitDirection::Vertical,
            ratio,
            first: Box::new(LayoutNode::Pane(new_pane_id)),
            second: Box::new(old_root),
        };
    }

    /// Remove a pane from the layout
    pub fn remove_pane(&mut self, pane_id: PaneId) -> bool {
        if let Some(new_root) =
            std::mem::replace(&mut self.root, LayoutNode::Pane(0)).remove_pane(pane_id)
        {
            self.root = new_root;
            true
        } else {
            false
        }
    }
}
