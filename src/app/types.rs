use crate::app::gen::enums::{OpType, TripCloseCode};
use crate::app::gen::variations::gv::Variation;
use chrono::{DateTime, SecondsFormat, TimeZone, Utc};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Timestamp {
    pub value: u64,
}

impl Timestamp {
    pub const MAX_VALUE: u64 = 0x0000_FFFF_FFFF_FFFF;
    pub const OUT_OF_RANGE: &'static str = "<out of range>";

    pub fn new(value: u64) -> Self {
        Self {
            value: value & Self::MAX_VALUE,
        }
    }

    pub(crate) fn checked_add(self, x: u16) -> Option<Timestamp> {
        if self.value > Self::MAX_VALUE {
            return None;
        }
        let max_add = Self::MAX_VALUE - self.value;
        if x as u64 > max_add {
            return None;
        }
        Some(Timestamp::new(self.value + x as u64))
    }

    pub fn min() -> Self {
        Self::new(0)
    }

    pub fn max() -> Self {
        Self::new(Self::MAX_VALUE)
    }

    pub fn to_datetime_utc(self) -> Option<DateTime<Utc>> {
        Utc.timestamp_millis_opt(self.value as i64).single()
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.to_datetime_utc() {
            Some(x) => write!(f, "{}", x.to_rfc3339_opts(SecondsFormat::Millis, true)),
            None => f.write_str(Timestamp::OUT_OF_RANGE),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DoubleBit {
    Intermediate,
    DeterminedOff,
    DeterminedOn,
    Indeterminate,
}

impl DoubleBit {
    // the lowest two bits of this number
    pub fn from(x: u8) -> Self {
        match x & 0b0000_0011 {
            0b00 => DoubleBit::Intermediate,
            0b01 => DoubleBit::DeterminedOff,
            0b10 => DoubleBit::DeterminedOn,
            _ => DoubleBit::Indeterminate,
        }
    }
}

impl std::fmt::Display for DoubleBit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ControlCode {
    pub tcc: TripCloseCode,
    pub clear: bool,
    pub queue: bool,
    pub op_type: OpType,
}

impl ControlCode {
    const TCC_MASK: u8 = 0b1100_0000;
    const CR_MASK: u8 = 0b0010_0000;
    const QU_MASK: u8 = 0b0001_0000;
    const OP_MASK: u8 = 0b0000_1111;

    pub fn from(x: u8) -> Self {
        Self {
            tcc: TripCloseCode::from((x & Self::TCC_MASK) >> 6),
            clear: x & Self::CR_MASK != 0,
            queue: x & Self::QU_MASK != 0,
            op_type: OpType::from(x & Self::OP_MASK),
        }
    }
    pub fn as_u8(self) -> u8 {
        let mut x = 0;
        x |= self.tcc.as_u8() << 6;
        if self.clear {
            x |= Self::CR_MASK;
        }
        if self.queue {
            x |= Self::QU_MASK;
        }
        x |= self.op_type.as_u8();
        x
    }
}

impl std::fmt::Display for Variation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (g, v) = self.to_group_and_var();
        write!(f, "g{}v{}", g, v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn conversion_from_timestamp_to_datetime_utc_cannot_overflow() {
        let timestamp = Timestamp::new(std::u64::MAX);
        timestamp.to_datetime_utc();
    }

    #[test]
    fn timestamp_display_formatting_works_as_expected() {
        assert_eq!(format!("{}", Timestamp::min()), "1970-01-01T00:00:00.000Z");
        assert_eq!(
            format!("{}", Timestamp::max()),
            "+10889-08-02T05:31:50.655Z"
        );
    }

    fn test_control_code_round_trip(byte: u8, cc: ControlCode) {
        assert_eq!(cc.as_u8(), byte);
        assert_eq!(ControlCode::from(byte), cc)
    }

    #[test]
    fn correctly_converts_control_code_to_and_from_u8() {
        test_control_code_round_trip(
            0b10_1_1_0100,
            ControlCode {
                tcc: TripCloseCode::Trip,
                clear: true,
                queue: true,
                op_type: OpType::LatchOff,
            },
        );

        test_control_code_round_trip(
            0b10_0_1_0100,
            ControlCode {
                tcc: TripCloseCode::Trip,
                clear: false,
                queue: true,
                op_type: OpType::LatchOff,
            },
        );

        test_control_code_round_trip(
            0b10_1_0_0100,
            ControlCode {
                tcc: TripCloseCode::Trip,
                clear: true,
                queue: false,
                op_type: OpType::LatchOff,
            },
        );

        test_control_code_round_trip(
            0b11_0_0_0000,
            ControlCode {
                tcc: TripCloseCode::Reserved,
                clear: false,
                queue: false,
                op_type: OpType::Nul,
            },
        );
    }
}
