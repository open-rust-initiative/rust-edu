use crate::database::base::{ColumnAttr, ColumnData, DataType, ForeignKeyAttr};
use crate::parser::condition::Condition;
use crate::parser::create::CreateQuery;
use crate::parser::join::JoinInfo;
use crate::parser::select::{BinaryOpCus, SelectQuery};
use crate::system::errors::Errors;
use crate::system::utils::{custom_strip, wildcard_match};
use prettytable::Attr;
use prettytable::{Cell, Row, Table as PTable};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Formatter;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Table {
    pub name: String,
    pub columns: Vec<ColumnAttr>,
    pub col_map: HashMap<String, ColumnData>,
    pub foreign_keys: Option<Vec<ForeignKeyAttr>>,
}

impl Table {
    pub fn new(cq: CreateQuery) -> Table {
        let tb_name = cq.tb_name;
        let columns = cq.cols;
        let mut tb_cols: Vec<ColumnAttr> = vec![];
        let mut tb_col_map: HashMap<String, ColumnData> = HashMap::new();
        for column in &columns {
            let col_header = ColumnAttr::new(
                column.name.to_string(),
                column.datatype.to_string(),
                column.is_pk,
                column.is_nullable,
                column.default.clone(),
            );
            tb_cols.push(col_header);
            tb_col_map.insert(
                column.name.to_string(),
                match DataType::new(column.datatype.to_string()) {
                    DataType::Float => ColumnData::Float(vec![]),
                    DataType::Int => ColumnData::Int(vec![]),
                    DataType::Bool => ColumnData::Bool(vec![]),
                    DataType::String => ColumnData::Str(vec![]),
                    DataType::Invalid => ColumnData::None,
                },
            );
        }
        Table {
            name: tb_name,
            columns: tb_cols,
            col_map: tb_col_map,
            foreign_keys: Some(cq.foreign_key),
        }
    }

    pub fn insert_row(&mut self, cols: Vec<String>, rows: Vec<Vec<String>>) {
        let col_ix_map: HashMap<String, usize> = cols
            .iter()
            .enumerate()
            .map(|(x, k)| (k.to_owned(), x))
            .collect::<HashMap<String, usize>>();
        for row in &rows {
            let col_names: Vec<String> = self
                .col_map
                .iter()
                .map(|(k, _)| k.to_owned())
                .collect::<Vec<String>>();
            for col_name in col_names {
                let mut data = None;
                if cols.contains(&col_name) {
                    if let Some(ix) = col_ix_map.get(&col_name).to_owned() {
                        let col_ix = ix.to_owned();
                        let result = row.to_owned();
                        data = Option::from(result.get(col_ix).unwrap().to_owned());
                    }
                } else {
                    data = None;
                }
                if let Some(col_data) = self.col_map.get_mut(&col_name.to_string()) {
                    match col_data {
                        ColumnData::Int(v) => {
                            if data.is_some() {
                                v.push(Option::from(data.unwrap().parse::<i32>().unwrap()))
                            } else {
                                v.push(None)
                            }
                        }
                        ColumnData::Float(v) => {
                            if data.is_some() {
                                v.push(Option::from(data.unwrap().parse::<f32>().unwrap()))
                            } else {
                                v.push(None)
                            }
                        }
                        ColumnData::Str(v) => {
                            if data.is_some() {
                                v.push(Option::from(data.unwrap().to_string()))
                            } else {
                                v.push(None)
                            }
                        }
                        ColumnData::Bool(v) => {
                            if data.is_some() {
                                v.push(Option::from(data.unwrap().parse::<bool>().unwrap()))
                            } else {
                                v.push(None)
                            }
                        }
                        ColumnData::None => {}
                    }
                }
            }
        }
    }

    pub fn select_data(&self, query: SelectQuery) {
        let condition = &query.condition;
        let mut projection = &query.projection;
        let mut proj_set: HashSet<String> = HashSet::new();
        let mut proj_loc: HashMap<String, usize> = HashMap::new();
        let mut proj_loc_ix = 0;
        for proj in projection {
            if proj.eq("*") {
                let all_cols = self
                    .columns
                    .iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<String>>();
                for col in &all_cols {
                    proj_loc.insert(col.to_string(), proj_loc_ix);
                    proj_loc_ix += 1;
                }
                proj_set.extend(all_cols);
            } else {
                proj_set.insert(proj.to_string());
                if proj_loc.get(proj).is_none() {
                    proj_loc.insert(proj.to_string(), proj_loc_ix);
                    proj_loc_ix += 1;
                }
            }
        }
        let mut binding = proj_set
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        binding.sort_by_key(|k| proj_loc.get(k));
        projection = &binding;

        let mut rows: Vec<HashMap<String, String>> = match self.get_rows() {
            Ok(v) => v,
            Err(err) => {
                err.print();
                return;
            }
        };
        rows = self.filter_rows(condition, rows.clone(), Option::from(projection).cloned());

        let pt = PrettyTable::create("".to_string(), projection.to_vec(), rows);
        println!("{pt}");
    }

    pub fn evaluate_condition(&self, row: &HashMap<String, String>, condition: &Condition) -> bool {
        match &condition {
            Condition::Comparison { left, op, right } => {
                let left_value = row.get(left).unwrap().as_str().to_owned();
                let mut right_value = right.clone().unwrap_or("false".to_string());
                right_value = custom_strip(right_value.as_str(), "\"").to_string();
                match op {
                    BinaryOpCus::Lt => match self.col_map.get(left) {
                        None => {
                            panic!("")
                        }
                        Some(x) => match x {
                            ColumnData::Int(_) => {
                                left_value.parse::<i32>().unwrap()
                                    < right_value.parse::<i32>().unwrap()
                            }
                            ColumnData::Float(_) => {
                                left_value.parse::<f32>().unwrap()
                                    < right_value.parse::<f32>().unwrap()
                            }
                            ColumnData::Str(_) => left_value < right_value,
                            ColumnData::Bool(_) => {
                                left_value.parse::<bool>().unwrap()
                                    < right_value.parse::<bool>().unwrap()
                            }
                            ColumnData::None => left_value < right_value,
                        },
                    },
                    BinaryOpCus::Gt => match self.col_map.get(left) {
                        None => {
                            panic!("")
                        }
                        Some(x) => match x {
                            ColumnData::Int(_) => {
                                left_value.parse::<i32>().unwrap()
                                    > right_value.parse::<i32>().unwrap()
                            }
                            ColumnData::Float(_) => {
                                left_value.parse::<f32>().unwrap()
                                    > right_value.parse::<f32>().unwrap()
                            }
                            ColumnData::Str(_) => left_value > right_value,
                            ColumnData::Bool(_) => {
                                left_value.parse::<bool>().unwrap()
                                    > right_value.parse::<bool>().unwrap()
                            }
                            ColumnData::None => left_value > right_value,
                        },
                    },
                    BinaryOpCus::Eq => match self.col_map.get(left) {
                        None => {
                            panic!("")
                        }
                        Some(x) => match x {
                            ColumnData::Int(_) => {
                                left_value.parse::<i32>().unwrap()
                                    == right_value.parse::<i32>().unwrap()
                            }
                            ColumnData::Float(_) => {
                                left_value.parse::<f32>().unwrap()
                                    == right_value.parse::<f32>().unwrap()
                            }
                            ColumnData::Str(_) => left_value.eq(&right_value),
                            ColumnData::Bool(_) => {
                                left_value.parse::<bool>().unwrap()
                                    == right_value.parse::<bool>().unwrap()
                            }
                            ColumnData::None => left_value == right_value,
                        },
                    },
                    BinaryOpCus::IsNull => match self.col_map.get(left) {
                        None => {
                            panic!("")
                        }
                        Some(_) => left_value.is_empty(),
                    },
                    BinaryOpCus::Like => match self.col_map.get(left) {
                        None => {
                            panic!("")
                        }
                        Some(_) => wildcard_match(right_value.as_str(), left_value.as_str()),
                    },
                    _ => false,
                }
            }
            Condition::Logical { left, op, right } => {
                let left_result = self.evaluate_condition(row, &**left);
                let right_result = self.evaluate_condition(row, &**right);
                match op {
                    BinaryOpCus::And => left_result && right_result,
                    BinaryOpCus::Or => left_result || right_result,
                    _ => false,
                }
            }
        }
    }

    pub fn add_column(&mut self, column_attr: ColumnAttr) {
        self.columns.push(column_attr.clone());
        self.col_map.insert(
            column_attr.name.to_string(),
            match DataType::new(column_attr.datatype.to_string()) {
                DataType::Float => ColumnData::Float(vec![]),
                DataType::Int => ColumnData::Int(vec![]),
                DataType::Bool => ColumnData::Bool(vec![]),
                DataType::String => ColumnData::Str(vec![]),
                DataType::Invalid => ColumnData::None,
            },
        );
    }

    pub fn add_row(&mut self, row: HashMap<String, String>) {
        for (k, val) in row {
            if let Some(col_data) = self.col_map.get_mut(k.as_str()) {
                match col_data {
                    ColumnData::Int(v) => {
                        if val.is_empty() {
                            v.push(None)
                        } else {
                            v.push(Option::from(val.parse::<i32>().unwrap()))
                        }
                    }
                    ColumnData::Float(v) => {
                        if val.is_empty() {
                            v.push(None)
                        } else {
                            v.push(Option::from(val.parse::<f32>().unwrap()))
                        }
                    }
                    ColumnData::Str(v) => {
                        if val.is_empty() {
                            v.push(None)
                        } else {
                            v.push(Option::from(val))
                        }
                    }
                    ColumnData::Bool(v) => {
                        if val.is_empty() {
                            v.push(None)
                        } else {
                            v.push(Option::from(val.parse::<bool>().unwrap()))
                        }
                    }
                    ColumnData::None => {}
                }
            }
        }
    }

    pub fn get_rows(&self) -> Result<Vec<HashMap<String, String>>, Errors> {
        let row_nums = match self
            .col_map
            .get(
                &self
                    .columns
                    .iter()
                    .filter(|row| row.is_pk)
                    .map(|row| row.name.to_string())
                    .collect::<Vec<String>>()
                    .first()
                    .unwrap()
                    .to_owned(),
            )
            .unwrap()
            .count()
        {
            Ok(v) => v,
            Err(err) => {
                return Err(err);
            }
        };
        let rows = (0..row_nums)
            .map(|rid| {
                self.col_map
                    .iter()
                    .map(|(k, v)| {
                        let data = v.get_data_by_ix(&vec![rid]);
                        (
                            k.to_owned(),
                            data.expect(Errors::InvalidColumnType.to_string().as_str())
                                .first()
                                .unwrap()
                                .to_owned(),
                        )
                    })
                    .collect::<HashMap<String, String>>()
            })
            .collect::<Vec<HashMap<String, String>>>();
        Ok(rows)
    }

    pub fn filter_rows(
        &self,
        condition: &Option<Condition>,
        rows: Vec<HashMap<String, String>>,
        projection: Option<Vec<String>>,
    ) -> Vec<HashMap<String, String>> {
        match condition {
            None => rows,
            Some(con) => rows
                .iter()
                .filter(|&row| self.evaluate_condition(row, &con))
                .clone()
                .map(|row| {
                    row.iter()
                        .filter(|(col, _)| {
                            if projection.is_some() {
                                projection.clone().unwrap().contains(&col)
                            } else {
                                true
                            }
                        })
                        .map(|(k, v)| (k.to_owned(), v.to_owned()))
                        .collect::<HashMap<String, String>>()
                })
                .collect::<Vec<HashMap<String, String>>>(),
        }
    }

    pub fn join_tbs(tb1: &Table, tb2: &Table, join_info: JoinInfo) -> Table {
        let mut joint_table = Table {
            name: format!("{}-{}", tb1.name, tb2.name),
            columns: vec![],
            col_map: Default::default(),
            foreign_keys: None,
        };
        for col in &tb1.columns {
            joint_table.add_column(ColumnAttr {
                name: format!("{}.{}", tb1.name, col.name),
                datatype: col.clone().datatype,
                is_pk: col.is_pk,
                is_nullable: col.is_nullable,
                default: col.clone().default,
            });
        }
        for col in &tb2.columns {
            joint_table.add_column(ColumnAttr {
                name: format!("{}.{}", tb2.name, col.name),
                datatype: col.clone().datatype,
                is_pk: col.is_pk,
                is_nullable: col.is_nullable,
                default: col.clone().default,
            });
        }
        let left_col_data = tb1
            .col_map
            .get(&join_info.left_column)
            .unwrap()
            .get_all_data()
            .unwrap();
        let right_col_data = tb2
            .col_map
            .get(&join_info.right_column)
            .unwrap()
            .get_all_data()
            .unwrap();
        for (left_ix, left_row) in left_col_data.iter().enumerate() {
            if left_row.is_empty() {
                continue;
            }
            for (right_ix, right_row) in right_col_data.iter().enumerate() {
                if right_row.is_empty() {
                    continue;
                }
                if !right_row.eq(left_row) {
                    continue;
                }
                let mut left_row_data = tb1.get_data_by_row(left_ix, true);
                let right_row_data = tb2.get_data_by_row(right_ix, true);
                left_row_data.extend(right_row_data);
                joint_table.add_row(left_row_data);
            }
        }
        joint_table
    }

    pub fn get_data_by_row(&self, row_ix: usize, is_join: bool) -> HashMap<String, String> {
        self.col_map
            .iter()
            .map(|(k, v)| {
                let val = v
                    .get_data_by_ix(vec![row_ix].as_ref())
                    .expect(Errors::InvalidColumnType.to_string().as_str())
                    .first()
                    .unwrap()
                    .to_owned();
                let key = if is_join {
                    format!("{}.{}", self.name, k.to_string())
                } else {
                    k.to_string()
                };
                (key, val)
            })
            .collect::<HashMap<String, String>>()
    }

    pub fn show_info(&self) {
        let headers = vec!["name", "datatype", "is_pk", "is_nullable", "default"];
        let rows = self
            .columns
            .iter()
            .map(|c| c.attr())
            .collect::<Vec<HashMap<String, String>>>();
        let mut pt = PrettyTable::create(
            self.name.to_string(),
            headers
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<String>>(),
            rows,
        );
        if self.foreign_keys.is_some() {
            let foreign_keys = self.foreign_keys.as_ref().unwrap();
            let fk_str = foreign_keys
                .iter()
                .map(|x| format!("{}---{}.{}", x.col_a, x.table, x.col_b))
                .collect::<Vec<String>>();
            pt.add_more("Foreign Keys".to_string(), fk_str);
        }
        println!("{pt}");
    }
    pub fn print_table_data(&self) {
        let mut p_table = PTable::new();

        let cnames = self
            .columns
            .iter()
            .map(|col| col.name.to_string())
            .collect::<Vec<String>>();

        let header_row = Row::new(
            cnames
                .iter()
                .map(|col| Cell::new(&col))
                .collect::<Vec<Cell>>(),
        );

        let first_col_data = self
            .col_map
            .get(&self.columns.first().unwrap().name)
            .unwrap();
        let num_rows = match first_col_data.count() {
            Ok(v) => v,
            Err(err) => {
                err.print();
                return;
            }
        };
        let mut print_table_rows: Vec<Row> = vec![Row::new(vec![]); num_rows];

        for col_name in &cnames {
            let col_val = self
                .col_map
                .get(col_name)
                .expect("Can't find any rows with the given column");
            let columns: Vec<String> = match col_val.get_all_data() {
                Ok(v) => v,
                Err(err) => {
                    err.print();
                    return;
                }
            };

            for i in 0..num_rows {
                print_table_rows[i].add_cell(Cell::new(&columns[i]));
            }
        }

        p_table.add_row(header_row);
        for row in print_table_rows {
            p_table.add_row(row);
        }

        p_table.printstd();
    }
}

pub struct PrettyTable {
    pub name: String,
    pub header: Vec<String>,
    pub values: HashMap<String, Vec<String>>,
    pub rows: Vec<HashMap<String, String>>,
    pub others: HashMap<String, Vec<String>>,
}

impl PrettyTable {
    pub fn new(
        name: String,
        header: Vec<String>,
        values: HashMap<String, Vec<String>>,
    ) -> PrettyTable {
        return PrettyTable {
            name,
            header,
            values,
            rows: vec![],
            others: HashMap::new(),
        };
    }
    pub fn create(
        name: String,
        header: Vec<String>,
        rows: Vec<HashMap<String, String>>,
    ) -> PrettyTable {
        return PrettyTable {
            name,
            header,
            values: HashMap::new(),
            rows,
            others: HashMap::new(),
        };
    }
    pub fn add_more(&mut self, key: String, val: Vec<String>) {
        self.others.insert(key, val);
    }
}

impl fmt::Display for PrettyTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut pt = PTable::new();
        let header_row = Row::new(
            self.header
                .iter()
                .map(|col| Cell::new(&col).with_style(Attr::Bold))
                .collect::<Vec<Cell>>(),
        );
        let mut pt_rows: Vec<Row> = vec![];
        if !self.rows.is_empty() {
            let num_rows = self.rows.len();
            pt_rows = vec![Row::new(vec![]); num_rows];
            for col in &self.header {
                for i in 0..num_rows {
                    pt_rows[i].add_cell(Cell::new(self.rows[i].get(col).unwrap()));
                }
            }
        } else if !self.values.is_empty() {
            let num_rows = self.values.get(&self.header[0]).unwrap_or(&vec![]).len();
            pt_rows = vec![Row::new(vec![]); num_rows];
            for col in &self.header {
                let col_vals = self.values.get(col).unwrap();
                for i in 0..num_rows {
                    pt_rows[i].add_cell(Cell::new(&col_vals[i]));
                }
            }
        }
        pt.add_row(header_row);
        for row in pt_rows {
            pt.add_row(row);
        }
        if !self.others.is_empty() {
            let col_num = self.header.len() - 1;
            for (k, v) in &self.others {
                pt.add_row(Row::new(vec![
                    Cell::new(k.as_str()).style_spec("Fb"),
                    Cell::new(v.join(" ").as_str()).style_spec(format!("H{}", col_num).as_str()),
                ]));
            }
        }
        write!(f, "{}\n{}", self.name, pt)
    }
}
