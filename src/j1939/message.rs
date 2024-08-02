use crate::Conversion;
use crate::identifier::Id;
use crate::j1939::{J1939Id, NameField, DataField, Pdu, PduType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Message {
    id: Id,
    pdu: Pdu,
}

impl Message {
    /// Constructs a new Message from its parts: a 29-bit J1939 identifier and pdu containing 64 bits of generic data.
    ///
    /// # Arguments
    /// - `id`: An [`Id`] representing the 29-bit identifier of the message.
    /// - `pdu`: A [`Pdu`] containing the payload or content of the message.
    ///
    /// # Returns
    /// A new [`Message`] instance initialized with the provided parts.
    #[inline]
    pub fn from_parts(id: Id, pdu: Pdu) -> Option<Self> {
        match id {
            Id::Standard(_) => None,
            Id::Extended(v) => Some( Self { id: Id::J1939(J1939Id::from_bits(v)), pdu }),
            Id::J1939(_) => Some( Self { id, pdu }),
        }
    }

    /// Destructures the [`Message`] into its parts: a 29-bit J1939 identifier and pdu containing 64 bits of generic data.
    ///
    /// # Returns
    /// A tuple containing:
    /// - An [`Id`] representing the 29-bit identifier.
    /// - A [`Pdu`] containing the payload or content of the message.
    #[inline]
    #[must_use]
    pub fn into_parts(self) -> (Id, Pdu) {
        (self.id, self.pdu)
    }

    /// Constructs a new [`Message`] from raw bit representations of its components.
    #[inline]
    pub fn try_from_bits(hex_id: u32, hex_pdu: u64, pdu_type: PduType) -> Option<Self> {
        let id = Id::from_bits(hex_id, true);
        let pdu = match pdu_type {
            PduType::Name => match NameField::try_from_bits(hex_pdu) {
                Some(v) => Some(Pdu::NameField(v)),
                None => None,
            }
            PduType::Data => match DataField::try_from_bits(hex_pdu) {
                Some(v) => Some(Pdu::DataFiled(v)),
                None => None,
            },
        };

        match pdu {
            Some(pdu) => Some(Self { id, pdu }),
            None => None,
        }
    }

    /// Constructs a new [`Message`] from hexadecimal string representations of its components.
    #[inline]
    pub fn try_from_hex(hex_id: &str, hex_pdu: &str, pdu_type: PduType) -> Option<Self> {
        let id = Id::try_from_hex(hex_id, true);
        match id {
            Some(id) => {
                let pdu = match pdu_type {
                    PduType::Name => match NameField::try_from_hex(hex_pdu) {
                        Some(v) => Some(Pdu::NameField(v)),
                        None => None,
                    }
                    PduType::Data => match DataField::try_from_hex(hex_pdu) {
                        Some(v) => Some(Pdu::DataFiled(v)),
                        None => None,
                    },
                };

                match pdu {
                    Some(pdu) => Some(Self { id, pdu }),
                    None => None,
                }

            },
            None => None,
        }
    }

    /// Constructs a new [`Message`] from raw bit representations of its components.
    ///
    /// # Arguments
    /// - `hex_id`: A `u32` representing the hexadecimal encoded 29-bit J1939 identifier.
    /// - `hex_pdu`: A `u64` representing the hexadecimal encoded pdu.
    ///
    /// # Returns
    /// A new [`Message`] instance initialized with the decoded components.
    #[inline]
    #[must_use]
    pub fn from_bits(hex_id: u32, hex_pdu: u64, pdu_type: PduType) -> Self {
        let id = Id::from_bits(hex_id, true);
        let pdu = match pdu_type {
            PduType::Name => Pdu::NameField(NameField::from_bits(hex_pdu)),
            PduType::Data => Pdu::DataFiled(DataField::from_bits(hex_pdu)),
        };

        Self { id, pdu }
    }

    /// Constructs a new [`Message`] from hexadecimal string representations of its components.
    ///
    /// # Arguments
    /// - `hex_id`: A `&str` representing the hexadecimal encoded 29-bit J1939 identifier.
    /// - `hex_pdu`: A `&str` representing the hexadecimal encoded pdu.
    ///
    /// # Returns
    /// A new [`Message`] instance initialized with the decoded components.
    #[inline]
    #[must_use]
    pub fn from_hex(hex_id: &str, hex_pdu: &str, pdu_type: PduType) -> Self {
        let id = Id::from_hex(hex_id, true);
        let pdu = match pdu_type {
            PduType::Name => Pdu::NameField(NameField::from_hex(hex_pdu)),
            PduType::Data => Pdu::DataFiled(DataField::from_hex(hex_pdu)),
        };

        Self { id, pdu }
    }

    /// Retrieves the 29-bit J1939 identifier from the message.
    ///
    /// # Returns
    /// The [`Id`] bitfield associated with the message.
    #[inline]
    #[must_use]
    pub fn id(&self) -> Id {
        self.id
    }

    /// Retrieves the pdu from the message.
    ///
    /// # Returns
    /// The [`Pdu`] bitfield associated with the message.
    #[inline]
    #[must_use]
    pub fn pdu(&self) -> Pdu {
        self.pdu
    }
}



