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
    Entry,
    EntryExt,
    EditableSignals
};
use gtk::Orientation::{Vertical, Horizontal};
use relm::{Relm, Update, Widget, WidgetTest};

#[derive(Clone)]
struct HourUnit {
    date_hour: String,
    content: String,
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
    selected: Option<(usize, usize)>,
    content: String
}

// Create the structure that holds the widgets used in the view.
#[derive(Clone)]
struct Widgets {
    week: gtk::Box,
    select_view: gtk::Box,
    hover_view: gtk::Box,
    hover_date_hour_label: Label,
    hover_content_label: Label,
    select_date_hour_label: Label,
    select_content_label: Label,
    window: Window,
    input: Entry
}

struct Win {
    model: Model,
    widgets: Widgets,
}

#[derive(Msg)]
enum Msg {
    MouseEnter(usize, usize),
    MouseExit,
    Select(usize, usize),
    Quit,
    Edit,
    Change
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
            selected: None,
            content: "".to_string()
        }
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::Change => {
                self.model.content = self.widgets.input.get_text()
                    .expect("get_text failed")
                    .chars()
                    .collect();                
            },
            Msg::Edit => {
                match self.model.selected {
                    Some((i, j)) => {
                        self.model.week[i][j] = HourUnit {
                            content: self.model.content.clone(),
                            date_hour: self.model.week[i][j].date_hour.clone()
                        };

                        self.widgets.select_content_label.set_text(&self.model.content);
                        
                        self.model.selected = None;
                        self.model.content = "".to_string();
                        self.widgets.input.set_text("");                    
                    }, None => {}
                }
            },
            Msg::MouseEnter(i, j) => {
                let c = &self.widgets.hover_view.get_children();                
                c[0].hide();
                c[1].show();
                c[2].show();

                let hour_unit = &self.model.week[i][j];
                self.widgets.hover_date_hour_label.set_text(&hour_unit.date_hour[..]);
                self.widgets.hover_content_label.set_text(&hour_unit.content[..]);
            },
            Msg::MouseExit => {
                let c = &self.widgets.hover_view.get_children();
                c[0].show();
                c[1].hide();
                c[2].hide();
            },
            Msg::Select(i, j) => {
                let c = &self.widgets.select_view.get_children();

                //Todo: Only have to do this once
                c[0].hide();
                c[1].show();//date hour label
                c[2].show();//content label
                c[3].show();//input entry
                c[4].show();//edit button

                let hour_unit = &self.model.week[i][j];
                self.widgets.select_date_hour_label.set_text(&hour_unit.date_hour[..]);
                self.widgets.select_content_label.set_text(&hour_unit.content[..]);

                self.model.selected = Some((i, j));
            }
        }
    }
}

fn week_view(relm: &Relm<Win>, week: &Vec<Vec<HourUnit>>) -> gtk::Box {
    let week_buttons = gtk::Box::new(Horizontal, 0);
    for i in 0..5 {
        let day = gtk::Box::new(Vertical, 0);
        for j in 0..13 {
            let button = Button::new_with_label(&week[i][j].date_hour);
            day.add(&button);
            connect!(relm, button, connect_clicked(_), Msg::Select(i, j));
            connect!(relm, button, connect_enter_notify_event(_,_), return (Some(Msg::MouseEnter(i, j)), Inhibit(false)));
        }
        week_buttons.add(&day);
    }
    week_buttons
}

fn edit_view(relm: &Relm<Win>) -> (gtk::Box, gtk::Box, gtk::Box, Label, Label, Label, Label, Entry) {
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
    hover_view.add(&Label::new("Hover over a time to view"));
    
    let hover_date_hour_label = Label::new(None);
    let hover_content_label = Label::new(None);
    hover_view.add(&hover_date_hour_label);
    hover_view.add(&hover_content_label);
    edit_view.add(&hover_view);
    
    let select_view = gtk::Box::new(Vertical, 0);
    select_view.add(&Label::new("Click a time to edit"));

    let select_date_hour_label = Label::new(None);
    let select_content_label = Label::new(None);
    
    select_view.add(&select_date_hour_label);
    select_view.add(&select_content_label);

    let input = Entry::new();
    select_view.add(&input);
    connect!(relm, input, connect_changed(_), Msg::Change);

    let edit = Button::new_with_label("Edit");
    select_view.add(&edit);
    connect!(relm, edit, connect_clicked(_), Msg::Edit);
    
    edit_view.add(&select_view);    

    (edit_view, hover_view, select_view, hover_date_hour_label, hover_content_label, select_date_hour_label, select_content_label, input)
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

        let w_view = week_view(relm, &model.week);
        layout.add(&w_view);
        connect!(relm, w_view, connect_leave_notify_event(_,_), return (Some(Msg::MouseExit), Inhibit(false)));        
        
        let (e_view, hover_view, select_view, hover_date_hour_label, hover_content_label, select_date_hour_label, select_content_label, input) = edit_view(relm);
        layout.add(&e_view);
        window.add(&layout);

        window.show_all();

        let c = select_view.get_children();
        c[0].show();
        c[1].hide();
        c[2].hide();
        c[3].hide();
        c[4].hide();

        // Send the message Increment when the button is clicked.
        connect!(relm, window, connect_delete_event(_, _), return (Some(Msg::Quit), Inhibit(false)));

        Win {
            model,
            widgets: Widgets {
                hover_view: hover_view,
                select_view: select_view,
                hover_date_hour_label: hover_date_hour_label,
                hover_content_label: hover_content_label,
                select_date_hour_label: select_date_hour_label,
                select_content_label: select_content_label,
                input: input,
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
