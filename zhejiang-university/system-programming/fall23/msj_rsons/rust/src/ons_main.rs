use crate::ons_button::OnsButton;
use crate::ons_parser::*;
use godot::engine::global::Key;
use godot::engine::image::Format;
use godot::engine::{
    DisplayServer, Image, ImageTexture, InputEvent, InputEventKey, Label, Sprite2D, Texture2D,
};
use godot::prelude::*;
use regex::Regex;
use std::collections::HashMap;
use std::fs::read_to_string;

// Deriving GodotClass makes the class available to Godot
#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct OnsMain {
    script: Script,
    labels: HashMap<String, usize>,
    alias: HashMap<String, String>,
    int_vars: HashMap<String, i64>,
    str_vars: HashMap<String, String>,
    current_line: usize,
    defined: bool,
    wait_for_key: bool,
    wait_for_button: bool,
    clear_text: bool,
    line_limit: usize,
    size: Vector2i,
    btndef: Gd<Texture2D>,
    btns: Vec<Gd<OnsButton>>,
    btn_var: String,
    lsp: HashMap<i64, Gd<OnsButton>>,
    #[base]
    base: Base<Node2D>,
}

#[godot_api]
impl INode2D for OnsMain {
    fn init(base: Base<Node2D>) -> Self {
        if let Some(script) = read_to_string("./1.txt").ok().and_then(|f| parse(&f)) {
            godot_print!("Main init!");
            let mut labels = HashMap::new();
            for (i, line) in script.stmts.iter().enumerate() {
                match line {
                    Stmt::StmtLabel(s) => {
                        if s.starts_with('*') {
                            labels.insert(s.clone(), i);
                        } else {
                            labels.insert(s.clone() + ":" + &i.to_string(), i);
                        }
                    }
                    _ => {}
                }
            }
            godot_print!("{:#?}", labels);
            let mut s = OnsMain {
                script: script.clone(),
                labels: labels.clone(),
                alias: HashMap::new(),
                int_vars: HashMap::new(),
                str_vars: HashMap::new(),
                current_line: 0,
                defined: false,
                wait_for_key: false,
                wait_for_button: false,
                clear_text: false,
                line_limit: 30,
                size: Vector2i::new(800, 600),
                btndef: Gd::default(),
                btns: Vec::new(),
                btn_var: String::new(),
                lsp: HashMap::new(),
                base,
            };
            if let Some(current_line) = labels.get("*define") {
                s.current_line = *current_line;
            } else {
                godot_error!("Err:define label not found");
            }
            while s.current_line < s.script.stmts.len() && !s.defined {
                s.step();
            }
            if let Some(current_line) = labels.get("*start") {
                s.current_line = *current_line;
            } else {
                godot_error!("Err:start label not found");
            }
            godot_print!("alias: {:?}", s.alias);
            return s;
        }
        godot_error!("Err:init script");
        OnsMain {
            script: Script { stmts: Vec::new() },
            labels: HashMap::new(),
            alias: HashMap::new(),
            int_vars: HashMap::new(),
            str_vars: HashMap::new(),
            current_line: 0,
            defined: false,
            wait_for_key: false,
            wait_for_button: false,
            clear_text: false,
            line_limit: 30,
            size: Vector2i::new(800, 600),
            btndef: Gd::default(),
            btns: Vec::new(),
            btn_var: String::new(),
            lsp: HashMap::new(),
            base,
        }
    }

    fn process(&mut self, _delta: f64) {
        if !self.wait_for_key && !self.wait_for_button {
            self.step();
        }
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Ok(event) = event.clone().try_cast::<InputEventKey>() {
            if self.wait_for_key && event.get_keycode() == Key::KEY_ENTER && event.is_pressed() {
                self.wait_for_key = false;
            }
        }
    }

    fn ready(&mut self) {}
}

#[godot_api]
impl OnsMain {
    fn get_int_expr(&self, expr: &ExprInt) -> Option<i64> {
        match expr {
            ExprInt::Number(i) => Some(*i),
            ExprInt::VarInt(_s) => {
                if let Some(addr) = self.get_int_addr(expr) {
                    self.int_vars.get(&addr).cloned()
                } else {
                    None
                }
            }
            ExprInt::Operation(a, op, b) => {
                let a = self.get_int_expr(a);
                let b = self.get_int_expr(b);
                if a.is_none() || b.is_none() {
                    None
                } else {
                    let a = a.unwrap_or_default();
                    let b = b.unwrap_or_default();
                    Some(match op.as_str() {
                        "+" => a + b,
                        "-" => a - b,
                        "*" => a * b,
                        "/" => a / b,
                        "%" => a % b,
                        "&&" => ((a != 0) && (b != 0)) as i64,
                        ">=" => (a >= b) as i64,
                        "<=" => (a <= b) as i64,
                        ">" => (a > b) as i64,
                        "<" => (a < b) as i64,
                        "<>" => (a != b) as i64,
                        "==" => (a == b) as i64,
                        _ => return None,
                    })
                }
            }
        }
    }

    fn get_str_expr(&self, expr: &ExprStr) -> Option<String> {
        match expr {
            ExprStr::String(s) => Some(s.clone()),
            ExprStr::VarStr(_s) => {
                if let Some(addr) = self.get_str_addr(expr) {
                    self.str_vars.get(&addr).cloned()
                } else {
                    None
                }
            }
        }
    }

    fn get_int_value(&self, value: &Value) -> Option<i64> {
        match value {
            Value::Int(i) => self.get_int_expr(i),
            _ => None,
        }
    }

    fn get_str_value(&self, value: &Value) -> Option<String> {
        match value {
            Value::Str(s) => self.get_str_expr(s).map(|s| s.replace('\"', "")),
            Value::Keyword(s) => Some(s.clone().replace('\"', "")),
            _ => None,
        }
    }

    fn get_int_addr(&self, value: &ExprInt) -> Option<String> {
        match value {
            ExprInt::VarInt(s) => {
                let s = s[1..].to_string();
                if self.alias.contains_key(&s) {
                    self.alias.get(&s).cloned()
                } else {
                    Some(s)
                }
            }
            _ => None,
        }
    }
    fn get_str_addr(&self, value: &ExprStr) -> Option<String> {
        match value {
            ExprStr::VarStr(s) => {
                let s = s[1..].to_string();
                if self.alias.contains_key(&s) {
                    self.alias.get(&s).cloned()
                } else {
                    Some(s)
                }
            }
            _ => None,
        }
    }

    fn auto_newline(&self, input: &str) -> String {
        let mut result = String::new();
        for (index, character) in input.chars().enumerate() {
            if index > 0 && index % self.line_limit == 0 {
                result.push('\n');
            }
            result.push(character);
        }
        result
    }

    fn get_image_from_color(&self, color: &str, size: Vector2i) -> Option<Gd<Image>> {
        let mut color = color.to_string();
        if color.to_lowercase() == "black" {
            color = "#000000".to_string();
        } else if color.to_lowercase() == "white" {
            color = "#ffffff".to_string();
        }
        Image::create(size.x, size.y, false, Format::FORMAT_RGBA8).and_then(|mut img| {
            Color::from_html(&color).map(|color| {
                for x in 0..size.x {
                    for y in 0..size.y {
                        img.set_pixel(x, y, color);
                    }
                }
                img
            })
        })
    }

    fn get_texture_from_color(&self, color: &str, size: Vector2i) -> Option<Gd<ImageTexture>> {
        self.get_image_from_color(color, size)
            .and_then(ImageTexture::create_from_image)
    }

    fn get_image_area(&self, img: &Gd<Image>, x: i32, y: i32, w: i32, h: i32) -> Option<Gd<Image>> {
        Image::create(w, h, false, Format::FORMAT_RGBA8).map(|mut new_img| {
            for i in 0..w {
                for j in 0..h {
                    new_img.set_pixel(i, j, img.get_pixel(x + i, y + j));
                }
            }
            new_img
        })
    }

    fn get_texture_area(
        &self,
        img: &Gd<Texture2D>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> Option<Gd<Texture2D>> {
        img.get_image()
            .and_then(|img| self.get_image_area(&img, x, y, w, h))
            .and_then(|res| ImageTexture::create_from_image(res).map(|res| res.upcast()))
    }

    fn execute(&mut self, stmt: Stmt) {
        godot_print!("Executing: {:?}", stmt);
        match stmt {
            Stmt::StmtLabel(_s) => {
                // godot_print!("Label: {}", s);
            }
            Stmt::StmtWait(_s) => {
                // godot_print!("Wait: {}", s);
            }
            Stmt::StmtEcho(s) => {
                self.wait_for_key = true;
                let window = self.base.get_node_as::<Node2D>("window");
                let mut label = window.get_node_as::<Label>("text");
                if self.clear_text {
                    label.set_text(self.auto_newline(&s).to_godot());
                } else {
                    let s = label.get_text().to_string().replace('\n', "") + &s;
                    label.set_text(self.auto_newline(&s).to_godot());
                }
                self.clear_text = s.ends_with('\\');
            }
            Stmt::StmtIf(stmt) => {
                godot_print!(
                    "If: {:?} {:?} {:?}",
                    stmt.condition,
                    self.get_int_expr(&stmt.condition),
                    self.int_vars
                );
                if let Some(condition) = self.get_int_expr(&stmt.condition) {
                    if condition != 0 {
                        for stmt in &stmt.commands {
                            self.execute(Stmt::StmtCall(stmt.clone()));
                        }
                    }
                }
            }
            Stmt::StmtWhile(stmt) => {
                godot_print!(
                    "While: {:?} {:?} {:?}",
                    stmt.condition,
                    self.get_int_expr(&stmt.condition),
                    self.int_vars
                );
                let mut condition = self.get_int_expr(&stmt.condition).unwrap_or_default();
                while condition != 0 {
                    for stmt in &stmt.commands {
                        self.execute(Stmt::StmtCall(stmt.clone()));
                    }
                    condition = self.get_int_expr(&stmt.condition).unwrap_or_default();
                }
            }
            Stmt::StmtFor(stmt) => {
                godot_print!(
                    "For: {:?} {:?} {:?}",
                    stmt.condition,
                    self.get_int_expr(&stmt.condition),
                    self.int_vars
                );
                if let Some(condition) = self.get_int_expr(&stmt.condition) {
                    for i in 0..condition {
                        for stmt in &stmt.commands {
                            self.execute(Stmt::StmtCall(stmt.clone()));
                        }
                    }
                }
            }
            Stmt::StmtCall(call) => match call.identifier.as_str() {
                "arc" => {
                    self.int_vars.insert("arc".to_string(), 1);
                }
                "bg" => {
                    let mut bg = self.base.get_node_as::<Sprite2D>("bg");
                    bg.set_centered(false);
                    if let Some(img) = self.get_str_value(&call.params[0]) {
                        if img == "black" || img == "white" {
                            if let Some(img) = self.get_texture_from_color(&img, self.size) {
                                bg.set_texture(img.upcast());
                            } else {
                                godot_error!("Err:get_texture_from_color");
                            }
                        } else if let Some(img) =
                            try_load(("resources/".to_string() + &img).to_godot())
                        {
                            bg.set_texture(img);
                        } else {
                            godot_error!("Err:Image not found: {}", img);
                        }
                    }
                }
                "btn" => {
                    godot_print!("{:?}", self.btndef);
                    if let Some(btnimg) = self.get_texture_area(
                        &self.btndef,
                        self.get_int_value(&call.params[1]).unwrap_or_default() as i32,
                        self.get_int_value(&call.params[2]).unwrap_or_default() as i32,
                        self.get_int_value(&call.params[3]).unwrap_or_default() as i32,
                        self.get_int_value(&call.params[4]).unwrap_or_default() as i32,
                    ) {
                        let mut btn = OnsButton::alloc_gd();
                        btn.set_texture_normal(btnimg);
                        btn.set_size(Vector2::new(
                            self.get_int_value(&call.params[3]).unwrap_or_default() as f32,
                            self.get_int_value(&call.params[4]).unwrap_or_default() as f32,
                        ));
                        btn.set_position(Vector2::new(
                            self.get_int_value(&call.params[5]).unwrap_or_default() as f32,
                            self.get_int_value(&call.params[6]).unwrap_or_default() as f32,
                        ));
                        self.get_int_value(&call.params[0])
                            .map(|id| btn.call("set_id".into(), &[Variant::from(id)]));
                        self.btns.push(btn.clone());
                        self.base.add_child(btn.upcast());
                    } else {
                        godot_error!("Err:get_texture_area");
                    }
                }
                "btndef" => {
                    self.get_str_value(&call.params[0])
                        .map(|s| "resources/".to_string() + &s)
                        .and_then(try_load::<Texture2D>)
                        .map(|img| self.btndef = img);
                }
                "btnwait" => {
                    self.wait_for_button = true;
                    if let Value::Int(i) = &call.params[0] {
                        self.btn_var = self.get_int_addr(i).unwrap_or_default();
                    }
                }
                "caption" => {
                    if let Some(s) = self.get_str_value(&call.params[0]) {
                        DisplayServer::singleton().window_set_title(s.to_godot())
                    }
                }
                "cdfadeout" => {
                    self.int_vars.insert("cdfadeout".to_string(), 1);
                }
                "cellcheckspbtn" => {
                    let lspid = self.get_int_value(&call.params[0]).unwrap_or_default();
                    let id = self.get_int_value(&call.params[1]).unwrap_or_default();
                    if let Some(btn) = self.lsp.get(&lspid) {
                        btn.clone().call("set_id".into(), &[Variant::from(id)]);
                    }
                }
                "click" => {
                    println!("click");
                    self.wait_for_key = true;
                }
                "clickstr" => {
                    println!("clickstr");
                    self.wait_for_key = true;
                }
                "csel" => {
                    println!("csel");
                    self.wait_for_key = true;
                }
                "csp" => {
                    println!("csp");
                    self.wait_for_key = true;
                }
                "defaultspeed" => {
                    self.int_vars.insert("defaultspeed".to_string(), 1);
                }
                "defmp3vol" => println!("defmp3vol"),
                "defsevol" => println!("defsevol"),
                "defvoicevol" => println!("defvoicevol"),
                "dwave" => println!("dwave"),
                "dwaveloop" => println!("dwaveloop"),
                "dwavestop" => println!("dwavestop"),
                "effect" => println!("effect"),
                "end" => {
                    self.current_line = self.script.stmts.len();
                }
                "erasetextwindow" => {
                    let window = self.base.get_node_as::<Node2D>("window");
                    let mut label = window.get_node_as::<Label>("text");
                    label.set_text("".to_string().to_godot());
                }
                "game" => {
                    self.defined = true;
                }
                "getcursorpos" => println!("getcursorpos"),
                "globalon" => println!("globalon"),
                "gosub" => {}
                "goto" => {
                    println!("goto");
                    if let Value::Label(s) = &call.params[0] {
                        self.current_line = *self.labels.get(s).unwrap_or(&0);
                    }
                }
                "intlimit" => println!("intlimit"),
                "ispage" => println!("ispage"),
                "jumpb" => {
                    while self.current_line > 0 {
                        self.current_line -= 1;
                        if let Stmt::StmtLabel(s) = &self.script.stmts[self.current_line - 1] {
                            if s == &"~" {
                                break;
                            }
                        }
                    }
                }
                "kidokuskip" => println!("kidokuskip"),
                "killmenu" => println!("killmenu"),
                "lsp" => {
                    if let Ok(reg) =
                        Regex::new(r"(:a(/(\d+),(\d+),(\d+))?;)?([\da-zA-Z\.\\/]+).bmp")
                    {
                        self.get_str_value(&call.params[1]).map(|s| {
                            if let Some(s) = reg.captures(&s) {
                                let mut _num = 1;
                                let mut path = String::new();
                                godot_print!("{:?}", s);
                                if let Some(x) = s.get(3) {
                                    _num = x.as_str().parse::<i32>().unwrap_or_default();
                                }
                                if let Some(x) = s.get(6) {
                                    path = x.as_str().to_string();
                                }
                                let mut btn = OnsButton::alloc_gd();
                                btn.set_texture_normal(
                                    try_load::<Texture2D>(
                                        ("resources/".to_string() + &path + "_0.png").to_godot(),
                                    )
                                    .unwrap_or_default(),
                                );
                                btn.set_texture_pressed(
                                    try_load::<Texture2D>(
                                        ("resources/".to_string() + &path + "_0.png").to_godot(),
                                    )
                                    .unwrap_or_default(),
                                );
                                btn.set_position(Vector2::new(
                                    self.get_int_value(&call.params[2]).unwrap_or_default() as f32,
                                    self.get_int_value(&call.params[3]).unwrap_or_default() as f32,
                                ));
                                self.btns.push(btn.clone());
                                self.base.add_child(btn.clone().upcast());
                                let lspid = self.get_int_value(&call.params[0]).unwrap_or_default();
                                self.lsp.insert(lspid, btn);
                            } else {
                                godot_error!("Err:regex");
                            }
                        });
                    }
                }
                "lsph" => println!("lsph"),
                "menusetwindow" => println!("menusetwindow"),
                "mesbox" => println!("mesbox"),
                "mode_ext" => println!("mode_ext"),
                "mode_wave_demo" => println!("mode_wave_demo"),
                "mov" => {
                    println!("mov");
                    println!("{:?}", call.params);
                    match &call.params[0] {
                        Value::Int(i) => {
                            let addr = self.get_int_addr(i).unwrap_or_default();
                            let value = self.get_int_value(&call.params[1]).unwrap_or_default();
                            self.int_vars.insert(addr, value);
                        }
                        Value::Str(s) => {
                            let addr = self.get_str_addr(s).unwrap_or_default();
                            let value = self.get_str_value(&call.params[1]).unwrap_or_default();
                            self.str_vars.insert(addr, value);
                        }
                        _ => {}
                    }
                    for i in self.int_vars.clone() {
                        println!("{}: {}", i.0, i.1);
                    }
                }
                "mp3" => println!("mp3"),
                "mp3fadeout" => println!("mp3fadeout"),
                "mp3loop" => println!("mp3loop"),
                "numalias" => {
                    println!("numalias");
                    let s = self.get_str_value(&call.params[0]).unwrap_or_default();
                    let i = self.get_int_value(&call.params[1]).unwrap_or_default();
                    self.alias.insert(s, i.to_string());
                }
                "print" => {
                    println!("print");
                    let window = self.base.get_node_as::<Node2D>("window");
                    let _label = window.get_node_as::<Label>("text");
                }
                "return" => println!("return"),
                "rmenu" => println!("rmenu"),
                "rmode" => println!("rmode"),
                "savenumber" => println!("savenumber"),
                "saveon" => println!("saveon"),
                "select" => println!("select"),
                "selectcolor" => println!("selectcolor"),
                "setwindow" => {
                    println!("setwindow");
                    let window = self.base.get_node_as::<Node2D>("window");
                    let mut label = window.get_node_as::<Label>("text");
                    let mut windowbg = window.get_node_as::<Sprite2D>("bg");
                    windowbg.set_centered(false);
                    if call.params.len() == 14 {
                        if let Some(img) = self
                            .get_str_value(&call.params[11])
                            .unwrap_or_default()
                            .split(';')
                            .last()
                            .map(|f| load(("resources/".to_string() + f).to_godot()))
                        {
                            // label.set_size(img.get_size());
                            windowbg.set_texture(img);
                        }
                        windowbg.set_position(Vector2 {
                            x: self.get_int_value(&call.params[12]).unwrap_or_default() as f32,
                            y: self.get_int_value(&call.params[13]).unwrap_or_default() as f32,
                        });
                    } else {
                        godot_print!("{:?}", call.params);
                        let size = Vector2i {
                            x: self.get_int_value(&call.params[14]).unwrap_or_default() as i32
                                - self.get_int_value(&call.params[12]).unwrap_or_default() as i32,
                            y: self.get_int_value(&call.params[15]).unwrap_or_default() as i32
                                - self.get_int_value(&call.params[13]).unwrap_or_default() as i32,
                        };
                        // let img = self
                        //     .get_texture_from_color(&self.get_str_value(&call.params[11]), size);
                        label.set_size(Vector2 {
                            x: size.x as f32,
                            y: size.y as f32,
                        });
                        let color = self.get_str_value(&call.params[11]).unwrap_or_default();
                        let color = Color::from_html(color).unwrap_or_default();
                        label.add_theme_color_override("font_color".into(), color);
                        // windowbg.set_texture(img.upcast());
                    }
                    label.set_position(Vector2 {
                        x: self.get_int_value(&call.params[0]).unwrap_or_default() as f32,
                        y: self.get_int_value(&call.params[1]).unwrap_or_default() as f32,
                    });
                    self.line_limit = (self.get_int_value(&call.params[2]).unwrap_or_default()
                        as f32
                        / 2.0) as usize;
                    label.add_theme_font_size_override(
                        "font_size".into(),
                        (self.get_int_value(&call.params[5]).unwrap_or_default() as f32 * 1.5_f32)
                            as i32,
                    );
                }
                "spi" => println!("spi"),
                "stop" => {
                    println!("stop");
                }
                "soundpressplgin" => println!("soundpressplgin"),
                "texec" => println!("texec"),
                "textbtnwait" => println!("textbtnwait"),
                "textclear" => {
                    println!("textclear");
                    let window = self.base.get_node_as::<Node2D>("window");
                    let mut label = window.get_node_as::<Label>("text");
                    label.set_text("".to_string().to_godot());
                }
                "textgosub" => println!("textgosub"),
                "usewheel" => println!("usewheel"),
                "versionstr" => println!("versionstr"),
                "vsp" => println!("vsp"),
                "wait" => {
                    println!("wait");
                    if let Some(i) = self.get_int_value(&call.params[0]) {
                        std::thread::sleep(std::time::Duration::from_millis(i as u64));
                    }
                }
                "windowback" => println!("windowback"),
                "windoweffect" => println!("windoweffect"),
                _ => {
                    godot_error!("Command not found: {}", call.identifier);
                }
            },
        }
    }

    pub fn step(&mut self) {
        self.current_line += 1;
        if self.current_line > self.script.stmts.len() {
            return;
        }
        self.execute(self.script.stmts[self.current_line - 1].clone());
    }

    #[func]
    pub fn finish_button(&mut self, id: Variant) {
        if let Ok(id) = id.try_to::<i64>() {
            self.wait_for_button = false;
            let v = std::mem::take(&mut self.btns);
            for i in v {
                self.base.remove_child(i.clone().upcast());
            }
            self.int_vars.insert(self.btn_var.clone(), id);
        }
    }
}

#[test]
fn test() {
    let reg = Regex::new(r"(:a(/(\d+),(\d+),(\d+))?;)?([\da-zA-Z\.\\/]+)").unwrap();
    let s = r":a/2,0,3;yobi\system\chapter01.bmp";
    if let Some(s) = reg.captures(s) {
        println!("{:?}", s);
        if let Some(x) = s.get(3) {
            println!("{:?}", x.as_str());
        }
        if let Some(x) = s.get(6) {
            println!("{:?}", x.as_str());
        }
    } else {
        godot_error!("Err:regex");
    }
}
