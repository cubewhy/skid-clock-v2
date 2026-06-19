use embedded_graphics::{
    geometry::{Point, Size},
    primitives::Rectangle,
};

use std::vec::Vec;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn offset(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }

    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width as i32
            && point.y >= self.y
            && point.y < self.y + self.height as i32
    }

    pub fn to_rectangle(self) -> Rectangle {
        Rectangle::new(
            Point::new(self.x, self.y),
            Size::new(self.width, self.height),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Stretch,
    Center,
}

pub struct FlexNode<'a> {
    direction: FlexDirection,
    align_items: AlignItems,
    flex_grow: u32,
    fixed_width: Option<u32>,
    fixed_height: Option<u32>,
    children: Vec<FlexNode<'a>>,

    output: Option<&'a mut Rect>,
}

impl<'a> FlexNode<'a> {
    pub fn new(direction: FlexDirection) -> Self {
        Self {
            direction,
            align_items: AlignItems::Stretch,
            flex_grow: 0,
            fixed_width: None,
            fixed_height: None,
            children: Vec::new(),
            output: None,
        }
    }

    pub fn assign_to(mut self, rect: &'a mut Rect) -> Self {
        self.output = Some(rect);
        self
    }

    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }

    pub fn with_flex(mut self, grow: u32) -> Self {
        self.flex_grow = grow;
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.fixed_width = Some(width);
        self.fixed_height = Some(height);
        self
    }

    pub fn child(mut self, child: FlexNode<'a>) -> Self {
        self.children.push(child);
        self
    }

    pub fn layout(mut self, bounds: Rect) {
        self.layout_internal(bounds);
    }

    fn layout_internal(&mut self, bounds: Rect) {
        if let Some(ref mut out) = self.output {
            **out = bounds;
        }

        if self.children.is_empty() {
            return;
        }

        let total_flex: u32 = self.children.iter().map(|c| c.flex_grow).sum();
        let mut fixed_main_axis_sum = 0;

        for child in &self.children {
            if child.flex_grow == 0 {
                fixed_main_axis_sum += match self.direction {
                    FlexDirection::Row => child.fixed_width.unwrap_or(0),
                    FlexDirection::Column => child.fixed_height.unwrap_or(0),
                };
            }
        }

        let main_axis_total = match self.direction {
            FlexDirection::Row => bounds.width,
            FlexDirection::Column => bounds.height,
        };

        let remaining_space = main_axis_total.saturating_sub(fixed_main_axis_sum);
        let mut current_main_offset = 0;

        for child in &mut self.children {
            let main_size = if child.flex_grow > 0 && total_flex > 0 {
                (remaining_space * child.flex_grow) / total_flex
            } else {
                match self.direction {
                    FlexDirection::Row => child.fixed_width.unwrap_or(0),
                    FlexDirection::Column => child.fixed_height.unwrap_or(0),
                }
            };

            let cross_max = match self.direction {
                FlexDirection::Row => bounds.height,
                FlexDirection::Column => bounds.width,
            };

            let cross_size = match self.direction {
                FlexDirection::Row => child.fixed_height.unwrap_or(cross_max),
                FlexDirection::Column => child.fixed_width.unwrap_or(cross_max),
            };

            let cross_offset = match self.align_items {
                AlignItems::Stretch => 0,
                AlignItems::Center => ((cross_max.saturating_sub(cross_size)) / 2) as i32,
            };

            let child_rect = match self.direction {
                FlexDirection::Row => Rect::new(
                    bounds.x + current_main_offset,
                    bounds.y + cross_offset,
                    main_size,
                    cross_size,
                ),
                FlexDirection::Column => Rect::new(
                    bounds.x + cross_offset,
                    bounds.y + current_main_offset,
                    cross_size,
                    main_size,
                ),
            };

            child.layout_internal(child_rect);
            current_main_offset += main_size as i32;
        }
    }
}
