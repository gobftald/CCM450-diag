use super::AdapterError;

fn char_to_num(char: u8) -> Result<u8, AdapterError> {
    if char.is_ascii_digit() {
        Ok(char - b'0')
    } else if (b'A'..=b'F').contains(&char) {
        Ok(char - b'A' + 10)
    } else {
        Err(AdapterError::ParseIntError)
    }
}

pub fn from_ascii_bytes_to_u8(buf: &[u8]) -> Result<u8, AdapterError> {
    if buf.len() != 2 {
        return Err(AdapterError::ParseIntError);
    }

    let mut val = char_to_num(buf[0])? << 4;
    val += char_to_num(buf[1])?;

    Ok(val)
}

pub fn from_ascii_bytes_to_u16(buf: &[u8]) -> Result<u16, AdapterError> {
    if buf.len() < 4 {
        return Err(AdapterError::ParseIntError);
    }

    let mut i = 0;
    let mut val = (from_ascii_bytes_to_u8(&buf[i..i + 2])? as u16) << 8;
    i += 2;
    while buf[i] == b' ' {
        i += 1;
        continue;
    }
    val += from_ascii_bytes_to_u8(&buf[i..i + 2])? as u16;

    Ok(val)
}

pub fn from_u8_to_ascii_bytes(value: u8, buf: &mut [u8]) -> Result<(), AdapterError> {
    if buf.len() < 2 {
        return Err(AdapterError::ParseIntError);
    }

    let digits = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'A', b'B', b'C', b'D', b'E',
        b'F',
    ];
    buf[0] = digits[(value / 16) as usize];
    buf[1] = digits[(value % 16) as usize];

    Ok(())
}

pub fn from_u16_to_ascii_bytes(value: u16, buf: &mut [u8]) -> Result<(), AdapterError> {
    if buf.len() < 4 {
        return Err(AdapterError::ParseIntError);
    }

    from_u8_to_ascii_bytes((value / 256) as u8, &mut buf[0..2])?;
    from_u8_to_ascii_bytes((value % 256) as u8, &mut buf[2..4])?;

    Ok(())
}

pub fn compare_bytes(one: &[u8], another: &[u8]) -> bool {
    if one.len() != another.len() {
        return false;
    }
    for (i, one) in one.iter().enumerate() {
        if *one != another[i] {
            return false;
        }
    }
    true
}
