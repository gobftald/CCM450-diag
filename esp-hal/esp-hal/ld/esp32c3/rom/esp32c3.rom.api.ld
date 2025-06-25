/**
 * ROM APIs
 */

/* 4 */
PROVIDE ( esp_rom_crc32_le = crc32_le );

/* 58 */
PROVIDE ( esp_rom_regi2c_read = rom_i2c_readReg );
/* 60 */
PROVIDE ( esp_rom_regi2c_write = rom_i2c_writeReg );