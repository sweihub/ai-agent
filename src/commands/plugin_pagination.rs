use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_MAX_VISIBLE: usize = 5;

pub struct PaginationOptions {
    pub total_items: usize,
    pub max_visible: Option<usize>,
    pub selected_index: Option<usize>,
}

pub struct PaginationResult<T> {
    pub current_page: usize,
    pub total_pages: usize,
    pub start_index: usize,
    pub end_index: usize,
    pub needs_pagination: bool,
    pub page_size: usize,
    pub get_visible_items: Box<dyn Fn(&[T]) -> Vec<T>>,
    pub to_actual_index: Box<dyn Fn(usize) -> usize>,
    pub is_on_current_page: Box<dyn Fn(usize) -> bool>,
    pub go_to_page: Box<dyn Fn(usize)>,
    pub next_page: Box<dyn Fn()>,
    pub prev_page: Box<dyn Fn()>,
    pub handle_selection_change: Box<dyn Fn(usize, &mut usize)>,
    pub handle_pagenavigation: Box<dyn Fn(&str, &mut usize) -> bool>,
    pub scroll_position: ScrollPosition,
}

pub struct ScrollPosition {
    pub current: usize,
    pub total: usize,
    pub can_scroll_up: bool,
    pub can_scroll_down: bool,
}

pub fn use_pagination<T: Clone>(options: PaginationOptions) -> PaginationResult<T> {
    let total_items = options.total_items;
    let max_visible = options.max_visible.unwrap_or(DEFAULT_MAX_VISIBLE);
    let selected_index = options.selected_index.unwrap_or(0);

    let needs_pagination = total_items > max_visible;

    let scroll_offset_ref: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));

    let scroll_offset = if !needs_pagination {
        0
    } else {
        let prev_offset = *scroll_offset_ref.borrow();

        if selected_index < prev_offset {
            *scroll_offset_ref.borrow_mut() = selected_index;
            selected_index
        } else if selected_index >= prev_offset + max_visible {
            let new_offset = selected_index - max_visible + 1;
            *scroll_offset_ref.borrow_mut() = new_offset;
            new_offset
        } else {
            let max_offset = total_items.saturating_sub(max_visible);
            let clamped_offset = prev_offset.min(max_offset);
            *scroll_offset_ref.borrow_mut() = clamped_offset;
            clamped_offset
        }
    };

    let start_index = scroll_offset;
    let end_index = (scroll_offset + max_visible).min(total_items);

    let needs_pagination_clone = needs_pagination;
    let start_index_clone = start_index;
    let end_index_clone = end_index;
    let get_visible_items = Box::new(move |items: &[T]| -> Vec<T> {
        if !needs_pagination_clone {
            items.to_vec()
        } else {
            items[start_index_clone..end_index_clone].to_vec()
        }
    });

    let start_index_for_actual = start_index;
    let to_actual_index =
        Box::new(move |visible_index: usize| -> usize { start_index_for_actual + visible_index });

    let start_index_for_page = start_index;
    let end_index_for_page = end_index;
    let is_on_current_page = Box::new(move |actual_index: usize| -> bool {
        actual_index >= start_index_for_page && actual_index < end_index_for_page
    });

    let go_to_page = Box::new(|_page: usize| {});
    let next_page = Box::new(|| {});
    let prev_page = Box::new(|| {});

    let total_items_for_selection = total_items;
    let handle_selection_change = Box::new(move |new_index: usize, selected: &mut usize| {
        let clamped_index = new_index.min(total_items_for_selection.saturating_sub(1));
        *selected = clamped_index;
    });

    let handle_page_navigation =
        Box::new(|_direction: &str, _set_selected_index: &mut usize| -> bool { false });

    let total_pages = ((total_items + max_visible - 1) / max_visible).max(1);
    let current_page = scroll_offset / max_visible;

    PaginationResult {
        current_page,
        total_pages,
        start_index,
        end_index,
        needs_pagination,
        page_size: max_visible,
        get_visible_items,
        to_actual_index,
        is_on_current_page,
        go_to_page,
        next_page,
        prev_page,
        handle_selection_change,
        handle_pagenavigation: handle_page_navigation,
        scroll_position: ScrollPosition {
            current: selected_index + 1,
            total: total_items,
            can_scroll_up: scroll_offset > 0,
            can_scroll_down: scroll_offset + max_visible < total_items,
        },
    }
}
