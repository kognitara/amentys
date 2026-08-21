use x86_64::instructions::port::Port;

/// Reads a value from a specified Real-Time Clock (RTC) register.
/// # Arguments
/// * `reg` - A u8 value representing the RTC register to read from
/// # Returns
/// A u8 value read from the specified RTC register.
pub fn read_rtc_register(reg: u8) -> u8 {
    unsafe {
        // Safety: Accessing hardware ports is inherently unsafe, but we are using the correct ports for the RTC.
        let mut index = Port::<u8>::new(0x70);
        let mut data = Port::<u8>::new(0x71);

        index.write(reg);
        data.read()
    }
}

/// Converts a Binary-Coded Decimal (BCD) value to its binary equivalent.
///
/// # Arguments
/// * `value` - A u8 value representing a BCD number.
///
/// # Returns
/// A u8 value representing the binary equivalent of the input BCD value.
///
pub fn bcd_to_bin(value: u8) -> u8 {
    ((value & 0xF0) >> 1) + ((value & 0xF0) >> 3) + (value & 0x0F)
}
/// Reads the current date and time from the Real-Time Clock (RTC) and returns it as a tuple of six u8 values: (day, month, year, hour, minute, second).
///
/// # Returns
/// A tuple containing the current date and time in the format (day, month, year, hour, minute, second).
///
pub fn get_rtc_date() -> (u8, u8, u8, u8, u8, u8) {
    let second = bcd_to_bin(read_rtc_register(0x00));
    let minute = bcd_to_bin(read_rtc_register(0x02));
    let hour = bcd_to_bin(read_rtc_register(0x04));
    let day = bcd_to_bin(read_rtc_register(0x07));
    let month = bcd_to_bin(read_rtc_register(0x08));
    let year = bcd_to_bin(read_rtc_register(0x09));

    (day, month, year, hour, minute, second)
}

/// A struct representing the current date and time, as read from the Real-Time Clock (RTC). It contains fields for the year, month, day, hour, minutes, and seconds.
///
/// # Fields
/// - `year`: The current year (last two digits).
/// - `month`: The current month (1-12).
/// - `day`: The current day of the month (1-31).
/// - `hour`: The current hour (0-23).
/// - `minutes`: The current minutes (0-59).
/// - `secondes`: The current seconds (0-59).
///
/// # Example
/// ```no_run
/// let current_date = UtcDate::new();
/// println!("Current date and time: {:02}/{:02}/20{:02} {:02}:{:02}:{:02}", current_date.day, current_date.month, current_date.year, current_date.hour, current_date.minutes, current_date.secondes);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UtcDate {
    pub year: u8,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minutes: u8,
    pub secondes: u8,
}

impl UtcDate {
    /// Creates a new instance of `UtcDate` by reading the current date and time from the Real-Time Clock (RTC).
    /// # Returns
    /// A new `UtcDate` instance containing the current date and time.
    pub fn new() -> Self {
        let (day, month, year, hour, minutes, secondes): (u8, u8, u8, u8, u8, u8) = get_rtc_date();
        Self {
            year,
            month,
            day,
            hour,
            minutes,
            secondes,
        }
    }

    /// Converts the RTC date and time to a UNIX timestamp (seconds since Jan 1, 1970).
    /// This is required by `awq` to handle ephemeral plan expirations.
    #[must_use]
    pub fn to_timestamp(&self) -> u64 {
        // L'année renvoyée par le RTC est sur 2 chiffres (ex: 26 pour 2026)[cite: 10]
        let y = 2000 + self.year as u64;
        let m = self.month as u64;
        let d = self.day as u64;

        // Algorithme de conversion vers le nombre de jours depuis 1970
        let a = (14 - m) / 12;
        let y_adj = y + 4800 - a;
        let m_adj = m + 12 * a - 3;

        let julian_day =
            d + (153 * m_adj + 2) / 5 + 365 * y_adj + y_adj / 4 - y_adj / 100 + y_adj / 400 - 32045;
        let days_since_1970 = julian_day - 2440588; // 2440588 est le Jour Julien du 1er Janvier 1970

        days_since_1970 * 86400
            + (self.hour as u64) * 3600
            + (self.minutes as u64) * 60
            + (self.secondes as u64)
    }
}

impl Default for UtcDate {
    fn default() -> Self {
        Self::new()
    }
}
