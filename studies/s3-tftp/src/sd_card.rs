use core::mem::size_of;
use core::cell::{Cell, RefCell};

use embassy_time::Instant;
use esp_hal::{
    gpio::Output,
    delay::Delay,
    spi::master::{SpiDmaBus, Config as SpiConfig},
};
use embedded_hal::spi::{SpiBus, SpiDevice as SyncSpiDevice, Operation as SyncOp};
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    signal::Signal
};
use embedded_sdmmc::{BlockDevice, BlockIdx, Block};

use crate::{
    SharedSpiBus,
    gps::{GPS_DATA, GPS_UPDATED},
};

pub const FORMAT_NEEDED:       bool      = true;
//pub const MASTER_OFFSET:       u32     = 0;
pub const MASTER_OFFSET:       u32     = 15 * 1024 / 512 * 1024 * 1024;

pub const MAGIC:               [u8; 4] = *b"CCMG";
pub const VERSION:             u8      = 1;
pub const RECORD_SIZE:         u8      = 62;
pub const RECORDS_PER_SECTOR:  u8      = 8;
pub const INDEX_START:         u32      = 1 + MASTER_OFFSET;
pub const INDEX_SECTORS:       u16     = 256;
pub const DATA_START:          u32     = 257 + MASTER_OFFSET;
pub const ENTRIES_PER_SECTOR:  u32     = 32;    // 512 / 16
// 256 * 32 = 8192 days -> ~22 years

type BusGuard<'a> = embassy_sync::mutex::MutexGuard<
    'a, NoopRawMutex, SpiDmaBus<'static, esp_hal::Async>
>;

pub struct SdSpiBlockingProxy<'a> {
    pub bus: &'a SharedSpiBus,
    pub cs_pin: RefCell<&'a mut esp_hal::gpio::Output<'static>>,
    // UnsafeCell lets us modify the guard using a shared reference (&self)
    pub active_guard: core::cell::UnsafeCell<Option<BusGuard<'a>>>,
    // Controls when it is safe to jump to 16 MHz
    pub use_fast_speed: Cell<bool>,
}

impl<'a> SdSpiBlockingProxy<'a> {
    pub fn new(bus: &'a SharedSpiBus, cs_pin: &'a mut esp_hal::gpio::Output<'static>) -> Self {
        Self {
            bus,
            cs_pin: RefCell::new(cs_pin),
            active_guard: core::cell::UnsafeCell::new(None),
            use_fast_speed: Cell::new(false),
        }
    }

    pub async fn lock_bus_master(&self) {
        let guard = self.bus.lock().await;
        unsafe { *self.active_guard.get() = Some(guard); }
    }

    // Call this after your high-level SD / Storage operations are finished
    pub fn unlock_bus_master(&self) {
        unsafe { *self.active_guard.get() = None; }
    }

    fn do_transaction(&self, operations: &mut [SyncOp<'_, u8>]) -> Result<(), esp_hal::spi::Error> {
        // we need a master lock for the entire sd card operation
        // not only for its atomic spi bus operations
        let guard_slot = unsafe { &mut *self.active_guard.get() };

        let bus_guard = match guard_slot {
            Some(g) => g,
            None => {
                // Should never happen in normal operation
                panic!("SD transaction called without holding bus lock");
            }
        };

        // TARGET FREQUENCY DECISION:
        // Only override to 16 MHz if our loop explicitly turned on fast mode.
        // Otherwise, force 400 kHz to protect the card's early handshakes!
        let target_speed = if self.use_fast_speed.get() {
            esp_hal::time::Rate::from_mhz(16)
        } else {
            esp_hal::time::Rate::from_khz(400)
        };

        let _ = bus_guard.apply_config(&SpiConfig::default()
            .with_frequency(target_speed)
            .with_mode(esp_hal::spi::Mode::_0)
        );

        self.cs_pin.borrow_mut().set_low();

        for op in operations {
            match op {
                SyncOp::Read(buf) => {
                    SpiBus::read(&mut **bus_guard, buf)?;
                }
                SyncOp::Write(buf) => {
                    SpiBus::write(&mut **bus_guard, buf)?;
                }
                SyncOp::Transfer(read, write) => {
                    SpiBus::transfer(&mut **bus_guard, read, write)?;
                }
                SyncOp::TransferInPlace(buf) => {
                    SpiBus::transfer_in_place(&mut **bus_guard, buf)?;
                }
                SyncOp::DelayNs(ns) => {
                    esp_hal::delay::Delay::new().delay_nanos(*ns);
                }
            }
        }

        // Release Chip Select
        self.cs_pin.borrow_mut().set_high();

        Ok(())
    }
}

impl<'a> embedded_hal::spi::ErrorType for &SdSpiBlockingProxy<'a> {
    type Error = esp_hal::spi::Error;
}

impl<'a> SyncSpiDevice<u8> for &SdSpiBlockingProxy<'a> {
    fn transaction(&mut self, operations: &mut [SyncOp<'_, u8>]) -> Result<(), Self::Error> {
        (**self).do_transaction(operations)   // self: &mut &SdSpiBlockingProxy — no aliasing issue
    }
}

/// Sector 0 - written once at format time, never updated
#[repr(C, packed)]
pub struct FormatHeader {
    pub magic:              [u8; 4],
    pub version:            u8,
    pub record_size:        u8,
    pub records_per_sector: u8,
    pub index_start:        u32,        // = 1 + MASTER_OFFSET
    pub index_sectors:      u16,        // = 256
    pub _pad:               [u8; 2],
    pub data_start:         u32,        // = 257 + MASTER_OFFSET
    pub _reserved:          [u8; 493],
}

#[repr(C, packed)]
pub struct DayEntry {   
    pub date:         [u8; 6],  // "260516" YYMMDD
    pub _pad:         [u8; 2],
    pub start_sector: u32,      // absolute sector of first data sector
    pub _pad2:        [u8; 4],  // = 16 bytes total
}

#[repr(C, packed)]
pub struct DataSector {
    //pub sequence:     u32,              // 0xFFFFFFFF = empty
    pub sequence:     u32,              // 0x00000000 = empty
    pub day_index:    u16,              // index into DayEntry array
    pub record_count: u8,               // valid records in this sector (0-8)
    pub _pad:         [u8; 9],          // pad header to 16 bytes
    pub records:      [[u8; 62]; 8],    // 8 × 62 = 496 bytes
}

pub struct Storage<D> {
    dev:                D,          // SdCard
    day_index:          u16,        // current day's index (0-based)
    current_sector:     u32,        // absolute sector being written
    current_sequence:   u32,        // monotonically increasing
    sector_buf:         DataSector, // current sector (write) buffer in RAM
    read_buf:           Block,      // read buffer, reused, zero copy
}

impl<'a, D: BlockDevice> Storage<D>
where
    D::Error: core::fmt::Debug,
{
    // low level block read/write

    fn read_block(&mut self, sector: u32) -> Result<&Block, D::Error> {
        // if read_block in a fast loop - e.g. in mount scan    
        Delay::new().delay_micros(200);
        self.dev.read(
            core::slice::from_mut(&mut self.read_buf),
            BlockIdx(sector)
        )?;
        Ok(&self.read_buf)
    }

    fn write_block(&mut self, sector: u32, block: &Block) -> Result<(), D::Error> {
        let mut last_err = None;
        for _attempt in 0..3 {
            match self.dev.write(
                core::slice::from_ref(block),
                BlockIdx(sector)
            ) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    cfg_if::cfg_if! {
                        if #[cfg(feature = "defmt")] {
                            warn!("write_block {} attempt {} failed: {:?}",
                                sector, _attempt, defmt::Debug2Format(&e));
                        } else {
                            warn!("write_block failed");
                        }
                    }
                    last_err = Some(e);
                    Delay::new().delay_millis(10);
                }
            }
        }
        last_err.map_or(Ok(()), Err)
    }


    fn block_from<T: Sized>(val: &T) -> Block {
        let mut block = Block::new();
        unsafe {
            core::ptr::copy_nonoverlapping(
                val as *const T as *const u8,
                block.contents.as_mut_ptr(),
                size_of::<T>(),
            );
        }
        block
    }

    pub fn format(dev: D) -> Result<Self, D::Error> {
        let mut storage = Self {
            dev,
            day_index:        0xFFFF,
            current_sector:   DATA_START,
            current_sequence: 1, // since we changed sentinel FF to 00
            sector_buf:       unsafe { core::mem::zeroed() },
            read_buf:         Block::new(),
        };

        let header = FormatHeader {
            magic:              MAGIC,
            version:            VERSION,
            record_size:        RECORD_SIZE,
            records_per_sector: RECORDS_PER_SECTOR,
            index_start:        INDEX_START,
            index_sectors:      INDEX_SECTORS,
            _pad:               [0; 2],
            data_start:         DATA_START,
            _reserved:          [0; 493],
        };
        let block = Self::block_from(&header);
        storage.write_block(MASTER_OFFSET, &block)?;

        info!("Storage format done");
        Ok(storage)
    }

    // resume after boot
    pub fn mount(dev: D) -> Result<Self, D::Error> {
        let mut storage = Self {
            dev,
            day_index:        0xFFFF,
            current_sector:   DATA_START,
            current_sequence: 1, // since we changed sentinel FF to 00
            sector_buf:       unsafe { core::mem::zeroed() },
            read_buf:         Block::new()  
        };

        // verify magic
        let magic_ok = storage.read_block(MASTER_OFFSET)
            .ok()
            .map(|block| block.contents[0..4] == MAGIC)
            .unwrap_or(false);

        if !magic_ok {
            if FORMAT_NEEDED {
                warn!("Invalid magic, formatting...");
                return Self::format(storage.dev);
            } else {
                panic!("SD card corrupted");
            }
        }

        // scan index to find last day entry
        'outer: for idx_sector in 0..INDEX_SECTORS as u32 {
            debug!("storage.read_block({})", INDEX_START + idx_sector);
            let block = storage.read_block(INDEX_START + idx_sector)?;
            // copy contents to avoid borrow issues
            let contents = block.contents;
            
            // if we would not have been copy contents above
            // this slice (entries) would lock self (since block is reference into self.read_buf)
            // so at the next iteration storage.read_block would be failed
            let entries = unsafe {
                core::slice::from_raw_parts(
                    contents.as_ptr() as *const DayEntry,
                    ENTRIES_PER_SECTOR as usize,
                )
            };
            
            for (i, entry) in entries.iter().enumerate() {
                let first_byte = unsafe {
                    core::ptr::read_unaligned(
                        core::ptr::addr_of!(entry.date[0])
                    )
                };
                //if first_byte == 0xFF {
                if first_byte == 0x00 {
                    let global_idx = idx_sector * ENTRIES_PER_SECTOR + i as u32;
                    if global_idx == 0 {
                        storage.day_index = 0xFFFF;
                        storage.current_sector = DATA_START;
                    } else {
                        storage.day_index = (global_idx - 1) as u16;
                    }
                    break 'outer;
                }
                if idx_sector == INDEX_SECTORS as u32 - 1
                    && i == ENTRIES_PER_SECTOR as usize - 1
                {
                    panic!("Card is full - replace it after 22 years of continues use");
                }
            }
        }

        let mut search_counter: u32 = 0;

        // find next free data sector by scanning from last day's start
        if storage.day_index != 0xFFFF {
            let start_sector = storage.read_day_start_sector(storage.day_index)?;
            let mut sector = start_sector;
            loop {
                let block = storage.read_block(sector)?;
                search_counter += 1;
                let sequence = unsafe {
                    core::ptr::read_unaligned(
                        block.contents.as_ptr() as *const u32
                    )
                };
                //debug!("sector {} sequence 0x{:x}", sector, sequence);
                if sequence == 0x00000000 {
                    storage.current_sector = sector;
                    break;
                }
                storage.current_sequence = sequence + 1;
                sector += 1;
            }
        }

        info!(
            "Storage mounted: day_index={} current_sector={} sequence={} - {} read was needed",
            storage.day_index,
            storage.current_sector,
            storage.current_sequence,
            search_counter,
        );

        Ok(storage)
    }

    // index helpers

    fn read_day_start_sector(&mut self, index: u16) -> Result<u32, D::Error> {
        let sector = INDEX_START + (index as u32 / ENTRIES_PER_SECTOR);
        let offset = (index as u32 % ENTRIES_PER_SECTOR) as usize;
        let block = self.read_block(sector)?;
        let contents = block.contents;
        let ptr = unsafe {
            (contents.as_ptr() as *const DayEntry).add(offset)
        };
        Ok(unsafe {
            core::ptr::read_unaligned(
                core::ptr::addr_of!((*ptr).start_sector)
            )
        })
    }

    pub fn read_day_date(&mut self, index: u16) -> Result<[u8; 6], D::Error> {
        let sector = INDEX_START + (index as u32 / ENTRIES_PER_SECTOR);
        let offset = (index as u32 % ENTRIES_PER_SECTOR) as usize;
        let block = self.read_block(sector)?;
        let contents = block.contents;
        let ptr = unsafe {
            (contents.as_ptr() as *const DayEntry).add(offset)
        };
        Ok(unsafe {
            core::ptr::read_unaligned(core::ptr::addr_of!((*ptr).date))
        })
    }

    fn write_day_entry(&mut self, index: u16, entry: &DayEntry) -> Result<(), D::Error> {
        let sector = INDEX_START + (index as u32 / ENTRIES_PER_SECTOR);
        let offset = (index as u32 % ENTRIES_PER_SECTOR) as usize;

        // read existing index sector
        let mut block = {
            let b = self.read_block(sector)?;
            b.clone() // necessary here - need owned copy to modify
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                entry as *const DayEntry as *const u8,
                block.contents.as_mut_ptr()
                    .add(offset * size_of::<DayEntry>()),
                size_of::<DayEntry>(),
            );
        }
        self.write_block(sector, &block)
    }

    // new day

    pub fn new_day(&mut self, date: &[u8; 6]) -> Result<(), D::Error> {
        self.flush_sector()?;

        let new_index = if self.day_index == 0xFFFF { 0 }
                        else { self.day_index + 1 };

        let mut entry_date = [0u8; 6];
        entry_date.copy_from_slice(date);

        let entry = DayEntry {
            date:         entry_date,
            _pad:         [0; 2],
            start_sector: self.current_sector,
            _pad2:        [0; 4],
        };

        self.write_day_entry(new_index, &entry)?;
        self.day_index = new_index;

        info!(
            "New day: index={} start_sector={}",
            new_index,
            self.current_sector,
        );

        Ok(())
    }

    // write record

    pub fn write_record(&mut self, record: &[u8; 62]) -> Result<(), D::Error> {
        let count = self.sector_buf.record_count as usize;
        self.sector_buf.records[count].copy_from_slice(record);
        self.sector_buf.record_count += 1;

        // it is an important invariant
        // penalty: if power break we can loose max the last 7 seconds's log
        // the max 8 read-modify-write cycles are not problem
        // advantage: simpler code - we don't need to manage partially written sectors
        if self.sector_buf.record_count == RECORDS_PER_SECTOR {
            self.flush_sector()?;
        }

        Ok(())
    }

    // flush current sector to card
    fn flush_sector(&mut self) -> Result<(), D::Error> {
        if self.sector_buf.record_count == 0 {
            return Ok(());
        }

        self.sector_buf.sequence  = self.current_sequence;
        self.sector_buf.day_index = self.day_index;

        let block = Self::block_from(&self.sector_buf);

        // physical writes (for logs) happen only here (buffered)
        let start = Instant::now();
        let write_result =self.write_block(self.current_sector, &block);
        // it's typically 2 ms, if it need to wait its Mutex or other tasks, it's max 5ms
        debug!("write took {}ms", (Instant::now() - start).as_millis());

        self.current_sector   += 1;
        self.current_sequence += 1;
        self.sector_buf        = unsafe { core::mem::zeroed() };

        write_result?;
        Ok(())
    }

    pub fn dump_data_sectors(&mut self, count: u32) -> Result<(), D::Error> {
        for sector in DATA_START..DATA_START + count {
            let block = self.read_block(sector)?;
            let contents = block.contents;
            
            // interpret as DataSector
            let seq = u32::from_le_bytes(unwrap!(contents[0..4].try_into()));
            let day = u16::from_le_bytes(unwrap!(contents[4..6].try_into()));
            let rec_count = contents[6];
            
            //if seq == 0xFFFFFFFF {
            if seq == 0x00000000 {
                info!("sector {}: [empty]", sector);
                continue;
            }
            
            info!(
                "sector {}: seq={} day={} records={}",
                sector, seq, day, rec_count
            );
            
            // dump each record as hex
            for r in 0..rec_count.min(RECORDS_PER_SECTOR) as usize {
                let offset = 16 + r * RECORD_SIZE as usize;
                let record = &contents[offset..offset + RECORD_SIZE as usize];
                // log in 16-byte chunks (defmt can't do long slices)
                info!("  rec {}: {:x}", r, record[0..16]);
                info!("         {:x}", record[16..32]);
                info!("         {:x}", record[32..48]);
                info!("         {:x}", record[48..62]);
            }
        }
        Ok(())
    }


    // tftp: find day by date
    pub fn find_day(&mut self, date: &[u8; 6]) -> Result<Option<(u16, u32)>, D::Error> {
        for idx_sector in 0..INDEX_SECTORS as u32 {
            let block = self.read_block(INDEX_START + idx_sector)?;
            let contents = block.contents;
            let entries = unsafe {
                core::slice::from_raw_parts(
                    contents.as_ptr() as *const DayEntry,
                    ENTRIES_PER_SECTOR as usize,
                )
            };
            for (i, entry) in entries.iter().enumerate() {
                let first_byte = unsafe {
                    core::ptr::read_unaligned(core::ptr::addr_of!(entry.date[0]))
                };
                //if first_byte == 0xFF {
                if first_byte == 0x00 {
                    return Ok(None);
                }
                let entry_date = unsafe {
                    core::ptr::read_unaligned(core::ptr::addr_of!(entry.date))
                };
                if &entry_date == date {
                    let global_idx = (idx_sector * ENTRIES_PER_SECTOR + i as u32) as u16;
                    let start = unsafe {
                        core::ptr::read_unaligned(core::ptr::addr_of!(entry.start_sector))
                    };
                    return Ok(Some((global_idx, start)));
                }
            }
        }
        Ok(None)
    }

    // tftp: read sectors for a day
    pub fn read_day_sector(&mut self, sector: u32) -> Result<Option<DataSector>, D::Error> {
        let block = self.read_block(sector)?;
        let sequence = unsafe {
            core::ptr::read_unaligned(block.contents.as_ptr() as *const u32)
        };
        //if sequence == 0xFFFFFFFF {
        if sequence == 0x00000000 {
            return Ok(None);
        }
        Ok(Some(unsafe {
            core::ptr::read_unaligned(block.contents.as_ptr() as *const DataSector)
        }))
    }

    // tftp: ls like function generating an INDEX file
    pub fn list_days<F>(&mut self, mut callback: F) -> Result<(), D::Error>
    where
        F: FnMut(&[u8; 6]),
    {
        for idx_sector in 0..INDEX_SECTORS as u32 {
            let block = self.read_block(INDEX_START + idx_sector)?;
            let contents = block.contents;
            let entries = unsafe {
                core::slice::from_raw_parts(
                    contents.as_ptr() as *const DayEntry,
                    ENTRIES_PER_SECTOR as usize,
                )
            };
            for entry in entries.iter() {
                let first_byte = unsafe {
                    core::ptr::read_unaligned(core::ptr::addr_of!(entry.date[0]))
                };
                //if first_byte == 0xFF { return Ok(()); }
                if first_byte == 0x00 { return Ok(()); }
                let date = unsafe {
                    core::ptr::read_unaligned(core::ptr::addr_of!(entry.date))
                };
                callback(&date);
            }
        }
        Ok(())
    }
}

#[embassy_executor::task]
pub(crate) async fn sd_task(
    spi_bus: &'static SharedSpiBus,
    sd_ready: &'static Signal<NoopRawMutex, ()>,
    mut cs_pin: Output<'static>) {

    // Create the proxy instance
    let proxy = SdSpiBlockingProxy::new(spi_bus, &mut cs_pin);
    
    proxy.lock_bus_master().await;

    let mut storage = 'init: loop {
        for attempt in 0..16 {
            // Ensure proxy starts at 400 kHz for this handshake attempt
            proxy.use_fast_speed.set(false);

            let card = embedded_sdmmc::SdCard::new(
                &proxy,
            Delay::new(),
            );


            match card.num_bytes() {
                Ok(size) => {
                    trace!("SD init OK: {} bytes", size);

                    // Safely scale up to 16 MHz for all future work.
                    proxy.use_fast_speed.set(true);

                    // Storage takes ownership
                    match Storage::mount(card) {
                        Ok(storage) => break 'init storage,
                        Err(e) if attempt < 15 => {
                            warn!("Mount failed: {:?}, retrying...", e);
                            embassy_time::Timer::after_millis(500).await;
                            continue;
                        }
                        Err(e) => panic!("Mount failed after 15 attempts: {:?}", e),
                    }
                }
                Err(e) if attempt < 15 => {
                    embassy_time::Timer::after_millis(100).await;
                    trace!("attempt {} error {}", attempt, e);
                    continue;
                }
                Err(e) => {
                    panic!("SD init failed: {:?}", e);
                }
            }

        }
    };

    proxy.unlock_bus_master();

    // signaling handshake finised to LCD task
    sd_ready.signal(());
    
    // get GPS_UPDATES watch receiver
    let mut gps_updated = unwrap!(GPS_UPDATED.receiver());

    // wait for gps fix
    gps_updated.changed().await;

    // get current date
    let date = unsafe {
        core::ptr::read_unaligned(
            core::ptr::addr_of!(GPS_DATA.date)
        )
    };

    // only create new day entry if date differs from last stored day
    let need_new_day = if storage.day_index == 0xFFFF {
        true
    } else {
        proxy.lock_bus_master().await;
        let r = match storage.read_day_date(storage.day_index) {
            Ok(last_date) => last_date != date,
            Err(_) => true,
        };
        proxy.unlock_bus_master();
        r
    };

    if need_new_day {
        proxy.lock_bus_master().await;
        unwrap!(storage.new_day(&date));
        proxy.unlock_bus_master();
    }

    let mut current_date = date;

    loop {
        let date = unsafe {
            core::ptr::read_unaligned(core::ptr::addr_of!(GPS_DATA.date))
        };

        unsafe {
            // date changed
            if date != current_date {
                proxy.lock_bus_master().await;
                unwrap!(storage.new_day(&date));
                proxy.unlock_bus_master();
                current_date = date;
            }

            // write record
            let record = core::slice::from_raw_parts(
                &raw const GPS_DATA as *const u8,
                62,
            );
            proxy.lock_bus_master().await;
            storage.write_record(unwrap!(record.try_into())).unwrap_or_else(
                |error| debug!("*** write error {} ***", error)
            );
            proxy.unlock_bus_master();
        }

        // wait for next GPS update
        gps_updated.changed().await;
    }
}
