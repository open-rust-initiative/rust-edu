use ansi_term::Color;

pub fn create_table(sample: bool) -> String {
    let mut result = format!(
        "{} {} {}",
        Color::Red.bold().paint("Create table"),
        Color::Green.paint("table_name"),
        Color::Green.paint("(column datatype constrain)"),
    );
    if sample {
        result = result + "\n" + format!(
            "{} {} {}",
            Color::Red.bold().paint("Create table"),
            Color::Green.paint("articles"),
            Color::Green.paint("(id INT PRIMARY KEY, title VARCHAR(255) NOT NULL, user_id INT, category_id INT, FOREIGN KEY (user_id) REFERENCES users(id), FOREIGN KEY (category_id) REFERENCES category(id) )"),
        ).as_str();
    }
    result
}

pub fn drop_table(sample: bool) -> String {
    let mut result = format!(
        "{} {}",
        Color::Red.bold().paint("Drop table"),
        Color::Green.paint("table_name")
    );
    if sample {
        result = result
            + "\n"
            + format!(
                "{} {}",
                Color::Red.bold().paint("Drop table"),
                Color::Green.paint("articles")
            )
            .as_str();
    }
    result
}

pub fn select_data(sample: bool) -> String {
    let mut result = format!(
        "{} {} {} {} {} {}",
        Color::Red.bold().paint("Select"),
        Color::Green.paint("projections"),
        Color::Red.bold().paint("from"),
        Color::Green.paint("table"),
        Color::RGB(240, 240, 240).paint("<join another table on field1=field2>"),
        Color::RGB(240, 240, 240).paint("<where conditions>")
    );
    if sample {
        result = result
            + "\n"
            + format!(
                "{} {} {} {} {} {}",
                Color::Red.bold().paint("Select"),
                Color::Green.paint("*"),
                Color::Red.bold().paint("from"),
                Color::Green.paint("articles"),
                Color::RGB(240, 240, 240).paint("Join users On articles.user_id=users.id"),
                Color::RGB(240, 240, 240).paint(
                    "Where articles.title like \"%Hello World%\" and users.username like \"user_\""
                )
            )
            .as_str();
    }
    result
}

pub fn insert_data(sample: bool) -> String {
    let mut result = format!(
        "{} {} ({}) {} ({})",
        Color::Red.bold().paint("Insert Into"),
        Color::Green.paint("table"),
        Color::Green.paint("colA,colB,colC..."),
        Color::Red.bold().paint("Values"),
        Color::Green.paint("valA,valB,valC..."),
    );
    if sample {
        result = result
            + "\n"
            + format!(
                "{} {} ({}) {} ({})",
                Color::Red.bold().paint("Insert Into"),
                Color::Green.paint("users"),
                Color::Green.paint("(id, username, password, email)"),
                Color::Red.bold().paint("Values"),
                Color::Green.paint("(1, 'user1', 'password1', 'user1@qq.com')"),
            )
            .as_str();
    }
    result
}

pub fn delete_data(sample: bool) -> String {
    let mut result = format!(
        "{} {} {}",
        Color::Red.bold().paint("Delete from"),
        Color::Green.paint("table"),
        Color::RGB(240, 240, 240).paint("<where conditions>")
    );
    if sample {
        result = result
            + "\n"
            + format!(
                "{} {} {}",
                Color::Red.bold().paint("Delete from"),
                Color::Green.paint("users"),
                Color::RGB(240, 240, 240).paint("Where id>3")
            )
            .as_str();
    }
    result
}

pub fn update_data(sample: bool) -> String {
    let mut result = format!(
        "{} {} {} {} {}",
        Color::Red.bold().paint("UPDATE"),
        Color::Green.paint("table"),
        Color::Red.bold().paint("SET"),
        Color::Green.paint("colA = \"valA\""),
        Color::RGB(240, 240, 240).paint("<where conditions>")
    );
    if sample {
        result = result + "\n" + format!(
            "{} {} {} {} {}",
            Color::Red.bold().paint("UPDATE"),
            Color::Green.paint("users"),
            Color::Red.bold().paint("SET"),
            Color::Green.paint("password = \"new_password\", email = \"new_email@example.com\" WHERE email like \"%@qq.com\""),
            Color::RGB(240, 240, 240).paint("<where conditions>")
        ).as_str();
    }
    result
}

pub fn create_db() -> String {
    format!(
        "{} {}",
        Color::Red.bold().paint("sys createdb"),
        Color::Green.paint("db_name"),
    )
}

pub fn use_db() -> String {
    format!(
        "{} {}",
        Color::Red.bold().paint("sys usedb"),
        Color::Green.paint("db_name"),
    )
}

pub fn drop_db() -> String {
    format!(
        "{} {}",
        Color::Red.bold().paint("sys dropdb"),
        Color::Green.paint("db_name"),
    )
}

pub fn show_dbs() -> String {
    format!("{}", Color::Red.bold().paint("sys showdb"),)
}

pub fn change_pwd() -> String {
    format!(
        "{} {}",
        Color::Red.bold().paint("sys changepwd"),
        Color::Green.paint("newpwd"),
    )
}
