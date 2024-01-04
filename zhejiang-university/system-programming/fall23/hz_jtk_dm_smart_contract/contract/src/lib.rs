use std::collections::HashMap;
use std::ptr::null;
use std::fmt;
// Find all our documentation at https://docs.near.org
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::log_str;
use near_sdk::serde::{Serialize,Deserialize};
use near_sdk::near_bindgen;
use near_sdk::{Balance,AccountId,env};
use near_sdk::collections::{LookupMap, Vector,UnorderedMap,UnorderedSet};
// use near_sdk::test_utils::VMContextBuilder;
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
//医疗记录
pub struct MedicalRecord {
    //医生的账户
  pub doctor: String, 
  //病例详情
  pub detail: String,
  //看病时间
  pub time: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
//药方记录
pub struct Prescription {
    pub medicine: Vec<Medicine>,
    pub prescribing_doctor: String,
    pub prescription_time: String,
    pub is_use:bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
//药
pub struct Medicine {
    pub medicine_info: String,
    pub medicine_name: String,
    pub price: u32
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
// 住院信息
pub struct Hospitalization {
    pub patient: String,   // 患者 ID
    pub room_number: u32,     // 床位号
    pub admission_date: String,  // 入院时间戳
    pub discharge_date: Option<String>,  // 出院时间戳（可选）
    pub in_hospital: bool,  // 是否在院中
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
//床位信息
pub struct Bed {
    pub room_number: u32,   // 床位号
    pub is_occupied: bool,  // 是否已占用
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RoundsRecord {
    pub doctor: String,   // 医生 ID
    pub room_number: u32,    // 床位号
    pub timestamp: String,      // 巡查时间戳
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
//患者预约挂号记录
pub struct Reservation {
    pub patient: String,
    pub doctor: String,
    pub time: String
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
//访客信息
pub struct Vistor {
    pub visitor: String,
    pub time: String
}
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
//患者账单信息
pub struct Bill {
    pub total_amount: u32,//为了节省链上存储开销选择32位无符号整型，谁家好人看病能花42亿以上啊……
    pub last_update_time: String,
    pub is_paid: bool
}

impl fmt::Display for MedicalRecord {
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "MedicalRecord(doctor: {}, detail: {}, time: {})", self.doctor, self.detail, self.time)
    }
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    //账户身份
    pub identify:UnorderedMap<String, String>,
    //病例详情
    pub patient_record: UnorderedMap<String, Vector<MedicalRecord>>,
    //授权病例查询列表
    pub allow_record: UnorderedMap<String, UnorderedSet<String>>,
    //病人药方记录
    pub patient_medicine: UnorderedMap<String, Vector<Prescription>>,
    //患者预约挂号记录列表
    pub reservation_record: UnorderedMap<String, Reservation>,
    //医生状态 true代表空闲 false代表已有预约
    pub doctor_status: UnorderedMap<String, bool>,
    
    pub hospitalizations: UnorderedMap<String, Hospitalization>,  // 存储住院信息的映射
    pub beds: UnorderedMap<u32, Bed> ,                    // 存储床位信息的映射
    // 用于存储巡查记录，key 为病床号
    pub rounds_records: UnorderedMap<u32, Vec<RoundsRecord>>,
    //住院患者的访客记录列表
    pub vistor_list: UnorderedMap<String, Vector<Vistor>>,
    //患者账单信息
    pub bill_record: UnorderedMap<String, Bill>
}
const MIN_STORAGE: Balance = 1_100_000_000_000_000_000_000_000; //1.1Ⓝ
const POINT_ONE: Balance = 100_000_000_000_000_000_000_000;//0.1N
// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self 
        {   
            //需要注意，每一个区块链接口提供的数据结构在初始化的时候都需要添加一个前缀，如果是嵌套结构，在嵌套结构里也需要添加前缀，前缀可以使用账户ID或者其他形式，前缀不能一样
            patient_record: UnorderedMap::new(b"p"),
            identify: UnorderedMap::new(b"i"),
            allow_record:UnorderedMap::new(b"a"),
            patient_medicine:UnorderedMap::new(b"m"),
            reservation_record:UnorderedMap::new(b"r"),
            doctor_status:UnorderedMap::new(b"d"),
            vistor_list:UnorderedMap::new(b"v"),
            bill_record:UnorderedMap::new(b"b"),
            hospitalizations:UnorderedMap::new(b"h"),
            beds:UnorderedMap::new(b"s"),
            rounds_records:UnorderedMap::new(b"c")
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    //注册
    pub fn register(&mut self,role:String) -> bool {
        log_str(&format!("Saving greeting: {:?}", role));
        log_str(&format!("{}",env::signer_account_id().as_str().to_string()));
        self.identify.insert(&env::signer_account_id().as_str().to_string(),&role);
        if role == "doctor" {
            self.doctor_status.insert(&env::signer_account_id().as_str().to_string(),&true);
        } else if role == "patient" {
            let patient_bill = Bill {
                total_amount: 0,
                last_update_time: env::block_timestamp().to_string(),
                is_paid: false
            };

            let patient_id = env::signer_account_id().as_str().to_string();
            self.bill_record.insert(&patient_id, &patient_bill);
        }
        return true;
    }

    //查询用户当前角色
    pub fn get_role(&self) -> String {
        
        let val = match self.identify.get(&env::signer_account_id().as_str().to_string()) {
            Some(x) => return x,
            None => panic!("该账户还没有角色")
          };
    }

    //检查某用户是否拥有某个权限
    pub fn check_role(&self, account_id: &String, role:String) -> bool {
        match self.identify.get(account_id){
            Some(x) => return x == role,
            None => panic!("该账户还没有注册成为{}角色",role)
        }
    }

    //添加病例
    pub fn add_record(&mut self,patient:String,detail:String)->bool{
        let val = match self.identify.get(&env::signer_account_id().as_str().to_string()) {
            Some(x) => {
                assert!(x=="doctor","该账户不是医生，不能看病");
                match self.identify.get(&patient){
                    Some(patient_id) => {
                        
                        let medical_record = MedicalRecord {
                            doctor: env::signer_account_id().as_str().to_string(),
                            detail: detail,
                            time: env::block_timestamp().to_string(),//timestamp?
                        };
                        //如果存在直接插入一条新记录，如果不存在则新建一个Vector,并插入
                        if self.patient_record.get(&patient).is_some() {
                            let mut patient_medicine_vec = self.patient_record.get(&patient).unwrap();
                            patient_medicine_vec.push(&medical_record);
                            self.patient_record.insert(&patient, &patient_medicine_vec);

                            
                            log_str(&format!("保存病例成功: {medical_record}"));
                            log_str(&format!("当前时间: {}",env::block_timestamp()));
                        }else {
                            let prefix: Vec<u8> = 
                            [
                                b"m".as_slice(),
                                &near_sdk::env::sha256_array(patient.as_bytes()),
                            ].concat();//prefix的作用?
                            let mut patient_recode_vec:Vector<MedicalRecord>=Vector::new(prefix);
                            patient_recode_vec.push(&medical_record);
                            self.patient_record.insert(&patient,&patient_recode_vec);
                            log_str(&format!("保存病例成功: {medical_record}"));
                            log_str(&format!("病人编号: {patient}"));
                        log_str(&format!("当前时间: {}",env::block_timestamp()));
                        }
                        
                        return true;
                    },
                    None => return false,//panic
                }

            },
            None => panic!("该账户还没有角色")
          };
    }
    //授权某人查询病例
    pub fn add_allow_record(&mut self,doctor:String)->bool{
        assert!(self.identify.get(&doctor).is_some(), "授权账户没有角色");
        assert!(self.identify.get(&env::signer_account_id().as_str().to_string()).is_some(), "该账户还没有角色");
        //如果允许列表已经含有该角色的授权列表
        if self.allow_record.get(&env::signer_account_id().as_str().to_string()).is_some() {
            let mut patient_medicine_vec = self.allow_record.get(&env::signer_account_id().as_str().to_string()).unwrap();
            patient_medicine_vec.insert(&doctor);
            self.allow_record.insert(&env::signer_account_id().as_str().to_string(), &patient_medicine_vec);
        }else {
            let prefix: Vec<u8> = 
            [
                b"a".as_slice(),
                &near_sdk::env::sha256_array(&env::signer_account_id().as_str().to_string().as_bytes()),
            ].concat();//prefix的作用?
            let mut patient_recode_vec:UnorderedSet<String>=UnorderedSet::new(prefix);
            patient_recode_vec.insert(&doctor);
            self.allow_record.insert(&env::signer_account_id().as_str().to_string(),&patient_recode_vec);
            //打印输出结果
            if let Some(allowed_set) = self.allow_record.get(&env::signer_account_id().as_str().to_string()) {
                // 将 UnorderedSet 转换为 Vec<String> 以便遍历
                let set_values: Vec<String> = allowed_set.to_vec();
            
                // 遍历并输出所有值
                for (index, value) in set_values.iter().enumerate() {
                    log_str(&format!("Index {}: Value: {}", index, value));
                }
            } else {
                // 处理键不存在的情况
                log_str("指定的键不存在");
            }
        }
        return true;
    }
    //查询某人病例，需要授权，自己查询自己病例除外
    // 查询用户所有病例的函数
    pub fn get_user_medical_records(&self, patient_id: String) -> Vec<MedicalRecord> {
        // 检查调用者是否是病人本人
        let caller = env::signer_account_id();
        let is_patient_himself = patient_id == caller.to_string();

        // 如果不是病人本人，则检查调用者是否在授权名单中
        if !is_patient_himself {
            assert!(self.allow_record.get(&patient_id).is_some() && self.allow_record.get(&patient_id).unwrap().contains(&caller.to_string()), "您没有权限查询该病人的病例");
        }

        // 获取患者的病例列表
        if let Some(patient_rec_vec) = self.patient_record.get(&patient_id) {
            // 返回病例列表
            return patient_rec_vec.to_vec();
        } else {
            log_str(&format!("病人编号: {patient_id}"));
            log_str(&format!("没有数据"));
            return Vec::new();
        }
    }
    //医生开药方
    pub fn prescribe_medicine(&mut self, patient_id: String, medicine: Vec<Medicine>) -> bool {
        // 获取调用者账户ID
        let doctor_id = env::signer_account_id().to_string();
        assert_eq!(self.check_role(&patient_id,"patient".to_string()),true,"Input patient id isn't valid.");

        // 确保调用者是医生
        assert!(
            self.identify.get(&doctor_id) == Some("doctor".to_string()),
            "只有医生可以调用此函数"
        );

        let medicine_info = medicine.clone();

        // 创建处方记录
        let prescription = Prescription {
            medicine,
            prescribing_doctor: doctor_id.clone(),
            prescription_time: env::block_timestamp().to_string(),
            is_use: false,
        };

        // 将处方添加到患者记录中
        if self.patient_medicine.get(&patient_id).is_some(){
            let mut patient_medicine_vec = self.patient_medicine.get(&patient_id).unwrap();
            patient_medicine_vec.push(&prescription);
            self.patient_medicine.insert(&patient_id, &patient_medicine_vec);
        }else {
            let prefix: Vec<u8> = 
            [
                b"p".as_slice(),
                &near_sdk::env::sha256_array(&patient_id.as_bytes()),
            ].concat();
            let mut new_prescriptions: Vector<Prescription> = Vector::new(prefix);
            new_prescriptions.push(&prescription);
            self.patient_medicine.insert(&patient_id, &new_prescriptions);
            log_str(&format!("保存药方成功2: "));
        }

        //更新账单信息
        if let Some(mut bill) = self.bill_record.get(&patient_id) {
            for element in &medicine_info {
                bill.total_amount += element.price;
            }
            bill.last_update_time = env::block_timestamp().to_string();
            self.bill_record.insert(&patient_id,&bill);
        } else {
            panic!("Could not find the bill of this patient.");
        }

        true
    }

    pub fn pay_the_bill(&mut self, patient_id: &String, balance: u32) -> bool {
        assert_eq!(self.check_role(patient_id,"patient".to_string()),true,"Input patient id isn't valid.");
        let (amount,_,paid) = self.get_bill_info(patient_id);
        assert_eq!(paid,false,"This patient has paid his/her bill.");
        if amount > balance {
            log_str("Payment failed: insufficient balance.");
            return false;
        } else {
            let new_bill = Bill {
                total_amount: amount.clone(),
                last_update_time: env::block_timestamp().to_string(),
                is_paid: true
            };
            self.bill_record.insert(patient_id,&new_bill);
            let remain = balance - amount;
            log_str(&format!("Payment success: The remaining amount is {}.",remain));
        }
        true
    }

    pub fn check_bill_is_paid(&self, patient_id: &String) -> bool {
        assert_eq!(self.check_role(patient_id,"patient".to_string()),true,"Input patient id isn't valid.");
        let (_,_,status) = self.get_bill_info(patient_id);
        status
    }

     // 药房工作人员确认用户缴费并发药
    pub fn confirm_payment_and_dispense_medicine(&mut self, patient_id: String, prescription_index: u64) -> bool {
        // 获取调用者账户ID
        let pharmacy_staff_id = env::signer_account_id().to_string();
        // 确保调用者是药房工作人员
        assert!(
            self.identify.get(&pharmacy_staff_id) == Some("pharmacy".to_string()),
            "只有药房工作人员可以调用此函数"
        );
        // 获取患者药方记录
        if self.patient_medicine.get(&patient_id).is_some() {
            // 确保索引有效
            assert!(prescription_index < self.patient_medicine.get(&patient_id).unwrap().len() as u64, "药方索引无效");
            // 获取要更新的药方记录
            let mut patient_prescription=self.patient_medicine.get(&patient_id).unwrap();
            if patient_prescription.get(prescription_index as u64).is_some() {
                // 直接更新 is_use 变量,必须使用replace，不能直接修改变量的值，是不起作用的
                let mut pre=patient_prescription.get(prescription_index as u64).unwrap();
                pre.is_use=true;
                patient_prescription.replace(prescription_index, &pre);
                return true;
            }
            return false;
        }
        false
    }
    // 查询某人药方的函数
    pub fn get_user_prescriptions(&self, patient_id: String) -> Vec<Prescription> {
        // 检查调用者是否是患者本人
        let caller = env::signer_account_id();
        let is_patient_himself = patient_id == caller.to_string();

        // 如果不是患者本人，则检查调用者是否在授权名单中
        if !is_patient_himself {
            assert!(
                self.allow_record.get(&patient_id).is_some()
                    && self.allow_record.get(&patient_id).unwrap().contains(&caller.to_string()),
                "您没有权限查询该患者的药方"
            );
        }

        // 获取患者的药方记录
        if let Some(patient_prescriptions) = self.patient_medicine.get(&patient_id) {
            // 返回药方记录列表
            return patient_prescriptions.to_vec();
        } else {
            return Vec::new();
        }
    }

    pub fn get_doctor_status(&self, doctor_id: &String) -> bool {
        if let Some(status) = self.doctor_status.get(doctor_id){
            status
        } else {
            panic!("invalid doctor id.");
        }
    }

    // 患者和处于空闲状态的医生预约挂号看病
    pub fn make_reservation(&mut self, doctor_id: &String) -> bool {
        let caller = env::signer_account_id().to_string();
        assert_eq!(self.check_role(&caller,"patient".to_string()),true,"Method caller isn't a patient.");
        assert_eq!(self.check_role(doctor_id,"doctor".to_string()),true,"Input doctor id isn't valid.");
        let mut status = self.get_doctor_status(doctor_id);
        if status == false {
            panic!("This doctor isn't avaliable.");
            return false;
        } else {
            status = false;
            self.doctor_status.insert(doctor_id,&status);
            let reservation_rec = Reservation {
                patient: caller.clone(),
                doctor: doctor_id.clone(),
                time: env::block_timestamp().to_string(),
            };
            self.reservation_record.insert(&caller, &reservation_rec);
        }
        true
    }
    // 入院登记
    pub fn admit_patient(&mut self, patient_id: String)  -> bool{
        // 获取调用者账户ID
        let doctor_id = env::signer_account_id().to_string();

        // 确保调用者是医生
        assert!(
            self.identify.get(&doctor_id) == Some("doctor".to_string()),
            "只有医生可以调用此函数"
        );

        // 检查患者是否已经注册
        assert!(
            self.identify.get(&patient_id) == Some("patient".to_string()),
            "患者未注册"
        );

        // 获取可用的床位列表
        let available_beds = self.get_available_beds();
        
        // 检查是否有可用床位
        assert!(!available_beds.is_empty(), "No available beds");

        // 获取第一个可用的床位
        let room_number = *available_beds.first().unwrap();

        // 获取当前时间戳作为入院时间
        let admission_date = env::block_timestamp().to_string();

        // 创建住院信息对象
        let hospitalization = Hospitalization {
            patient: patient_id.clone(),
            room_number,
            admission_date,
            discharge_date: None,
            in_hospital: true,// 更新患者状态为在院
        };

        // 将住院信息存储到合约状态
        self.hospitalizations.insert(&patient_id, &hospitalization);

        // 更新床位状态为已占用
        let mut bed = self.beds.get(&room_number).unwrap();
        bed.is_occupied = true;
        self.beds.insert(&room_number, &bed);

        // 更新患者状态为在院
        // let mut patient = self.patients.get(&patient_id).unwrap();
        // patient.in_hospital = true;
        // self.patients.insert(&patient_id, &patient);
        log_str(&format!("入院登记成功: {patient_id}"));
        true
    }

     // 出院手续办理
     pub fn discharge_patient(&mut self, patient_id: String)  -> bool{
        // 检查患者是否已经注册
        //assert!(self.patients.contains_key(&patient_id), "Patient not registered");
        // 获取调用者账户ID
        let doctor_id = env::signer_account_id().to_string();

        // 确保调用者是医生
        assert!(
            self.identify.get(&doctor_id) == Some("doctor".to_string()),
            "只有医生可以调用此函数"
        );
        // 检查患者是否已入院
        assert!(self.hospitalizations.get(&patient_id).is_some(), "患者未入院");

        // 获取患者的住院信息
        let mut hospitalization = self.hospitalizations.get(&patient_id).unwrap();

        // 设置出院时间为当前时间戳
        hospitalization.discharge_date = Some(env::block_timestamp().to_string());

         // 更新在院状态为 false
        hospitalization.in_hospital = false;

        // 更新住院信息
        self.hospitalizations.insert(&patient_id, &hospitalization);

        // 获取患者的床位号
        let room_number = hospitalization.room_number;

        // 更新床位状态为未占用
        let mut bed = self.beds.get(&room_number).unwrap();
        bed.is_occupied = false;
        self.beds.insert(&room_number, &bed);

        // 更新患者状态为非在院
        // let mut patient = self.patients.get(&patient_id).unwrap();
        // patient.in_hospital = false;
        // self.patients.insert(&patient_id, &patient);
        log_str(&format!("出院登记成功: {patient_id}"));
        true
    }

    // 获取空床位列表
    pub fn get_available_beds(&self) -> Vec<u32> {
        self.beds
            .iter()
            .filter_map(|(room_number, bed)| if !bed.is_occupied { Some(room_number) } else { None })
            .collect()
    }
    // 添加空床位列表
    pub fn add_available_beds(&mut self, room_number: u32)  -> bool{
        let bed = Bed {
            room_number,
            is_occupied: false,
        };
        self.beds.insert(&room_number, &bed);
        true
    }

    // 在医生巡查函数中添加记录巡查的逻辑
    pub fn perform_rounds(&mut self, room_number: u32)  -> bool{
        // 获取调用者账户ID
        let doctor_id = env::signer_account_id().to_string();

        // 确保调用者是医生
        assert!(
            self.identify.get(&doctor_id) == Some("doctor".to_string()),
            "只有医生可以调用此函数"
        );

        if let Some(bed) = self.beds.get(&room_number){
            //assert!(false, "入院登记失败");
            if bed.is_occupied{

            }else{
                assert!(false, "床位为空");
            }
        }else{
             // 如果没有找到，返回 None
             assert!(false, "床位号错误");
        }
        // 获取当前时间戳
        let timestamp = env::block_timestamp().to_string();

        // 创建巡查记录对象
        let rounds_record = RoundsRecord {
            doctor:doctor_id.clone(),
            room_number,
            timestamp,
        };

        // 获取病床对应的巡查记录列表，如果不存在则创建一个新的列表
        let mut records = self.rounds_records.get(&room_number).unwrap_or_else(|| Vec::new());

        // 添加新的巡查记录到列表中
        records.push(rounds_record);

        // 更新巡查记录映射
        self.rounds_records.insert(&room_number, &records);
        log_str(&format!("医生巡查成功: {room_number}"));
        true
    }



    // 患者的访客记录
    pub fn record_visitor(&mut self, patient_id: &String) -> bool {
        let visitor_id = env::signer_account_id().to_string();
        assert_eq!(self.check_role(&visitor_id,"visitor".to_string()),true,"Method caller should register as a visitor.");
        assert_eq!(self.check_role(patient_id,"patient".to_string()),true,"Input patient id isn't valid.");

        let vistior_info = Vistor {
            visitor: visitor_id,
            time: env::block_timestamp().to_string(),
        };

        if self.vistor_list.get(&patient_id).is_some(){
            let mut patient_medicine_vec = self.vistor_list.get(&patient_id).unwrap();
            patient_medicine_vec.push(&vistior_info);
            self.vistor_list.insert(&patient_id, &patient_medicine_vec);
            //self.vistor_list.get(&patient_id).unwrap().push(&vistior_info);
        }else {
            let prefix: Vec<u8> = 
            [
                b"v".as_slice(),
                &near_sdk::env::sha256_array(&patient_id.as_bytes()),
            ].concat();
            let mut new_vector: Vector<Vistor> = Vector::new(prefix);
            new_vector.push(&vistior_info);
            self.vistor_list.insert(&patient_id, &new_vector);
        }

        true

    }

    pub fn get_visitor_list(&self, patient_id: &String) -> Vec<Vistor> {
        if let Some(visitor_vector) = self.vistor_list.get(&patient_id) {
            return visitor_vector.to_vec();
        } else {
            return Vec::new();
        }
    }

    pub fn get_bill_info(&self, patient_id: &String) -> (u32,String,bool) {
        if let Some(bill_info) = self.bill_record.get(&patient_id) {
            return (bill_info.total_amount,bill_info.last_update_time,bill_info.is_paid);
        } else {
            panic!("Could not find the bill of this patient.");
        }
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 */
#[cfg(test)]
mod tests {
    use near_sdk::collections::vector;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use super::*;
    //建立测试环境
    fn set_context(is_view: bool,user:String) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(user.parse().unwrap());
        builder.signer_account_id(user.parse().unwrap());
        builder.is_view(is_view);
        testing_env!(builder.build());
    }

    //测试更变账户角色
    #[test]
    fn register_and_checkrole_test(){
        let mut contract = Contract::default();
        set_context(false,"alice.near".to_string());
        let mut doctor_id = "alice.near".to_string();
        let mut role = "doctor".to_string();
        let grant_role_status =  contract.register(role);
        assert_eq!(grant_role_status,true,"grant doctor role failed.");
        let check_role = contract.check_role(&doctor_id,"doctor".to_string());
        assert_eq!(check_role,true,"check doctor role failed.");

        set_context(false,"bob.near".to_string());
        let mut patient_id = "bob.near".to_string();
        let mut role2 = "patient".to_string();
        let grant_role2_status =  contract.register(role2);
        assert_eq!(grant_role2_status,true,"grant patient role failed.");
        let check_role2 = contract.check_role(&patient_id,"patient".to_string());
        assert_eq!(check_role2,true,"check patient role failed.");
    }

    #[test]
    fn admit_and_discharge_patient_test(){
        // 创建合约实例
        let mut contract = Contract::default();

        // 假设这是一个病人ID
        let context2 = get_context(false,"patient.near".to_string());
        testing_env!(context2);
        let patient_id = "patient.near".to_string();
        contract.identify.insert(&patient_id,&"patient".to_string());

        let context = get_context(false, "doctor.near".to_string());
        testing_env!(context);
        let doctor_id = "doctor.near".to_string();
        contract.identify.insert(&doctor_id,&"doctor".to_string());

        //Write a for loop from 1 to 10 to add_available_beds
        for i in 1..=10 {
            contract.add_available_beds(i);
        }

        let bed_list = contract.get_available_beds();
        assert_eq!(bed_list.len(),10,"available bed num is not correct.");
        
        //入院登记
        let mut result=false;
        result=contract.admit_patient(patient_id.clone());
        assert!(&result, "入院登记失败");

        let bed_list2 = contract.get_available_beds();
        assert_eq!(bed_list2.len(),9,"available bed num is not correct.");

        //查询病人床位
        if let Some(hospitalization) = contract.hospitalizations.get(&patient_id) {
            // 如果找到了，返回包含病房号和在院状态的元组
            //Some((hospitalization.room_number, hospitalization.in_hospital))
            println!("病人在病房号 {}，是否住院中：{}", &hospitalization.room_number, hospitalization.in_hospital);
            let _result=contract.perform_rounds(hospitalization.room_number);
            assert!(_result, "查房失败");

            if let Some(current_rounds_record) = contract.rounds_records.get(&hospitalization.room_number){
                assert_eq!(current_rounds_record.len(),1,"incorrect round record num.");
            } else {
                assert!(false, "查房信息查询失败");
            }

        } else {
            // 如果没有找到，返回 None
            assert!(false, "入院登记失败");
        }
        //出院手续办理
        result=contract.discharge_patient(patient_id.clone());
        assert!(&result, "出院失败");

        //查询病人床位
        if let Some(hospitalization) = contract.hospitalizations.get(&patient_id) {
            // 如果找到了，返回包含病房号和在院状态的元组
            //Some((hospitalization.room_number, hospitalization.in_hospital))
            println!("病人在病房号 {}，是否住院中：{}", &hospitalization.room_number, hospitalization.in_hospital);
            //let _result=contract.perform_rounds(hospitalization.room_number);
            //assert!(_result, "查房失败");

        } else {
            // 如果没有找到，返回 None
            assert!(false, "入院登记失败");
        }

        let bed_list3 = contract.get_available_beds();
        assert_eq!(bed_list3.len(),10,"available bed num is not correct.");
    }

    //测试医生为病人添加病例
    #[test]
    fn add_record_terst(){
        let mut contract = Contract::default();
        let mut role="doctor".to_string();
        let judge=contract.register(role);
        let detail="病情无碍".to_string();
        let judge=contract.add_record("bob.near".to_string(),detail);
        
    }
    //测试添加授权
    #[test]
    fn allow_test(){
        let mut contract = Contract::default();
        
        let mut role1="patient".to_string();
        
        let judge1=contract.register(role1);
        let my_role=contract.add_allow_record("bob.near".to_string());
        println!("{}",my_role);
    }
    fn get_context(is_view: bool,user:String) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(user.parse().unwrap())
            .signer_account_id(user.parse().unwrap())
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_get_user_medical_records() {
        // 创建合约实例
        let mut contract = Contract::default();

        // 设置测试上下文，模拟医生查询病人的病例
        let context = get_context(false,"bob.near".to_string());
        testing_env!(context);

        // 假设这是一个病人ID和医生ID
        let patient_id = "bob.view".to_string();
        let doctor_id = "bob.near".to_string();
        contract.identify.insert(&patient_id,&"patient".to_string());
        contract.identify.insert(&doctor_id,&"doctor".to_string());
        // 假设这是一个病例详情
        let medical_record = MedicalRecord {
            doctor: doctor_id.clone(),
            detail: "Some details".to_string(),
            time: "2023-01-01".to_string(),
        };

        // let mut test_vec:Vector<MedicalRecord>=Vector::new(b"n");
        // test_vec.push(&medical_record);
        // // 将病例添加到患者记录中
        // contract.patient_record.insert(&patient_id, &test_vec);
        log_str(&format!("病人编号:2 {patient_id}"));
        contract.add_record(patient_id.clone(),"patient is already present".to_string());
        // 将医生添加到授权名单中
        let mut authorized_doctors = UnorderedSet::new(b"a".to_vec());
        authorized_doctors.insert(&doctor_id);
        contract.allow_record.insert(&patient_id, &authorized_doctors);

        // 以医生身份调用函数
        let patient_records = contract.get_user_medical_records(patient_id.clone());

        // 验证结果
        assert_eq!(patient_records.len(), 1);

        // 重置上下文为其他测试
        let context = get_context(true,"bob.near".to_string());
        testing_env!(context);
    }
    //测试添加药方，及验证药方及查询药方
    #[test]
    fn test_prescribe_medicine() {
        // 创建合约实例
        let mut contract = Contract::default();

        set_context(false,"patient.near".to_string());
        let role = "patient".to_string();
        let grant_role_status =  contract.register(role);
        set_context(false,"doctor.near".to_string());
        let role2 = "doctor".to_string();
        let grant_role2_status =  contract.register(role2);

        let patient_id = "patient.near".to_string();
        let doctor_id = "doctor.near".to_string();

        // 假设这是一个药物信息
        let medicine = vec![
            Medicine {
                medicine_info: "Information about Medicine A".to_string(),
                medicine_name: "Medicine A".to_string(),
                price: 10
            },
            Medicine {
                medicine_info: "Information about Medicine B".to_string(),
                medicine_name: "Medicine B".to_string(),
                price: 20
            },
        ];

        // 调用医生开药方函数
        let result = contract.prescribe_medicine(patient_id.clone(), medicine.clone());
        let result = contract.prescribe_medicine(patient_id.clone(), medicine.clone());
        // 验证结果
        assert!(result, "开药方失败");

        // 验证药方是否正确添加到患者记录中
        let patient_medicine = contract.patient_medicine.get(&patient_id).unwrap();
        assert_eq!(patient_medicine.len(), 2, "患者记录中应有一条药方");

        let prescribed_medicine = &patient_medicine.get(0).unwrap();;
        assert_eq!(
            prescribed_medicine.medicine.len(),
            2,
            "药方中应包含两种药物"
        );
        //验证药方付款后修改使用状态
        contract.identify.insert(&doctor_id,&"pharmacy".to_string());

        // 重置上下文为其他测试
       
        let result = contract.confirm_payment_and_dispense_medicine(patient_id.clone(), 0);
         // 验证结果
         assert!(result);
       // 验证药方记录是否更新
        let updated_prescription = contract.patient_medicine.get(&patient_id).unwrap().get(0).unwrap();
        assert!(updated_prescription.is_use);

        // 额外检查：在修改后读取 is_use，并验证它是否被正确设置
        let is_use_after_update = updated_prescription.is_use;
        assert!(is_use_after_update);

        //测试查询用户药方情况
        // 以医生身份调用函数
        let mut authorized_doctors = UnorderedSet::new(b"a".to_vec());
        authorized_doctors.insert(&doctor_id);
        contract.allow_record.insert(&patient_id, &authorized_doctors);

        let patient_records = contract.get_user_prescriptions(patient_id.clone());

        // 验证结果
        assert_eq!(patient_records.len(), 2);

        let context = get_context(true, "doctor.near".to_string());
        testing_env!(context);
    }

    //测试预约挂号功能
    #[test]
    #[should_panic(expected = "This doctor isn't avaliable.")]
    fn reservation_test() {
        let mut contract = Contract::default();
        set_context(false,"doctor.near".to_string());
        let role = "doctor".to_string();
        let grant_role_status =  contract.register(role);

        set_context(false,"patient.near".to_string());
        let role2 = "patient".to_string();
        let grant_role2_status =  contract.register(role2);

        let doctor_id = "doctor.near".to_string();
        let status = contract.get_doctor_status(&doctor_id);
        assert_eq!(status,true,"doctor status invalid.");

        let reservation_result = contract.make_reservation(&doctor_id);
        assert_eq!(reservation_result,true,"reservation failed.");

        let new_status = contract.get_doctor_status(&doctor_id);
        assert_eq!(new_status,false,"doctor status should be false.");

        let _ = contract.make_reservation(&doctor_id);//should panic
    }

    #[test]
    #[should_panic(expected =  "Method caller should register as a visitor.")]
    fn visitor_record_test() {
        let mut contract = Contract::default();
        set_context(false,"patient.near".to_string());
        let role = "patient".to_string();
        let grant_role_status =  contract.register(role);

        set_context(false,"visitor1.near".to_string());
        let role2 = "visitor".to_string();
        let grant_role2_status = contract.register(role2);

        let patient_id = "patient.near".to_string();
        let result = contract.record_visitor(&patient_id);

        assert_eq!(result,true,"visitor record failed.");
        let visitor_vec = contract.get_visitor_list(&patient_id);
        assert_eq!(visitor_vec.len(),1,"visitor vector num is not correct.");
        
        set_context(false,"visitor2.near".to_string());
        let role3 = "doctor".to_string();
        let grant_role3_status = contract.register(role3);
        let result2 = contract.record_visitor(&patient_id);
        let visitor_vec2 = contract.get_visitor_list(&patient_id);
        assert_eq!(visitor_vec2.len(),1,"visitor vector num is not correct.");
    }

    #[test]
    #[should_panic(expected = "This patient has paid his/her bill.")]
    fn bill_test() {
        let mut contract = Contract::default();
        set_context(false,"patient.near".to_string());
        let role = "patient".to_string();
        let grant_role_status =  contract.register(role);

        let patient_id = "patient.near".to_string();
        let (amount,_,paid) = contract.get_bill_info(&patient_id);

        assert_eq!(amount,0,"invalid bill total amount.");
        assert_eq!(paid,false,"invalid bill status.");

        set_context(false,"doctor.near".to_string());
        let role2 = "doctor".to_string();
        let grant_role2_status =  contract.register(role2);

        let medicine = vec![
            Medicine {
                medicine_info: "Information about Medicine A".to_string(),
                medicine_name: "Medicine A".to_string(),
                price: 10
            },
            Medicine {
                medicine_info: "Information about Medicine B".to_string(),
                medicine_name: "Medicine B".to_string(),
                price: 20
            },
        ];
        
        // 调用医生开药方函数
        let result = contract.prescribe_medicine(patient_id.clone(), medicine.clone());

        // 验证结果
        assert!(result, "开药方失败");

        let (new_amount,_,new_paid) = contract.get_bill_info(&patient_id);

        assert_eq!(new_amount,30,"invalid bill total amount.");
        assert_eq!(new_paid,false,"invalid bill status.");

        set_context(false,"patient.near".to_string());
        let patient_id = "patient.near".to_string();
        let payment_result = contract.pay_the_bill(&patient_id,10);
        assert_eq!(payment_result,false,"payment should failed.");
        let bill_status = contract.check_bill_is_paid(&patient_id);
        assert_eq!(bill_status,false,"bill ought to haven't been paid.");

        let payment_result2 = contract.pay_the_bill(&patient_id,30);
        assert_eq!(payment_result2,true,"payment should success.");
        let bill_status2 = contract.check_bill_is_paid(&patient_id);
        assert_eq!(bill_status2,true,"bill should has been paid.");

        let payment_result3 = contract.pay_the_bill(&patient_id,50);
    }

}