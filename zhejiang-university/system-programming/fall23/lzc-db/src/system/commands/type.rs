use crate::system::errors::Errors;

pub enum CommandType {
    CreateTable,
    Insert,
    Select,
    Delete,
    Drop,
    Update,
    ShowTable,
    ShowDB,
    TableInfo,
    System,
}

impl CommandType {
    pub fn new(command: String) -> Result<CommandType, Errors> {
        let vars = command.split(" ").collect::<Vec<&str>>();
        match vars[0].to_lowercase().as_str() {
            "create" => Ok(CommandType::CreateTable),
            "insert" => Ok(CommandType::Insert),
            "select" => Ok(CommandType::Select),
            "delete" => Ok(CommandType::Delete),
            "drop" => Ok(CommandType::Drop),
            "update" => Ok(CommandType::Update),
            "showtb" => Ok(CommandType::ShowTable),
            "showdb" => Ok(CommandType::ShowDB),
            "tableinfo" => Ok(CommandType::TableInfo),
            "sys" => Ok(CommandType::System),
            _ => Err(Errors::InvalidCommand),
        }
    }
}

pub enum SysCommand {
    CreateDatabase,
    UseDatabase,
    DropDatabase,
    ShowDatabases,
    ChangePassword,
    HelpTips,
    SysInfo,
}

impl SysCommand {
    pub fn new(command: String) -> Result<SysCommand, Errors> {
        let vars = command.split(" ").collect::<Vec<&str>>();
        match vars[1].to_lowercase().as_str() {
            "createdb" => Ok(SysCommand::CreateDatabase),
            "usedb" => Ok(SysCommand::UseDatabase),
            "dropdb" => Ok(SysCommand::DropDatabase),
            "showdb" => Ok(SysCommand::ShowDatabases),
            "changepwd" => Ok(SysCommand::ChangePassword),
            "help" => Ok(SysCommand::HelpTips),
            "showsys" => Ok(SysCommand::SysInfo),
            _ => Err(Errors::InvalidCommand),
        }
    }
}
