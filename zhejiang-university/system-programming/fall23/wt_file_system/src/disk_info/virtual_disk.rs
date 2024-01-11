use std::mem::size_of;
use serde::{Deserialize, Serialize};

pub const BLOCK_COUNT: usize = 1000;        // 块数量
pub const BLOCK_SIZE: usize = 1024 * 4;     // 块大小：4KB

pub const EOF_BYTE: u8 = 255;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FatStatus {
    UnUsed,           // 未使用的块
    NextBlock(usize), // 下一块块号
    EOF,              // 结束标志
}

#[derive(Serialize, Deserialize)]
pub struct VirtualDisk {
    pub fat: Vec<FatStatus>,
    data: Vec<u8>,
}

impl VirtualDisk {
    pub fn new() -> VirtualDisk {
        VirtualDisk {
            // FAT
            fat: vec![FatStatus::UnUsed; BLOCK_COUNT],
            // 数据区
            data: vec![
                0u8;
                (BLOCK_COUNT - size_of::<FatStatus>() * BLOCK_COUNT / BLOCK_SIZE - 1)
                    * BLOCK_SIZE
            ],
        }
    }

    // 向disk中的data插入数据。插入数据将覆写相应的位置。
    pub fn insert_data_by_block(&mut self, data: &[u8], block: usize) {
        self.insert_data_by_offset(data, block * BLOCK_SIZE);
    }

    // 向disk的data中插入数据。插入的数据将覆写相应位置的数据。
    pub fn insert_data_by_offset(&mut self, data: &[u8], offset: usize) {
        self.data
            .splice(offset..(offset + data.len()), data.iter().cloned());
    }


    // 向disk中的data插入数据。插入数据将覆写相应的位置。
    pub fn write_data_by_blocks_with_eof(
        &mut self,
        data: &[u8],
        blocks: &[usize],
        insert_eof: bool,
    ) {
        for i in 0..blocks.len() {
            if i < blocks.len() - 1 {
                // 正常分BLOCK_SIZE写入块
                self.insert_data_by_block(
                    &data[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE],
                    blocks[i],
                );
            } else {
                // 开始写入最后一个块
                let mut buffer: Vec<u8> = Vec::with_capacity(BLOCK_SIZE);
                buffer.extend((&data[i * BLOCK_SIZE..data.len()]).iter());
                if insert_eof {
                    // 插入EoF
                    buffer.push(EOF_BYTE);
                }
                if buffer.len() < BLOCK_SIZE {
                    // 若未到 BLOCK_SIZE 则用0填充
                    let mut zero = vec![0u8; BLOCK_SIZE - buffer.len()];
                    buffer.append(&mut zero);
                }
                self.insert_data_by_block(buffer.as_slice(), blocks[i])
            }
        }
    }

    // 从disk中读取数据。
    pub fn read_data_by_block(&self, block: usize) -> Vec<u8> {
        (&self.data[block * BLOCK_SIZE..(block + 1) * BLOCK_SIZE]).to_vec()
    }

    // 根据给出的块号，读出所有数据，并且检测EoF。
    pub fn read_data_by_blocks_without_eof(&self, blocks: &[usize]) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::with_capacity(blocks.len() * BLOCK_SIZE);

        // 循环读出所有数据
        for block in blocks {
            let mut buffer: Vec<u8> = self.read_data_by_block(*block);
            data.append(&mut buffer);
        }
        // 从后向前查找，从EoF开始截断。若未找到EoF则直接返回。
        for i in 1..BLOCK_SIZE {
            let index = data.len() - i;
            if data[index] == EOF_BYTE {
                // 不加不减，刚好将EoF截断在外
                data.truncate(index);
                break;
            }
        }
        data
    }
}
