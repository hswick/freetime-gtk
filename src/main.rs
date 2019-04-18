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
extern crate chrono;

use serde::{Serialize, Deserialize};

use std::fs;

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

use chrono::{Local, Weekday, NaiveDate, Datelike};

#[derive(Clone,Serialize, Deserialize)]
struct HourUnit {
    date_hour: String,
    content: String,
    day: u32
}

struct Model {
    week: Vec<Vec<HourUnit>>,
    selected: Option<(usize, usize)>,
    content: String,
    selected_date: NaiveDate,
    today: NaiveDate
}

// Create the structure that holds the widgets used in the view.
#[derive(Clone)]
struct Widgets {
    week: Vec<Vec<gtk::Button>>,
    hover_view: gtk::Box,
    hover_date_hour_label: Label,
    hover_content_label: Label,
    select_view: gtk::Box,
    select_date_hour_label: Label,
    select_content_label: Label,
    input: Entry,
    window: Window    
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
    Change,
    Last,
    Current,
    Next
}

fn week_file_path(date: &NaiveDate) -> String {
    let week = date.iso_week().week();
    let monday = NaiveDate::from_isoywd(date.year(), week, Weekday::Mon);
    let sunday = NaiveDate::from_isoywd(date.year(), week, Weekday::Sun);
    format!(".freetime/{}_{}_{}-{}_{}_{}.json", monday.month(), monday.day(), monday.year(), sunday.month(), sunday.day(), sunday.year())    
}

fn init_week(date: &NaiveDate) -> Vec<Vec<HourUnit>> {
    let y = date.year();
    let w = date.iso_week().week();

    let week = vec![NaiveDate::from_isoywd(y, w, Weekday::Mon),
                    NaiveDate::from_isoywd(y, w, Weekday::Tue),
                    NaiveDate::from_isoywd(y, w, Weekday::Wed),
                    NaiveDate::from_isoywd(y, w, Weekday::Thu),
                    NaiveDate::from_isoywd(y, w, Weekday::Fri),
                    NaiveDate::from_isoywd(y, w, Weekday::Sat),
                    NaiveDate::from_isoywd(y, w, Weekday::Sun)];

    week.iter().map(|d|
                    (8..21).map(|h|
                                HourUnit {
                                    date_hour: format!("{}/{}/{} {}:00", d.month(), d.day(), y, h),
                                    content: "".to_string(),
                                    day: d.day()
                                }
                    ).collect()

    ).collect()
}

fn get_week(date: &NaiveDate) -> Vec<Vec<HourUnit>> {
    let path = week_file_path(date);
    match fs::read(path) {
        Ok(s) => {
            serde_json::from_slice(&s).unwrap()
        },
        Err(_) => {
            init_week(date)
        }
    }
}

fn refresh_grid(model: &Model, grid: &Vec<Vec<Button>>) {
    for i in 0..model.week.len() {
        for j in 0..model.week[i].len() {
            let mut label = "-----";

            if model.week[i][j].day == model.today.day() {
                label = "+++++";
            }

            if &model.week[i][j].content != "" {
                label = ":::::";
            }            
            
            grid[i][j].set_label(label);
        }
    }
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
        let today = Local::now().date().naive_local();
        Model {
            week: get_week(&today),
            selected: None,
            content: "".to_string(),
            selected_date: today.clone(),
            today: today
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
                            date_hour: self.model.week[i][j].date_hour.clone(),
                            day: self.model.week[i][j].day.clone()
                        };

                        self.widgets.select_content_label.set_text(&self.model.content);
                        
                        self.model.content = "".to_string();
                        self.widgets.input.set_text("");

                        let path = week_file_path(&self.model.selected_date);
                        fs::write(path, serde_json::to_string(&self.model.week).unwrap()).unwrap();

                        refresh_grid(&self.model, &self.widgets.week);
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
                c[0].hide();
                c[1].show();//date hour label
                c[2].show();//content label
                c[3].show();//input entry
                c[4].show();//edit button

                let hour_unit = &self.model.week[i][j];
                self.widgets.select_date_hour_label.set_text(&hour_unit.date_hour[..]);
                self.widgets.select_content_label.set_text(&hour_unit.content[..]);

                self.model.selected = Some((i, j));
            },
            Msg::Last => {
                let selected = self.model.selected_date;

                let last_sunday = NaiveDate::from_isoywd(selected.year(), selected.iso_week().week(), Weekday::Mon).pred();
                let last_monday = NaiveDate::from_isoywd(selected.year(), last_sunday.iso_week().week(), Weekday::Mon);

                self.model.week = get_week(&last_monday);
                self.model.selected_date = last_monday;

                refresh_grid(&self.model, &self.widgets.week);                

            },
            Msg::Current => {
                let today = Local::now().date().naive_local();
                self.model.week = get_week(&today);
                self.model.selected_date = today;

                refresh_grid(&self.model, &self.widgets.week);                
            },
            Msg::Next => {
                let selected = self.model.selected_date;

                let next_monday = NaiveDate::from_isoywd(selected.year(), selected.iso_week().week(), Weekday::Sun).succ();

                self.model.week = get_week(&next_monday);
                self.model.selected_date = next_monday;

                refresh_grid(&self.model, &self.widgets.week);                
            }
        }
    }
}

fn week_view(relm: &Relm<Win>, model: &Model) -> (gtk::Box, Vec<Vec<Button>>) {
    let w_view = gtk::Box::new(Horizontal, 0);
    let mut week_buttons = Vec::new();
    for i in 0..7 {
        let day = gtk::Box::new(Vertical, 0);
        let mut col = Vec::new();
        for j in 0..13 {
            
            let hour_unit = &model.week[i][j];
            let mut label = "-----";
            
            if hour_unit.day == model.today.day() {
                label = "+++++";
            }

            if &hour_unit.content != "" {
                label = ":::::";
            }

            let button = Button::new_with_label(label);
                        
            day.add(&button);
            
            connect!(relm, button, connect_clicked(_), Msg::Select(i, j));
            connect!(relm, button, connect_enter_notify_event(_,_), return (Some(Msg::MouseEnter(i, j)), Inhibit(false)));

            col.push(button);
        }
        w_view.add(&day);
        week_buttons.push(col);
    }
    (w_view, week_buttons)
}

fn week_select_view(relm: &Relm<Win>) -> gtk::Box {
    let w_view = gtk::Box::new(Horizontal, 0);
    let last_button = Button::new_with_label("Last");
    connect!(relm, last_button, connect_clicked(_), Msg::Last);
    w_view.add(&last_button);

    let current_button = Button::new_with_label("Current");
    connect!(relm, current_button, connect_clicked(_), Msg::Current);
    w_view.add(&current_button);

    let next_button = Button::new_with_label("Next");
    connect!(relm, next_button, connect_clicked(_), Msg::Next);
    w_view.add(&next_button);

    w_view
}

fn hover_view() -> (gtk::Box, Label, Label) {
    let h_view = gtk::Box::new(Vertical, 0);
    h_view.add(&Label::new("Hover over a time to view"));
    
    let hover_date_hour_label = Label::new(None);
    let hover_content_label = Label::new(None);
    h_view.add(&hover_date_hour_label);
    h_view.add(&hover_content_label);

    (h_view, hover_date_hour_label, hover_content_label)
}

fn select_view(relm: &Relm<Win>) -> (gtk::Box, Label, Label, Entry) {
    let s_view = gtk::Box::new(Vertical, 0);
    s_view.add(&Label::new("Click a time to edit"));

    let select_date_hour_label = Label::new(None);
    let select_content_label = Label::new(None);
    
    s_view.add(&select_date_hour_label);
    s_view.add(&select_content_label);

    let input = Entry::new();
    s_view.add(&input);
    connect!(relm, input, connect_changed(_), Msg::Change);

    let edit = Button::new_with_label("Edit");
    s_view.add(&edit);
    connect!(relm, edit, connect_clicked(_), Msg::Edit);

    (s_view, select_date_hour_label, select_content_label, input)
}

fn edit_view(relm: &Relm<Win>) -> (gtk::Box, gtk::Box, gtk::Box, Label, Label, Label, Label, Entry) {
    let edit_view = gtk::Box::new(Vertical, 0);

    edit_view.add(&week_select_view(relm));

    let (h_view, hover_date_hour_label, hover_content_label) = hover_view();
    edit_view.add(&h_view);

    let (s_view, select_date_hour_label, select_content_label, input) = select_view(relm);
        
    edit_view.add(&s_view);

    (edit_view, h_view, s_view, hover_date_hour_label, hover_content_label, select_date_hour_label, select_content_label, input)
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

        let (w_view, week) = week_view(relm, &model);
        layout.add(&w_view);
        connect!(relm, w_view, connect_leave_notify_event(_,_), return (Some(Msg::MouseExit), Inhibit(false)));        
        
        let (
            e_view,
            h_view,
            s_view,
            hover_date_hour_label,
            hover_content_label,
            select_date_hour_label,
            select_content_label,
            input
        ) = edit_view(relm);
        
        layout.add(&e_view);
        window.add(&layout);

        window.show_all();

        let c = s_view.get_children();
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
                hover_date_hour_label: hover_date_hour_label,
                hover_content_label: hover_content_label,
                select_date_hour_label: select_date_hour_label,
                select_content_label: select_content_label,
                input: input,
                week: week,
                hover_view: h_view,
                select_view: s_view,
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

fn setup_freetime_dir() {
    match std::fs::create_dir(".freetime") {
        Ok(_) => {
            println!(".freetime directory has been created");
        },
        Err(e) => {
            println!("Unable to create .freetime directory {:?}", e.kind ());
        }
    }    
}

fn main() {
    setup_freetime_dir();
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
