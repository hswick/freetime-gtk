/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

extern crate gtk;
#[macro_use]
extern crate relm;
#[macro_use]
extern crate relm_derive;
#[cfg_attr(test, macro_use)]
extern crate gtk_test;

use gtk::{
    Button,
    ButtonExt,
    ContainerExt,
    Inhibit,
    Label,
    LabelExt,
    WidgetExt,
    Window,
    WindowType,
};
use gtk::Orientation::{Vertical, Horizontal};
use relm::{Relm, Update, Widget, WidgetTest};

struct HourUnit {
    date_hour: String,
    content: String
}

fn default_week() -> Vec<Vec<HourUnit>> {
    let mut week = Vec::new();
    for _ in 0..5 {
        let mut day = Vec::new();
        for hour in 8..21 {
            day.push(HourUnit {
                date_hour: format!("{:?}:00", hour),
                content: "".to_string()
            });
        }
        week.push(day);
    }
    week
}

struct Model {
    week: Vec<Vec<HourUnit>>,
    hovered: Option<HourUnit>,
    selected: Option<HourUnit>,    
}

#[derive(Msg)]
enum Msg {
    Quit,
}

// Create the structure that holds the widgets used in the view.
#[derive(Clone)]
struct Widgets {
    week: gtk::Box,
    window: Window,
}

struct Win {
    model: Model,
    widgets: Widgets,
}

impl Update for Win {
    // Specify the model used for this widget.
    type Model = Model;
    // Specify the model parameter used to init the model.
    type ModelParam = ();
    // Specify the type of the messages sent to the update function.
    type Msg = Msg;

    // Specify the initial state of the model
    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {
            week: default_week(),
            hovered: None,
            selected: None
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
        }
    }
}

fn week_view(week: &Vec<Vec<HourUnit>>) -> gtk::Box {
    let week_buttons = gtk::Box::new(Horizontal, 0);
    for i in 0..5 {
        let day = gtk::Box::new(Vertical, 0);
        for j in 0..13 {
            day.add(&Button::new_with_label(&week[i][j].date_hour));
        }
        week_buttons.add(&day);
    }
    week_buttons
}

impl Widget for Win {
    // Specify the type of the root widget.
    type Root = Window;

    // Return the root widget.
    fn root(&self) -> Self::Root {
        self.widgets.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        // Create the view using the normal GTK+ method calls.
        let window = Window::new(WindowType::Toplevel);
        let layout = gtk::Box::new(Horizontal, 0);

        let w_view = week_view(&model.week);
        layout.add(&w_view);

        let edit_view = gtk::Box::new(Vertical, 0);

        let week_select_view = gtk::Box::new(Horizontal, 0);
        let last_button = Button::new_with_label("Last");
        week_select_view.add(&last_button);

        let current_button = Button::new_with_label("Current");
        week_select_view.add(&current_button);

        let next_button = Button::new_with_label("Next");
        week_select_view.add(&next_button);

        edit_view.add(&week_select_view);        

        let hover_view = gtk::Box::new(Vertical, 0);
        match &model.hovered {
            Some(hu) => {
                hover_view.add(&Label::new(Some(&hu.date_hour[..])));
                hover_view.add(&Label::new(Some(&hu.content[..])));
            },
            None => {
                hover_view.add(&Label::new(Some("Hover over a time to view")));
            }
        };
        edit_view.add(&hover_view);

        let select_view = gtk::Box::new(Vertical, 0);
        match &model.selected {
            Some(hu) => {
                select_view.add(&Label::new(Some(&hu.date_hour[..])));
                select_view.add(&Label::new(Some(&hu.content[..])));
                select_view.add(&Button::new_with_label("Edit"));
            },
            None => {
                select_view.add(&Label::new(Some("Click a time to edit")));
            }
        };
        edit_view.add(&select_view);

        layout.add(&edit_view);
        window.add(&layout);

        window.show_all();

        // Send the message Increment when the button is clicked.
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));        
        //connect!(relm, plus_button, connect_clicked(_), Msg::Increment);
        //connect!(relm, minus_button, connect_clicked(_), Msg::Decrement);
        

        Win {
            model,
            widgets: Widgets {
                week: w_view,
                window: window,
            },
        }
    }
}

impl WidgetTest for Win {
    type Widgets = Widgets;

    fn get_widgets(&self) -> Self::Widgets {
        self.widgets.clone()
    }
}

fn main() {
    Win::run(()).expect("Win::run failed");
}

#[cfg(test)]
mod tests {
    use gtk::LabelExt;

    use relm;
    use gtk_test::click;

    use Win;

    #[test]
    fn label_change() {
        let (_component, widgets) = relm::init_test::<Win>(()).expect("init_test failed");
        let plus_button = &widgets.plus_button;
        let minus_button = &widgets.minus_button;
        let label = &widgets.counter_label;

        assert_text!(label, 0);
        click(plus_button);
        assert_text!(label, 1);
        click(plus_button);
        assert_text!(label, 2);
        click(plus_button);
        assert_text!(label, 3);
        click(plus_button);
        assert_text!(label, 4);

        click(minus_button);
        assert_text!(label, 3);
        click(minus_button);
        assert_text!(label, 2);
        click(minus_button);
        assert_text!(label, 1);
        click(minus_button);
        assert_text!(label, 0);
        click(minus_button);
        assert_text!(label, -1);
    }
}
