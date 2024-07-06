use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Address {
    PrimaryEngineController,
    SecondaryEngineController,
    PrimaryTransmissionController,
    TransmissionShiftSelector,
    Brakes,
    Retarder,
    CruiseControl,
    FuelSystem,
    SteeringController,
    InstrumentCluster,
    ClimateControl1,
    Compass,
    BodyController,
    OffVehicleGateway,
    DidVid,
    RetarderExhaustEngine1,
    HeadwayController,
    Suspension,
    CabController,
    TirePressureController,
    LightingControlModule,
    ClimateControl2,
    ExhaustEmissionController,
    AuxiliaryHeater,
    ChassisController,
    CommunicationsUnit,
    Radio,
    SafetyRestraintSystem,
    AftertreatmentControlModule,
    MultiPurposeCamera,
    SwitchExpansionModule,
    AuxiliaryGaugeSwitchPack,
    Iteris,
    QualcommPeopleNetTranslatorBox,
    StandAloneRealTimeClock,
    CenterPanel1,
    CenterPanel2,
    CenterPanel3,
    CenterPanel4,
    CenterPanel5,
    WabcoOnGuardRadar,
    SecondaryInstrumentCluster,
    OffboardDiagnostics,
    Trailer3Bridge,
    Trailer2Bridge,
    Trailer1Bridge,
    SafetyDirectProcessor,
    ForwardRoadImageProcessor,
    LeftRearDoorPod,
    RightRearDoorPod,
    DoorController1,
    DoorController2,
    Tachograph,
    HybridSystem,
    AuxiliaryPowerUnit,
    ServiceTool,
    SourceAddressRequest0,
    SourceAddressRequest1,
    Unknown(u8),
}

impl From<u8> for Address {
    fn from(value: u8) -> Self {
        match value {
            0 => Address::PrimaryEngineController,
            1 => Address::SecondaryEngineController,
            3 => Address::PrimaryTransmissionController,
            5 => Address::TransmissionShiftSelector,
            11 => Address::Brakes,
            15 => Address::Retarder,
            17 => Address::CruiseControl,
            18 => Address::FuelSystem,
            19 => Address::SteeringController,
            23 => Address::InstrumentCluster,
            25 => Address::ClimateControl1,
            28 => Address::Compass,
            33 => Address::BodyController,
            37 => Address::OffVehicleGateway,
            40 => Address::DidVid,
            41 => Address::RetarderExhaustEngine1,
            42 => Address::HeadwayController,
            47 => Address::Suspension,
            49 => Address::CabController,
            51 => Address::TirePressureController,
            55 => Address::LightingControlModule,
            58 => Address::ClimateControl2,
            61 => Address::ExhaustEmissionController,
            69 => Address::AuxiliaryHeater,
            71 => Address::ChassisController,
            74 => Address::CommunicationsUnit,
            76 => Address::Radio,
            83 => Address::SafetyRestraintSystem,
            85 => Address::AftertreatmentControlModule,
            127 => Address::MultiPurposeCamera,
            128 => Address::SwitchExpansionModule,
            132 => Address::AuxiliaryGaugeSwitchPack,
            139 => Address::Iteris,
            142 => Address::QualcommPeopleNetTranslatorBox,
            150 => Address::StandAloneRealTimeClock,
            151 => Address::CenterPanel1,
            152 => Address::CenterPanel2,
            153 => Address::CenterPanel3,
            154 => Address::CenterPanel4,
            155 => Address::CenterPanel5,
            160 => Address::WabcoOnGuardRadar,
            167 => Address::SecondaryInstrumentCluster,
            172 => Address::OffboardDiagnostics,
            184 => Address::Trailer3Bridge,
            192 => Address::Trailer2Bridge,
            200 => Address::Trailer1Bridge,
            209 => Address::SafetyDirectProcessor,
            232 => Address::ForwardRoadImageProcessor,
            233 => Address::LeftRearDoorPod,
            234 => Address::RightRearDoorPod,
            236 => Address::DoorController1,
            237 => Address::DoorController2,
            238 => Address::Tachograph,
            239 => Address::HybridSystem,
            247 => Address::AuxiliaryPowerUnit,
            249 => Address::ServiceTool,
            254 => Address::SourceAddressRequest0,
            255 => Address::SourceAddressRequest1,
            a => Address::Unknown(a),
        }
    }
}

impl From<Address> for u8 {
    fn from(value: Address) -> Self {
        match value {
            Address::PrimaryEngineController => 0,
            Address::SecondaryEngineController => 1,
            Address::PrimaryTransmissionController => 3,
            Address::TransmissionShiftSelector => 5,
            Address::Brakes => 11,
            Address::Retarder => 15,
            Address::CruiseControl => 17,
            Address::FuelSystem => 18,
            Address::SteeringController => 19,
            Address::InstrumentCluster => 23,
            Address::ClimateControl1 => 25,
            Address::Compass => 28,
            Address::BodyController => 33,
            Address::OffVehicleGateway => 37,
            Address::DidVid => 40,
            Address::RetarderExhaustEngine1 => 41,
            Address::HeadwayController => 42,
            Address::Suspension => 47,
            Address::CabController => 49,
            Address::TirePressureController => 51,
            Address::LightingControlModule => 55,
            Address::ClimateControl2 => 58,
            Address::ExhaustEmissionController => 61,
            Address::AuxiliaryHeater => 69,
            Address::ChassisController => 71,
            Address::CommunicationsUnit => 74,
            Address::Radio => 76,
            Address::SafetyRestraintSystem => 83,
            Address::AftertreatmentControlModule => 85,
            Address::MultiPurposeCamera => 127,
            Address::SwitchExpansionModule => 128,
            Address::AuxiliaryGaugeSwitchPack => 132,
            Address::Iteris => 139,
            Address::QualcommPeopleNetTranslatorBox => 142,
            Address::StandAloneRealTimeClock => 150,
            Address::CenterPanel1 => 151,
            Address::CenterPanel2 => 152,
            Address::CenterPanel3 => 153,
            Address::CenterPanel4 => 154,
            Address::CenterPanel5 => 155,
            Address::WabcoOnGuardRadar => 160,
            Address::SecondaryInstrumentCluster => 167,
            Address::OffboardDiagnostics => 172,
            Address::Trailer3Bridge => 184,
            Address::Trailer2Bridge => 192,
            Address::Trailer1Bridge => 200,
            Address::SafetyDirectProcessor => 209,
            Address::ForwardRoadImageProcessor => 232,
            Address::LeftRearDoorPod => 233,
            Address::RightRearDoorPod => 234,
            Address::DoorController1 => 236,
            Address::DoorController2 => 237,
            Address::Tachograph => 238,
            Address::HybridSystem => 239,
            Address::AuxiliaryPowerUnit => 247,
            Address::ServiceTool => 249,
            Address::SourceAddressRequest0 => 254,
            Address::SourceAddressRequest1 => 255,
            Address::Unknown(a) => a,
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Address::PrimaryEngineController => write!(f, "Primary Engine Controller | (CPC, ECM)"),
            Address::SecondaryEngineController => write!(f, "Secondary Engine Controller | (MCM, ECM #2)"),
            Address::PrimaryTransmissionController => write!(f, "Primary Transmission Controller | (TCM)"),
            Address::TransmissionShiftSelector => write!(f, "Transmission Shift Selector | (TSS)"),
            Address::Brakes => write!(f, "Brakes | System Controller (ABS)"),
            Address::Retarder => write!(f, "Retarder"),
            Address::CruiseControl => write!(f, "Cruise Control | (IPM, PCC)"),
            Address::FuelSystem => write!(f, "Fuel System | Controller (CNG)"),
            Address::SteeringController => write!(f, "Steering Controller | (SAS)"),
            Address::InstrumentCluster => write!(f, "Instrument Gauge Cluster (EGC) | (ICU, RX)"),
            Address::ClimateControl1 => write!(f, "Climate Control #1 | (FCU)"),
            Address::Compass => write!(f, "Compass"),
            Address::BodyController => write!(f, "Body Controller | (SSAM, SAM-CAB, BHM)"),
            Address::OffVehicleGateway => write!(f, "Off-Vehicle Gateway | (CGW)"),
            Address::DidVid => write!(f, "Vehicle Information Display | Driver Information Display"),
            Address::RetarderExhaustEngine1 => write!(f, "Retarder, Exhaust, Engine #1"),
            Address::HeadwayController => write!(f, "Headway Controller | (RDF) | (OnGuard)"),
            Address::Suspension => write!(f, "Suspension | System Controller (ECAS)"),
            Address::CabController => write!(f, "Cab Controller | Primary (MSF, SHM, ECC)"),
            Address::TirePressureController => write!(f, "Tire Pressure Controller | (TPMS)"),
            Address::LightingControlModule => write!(f, "Lighting Control Module | (LCM)"),
            Address::ClimateControl2 => write!(f, "Climate Control #2 | Rear HVAC | (ParkSmart)"),
            Address::ExhaustEmissionController => write!(f, "Exhaust Emission Controller | (ACM) | (DCU)"),
            Address::AuxiliaryHeater => write!(f, "Auxiliary Heater | (ACU)"),
            Address::ChassisController => write!(f, "Chassis Controller | (CHM, SAM-Chassis)"),
            Address::CommunicationsUnit => write!(f, "Communications Unit | Cellular (CTP, VT)"),
            Address::Radio => write!(f, "Radio"),
            Address::SafetyRestraintSystem => write!(f, "Safety Restraint System | Air Bag | (SRS)"),
            Address::AftertreatmentControlModule => write!(f, "Aftertreatment Control Module | (ACM)"),
            Address::MultiPurposeCamera => write!(f, "Multi-Purpose Camera | (MPC)"),
            Address::SwitchExpansionModule => write!(f, "Switch Expansion Module | (SEM #1)"),
            Address::AuxiliaryGaugeSwitchPack => write!(f, "Auxiliary Gauge Switch Pack | (AGSP3)"),
            Address::Iteris => write!(f, "Iteris"),
            Address::QualcommPeopleNetTranslatorBox => write!(f, "Qualcomm - PeopleNet Translator Box"),
            Address::StandAloneRealTimeClock => write!(f, "Stand-Alone Real Time Clock | (SART)"),
            Address::CenterPanel1 => write!(f, "Center Panel MUX Switch Pack #1"),
            Address::CenterPanel2 => write!(f, "Center Panel MUX Switch Pack #2"),
            Address::CenterPanel3 => write!(f, "Center Panel MUX Switch Pack #3"),
            Address::CenterPanel4 => write!(f, "Center Panel MUX Switch Pack #4"),
            Address::CenterPanel5 => write!(f, "Center Panel MUX Switch Pack #5"),
            Address::WabcoOnGuardRadar => write!(f, "Wabco OnGuard Radar | OnGuard Display | Collision Mitigation System"),
            Address::SecondaryInstrumentCluster => write!(f, "Secondary Instrument Cluster | (SIC)"),
            Address::OffboardDiagnostics => write!(f, "Offboard Diagnostics"),
            Address::Trailer3Bridge => write!(f, "Trailer #3 Bridge"),
            Address::Trailer2Bridge => write!(f, "Trailer #2 Bridge"),
            Address::Trailer1Bridge => write!(f, "Trailer #1 Bridge"),
            Address::SafetyDirectProcessor => write!(f, "Bendix Camera | Safety Direct Processor (SDP) Module"),
            Address::ForwardRoadImageProcessor => write!(f, "Forward Road Image Processor | PAM Module | Lane Departure Warning (LDW) Module | (VRDU)"),
            Address::LeftRearDoorPod => write!(f, "Left Rear Door Pod"),
            Address::RightRearDoorPod => write!(f, "Right Rear Door Pod"),
            Address::DoorController1 => write!(f, "Door Controller #1"),
            Address::DoorController2 => write!(f, "Door Controller #2"),
            Address::Tachograph => write!(f, "Tachograph | (TCO)"),
            Address::HybridSystem => write!(f, "Hybrid System"),
            Address::AuxiliaryPowerUnit => write!(f, "Auxiliary Power Unit | (APU)"),
            Address::ServiceTool => write!(f, "Service Tool"),
            Address::SourceAddressRequest0 => write!(f, "Source Address Request 0"),
            Address::SourceAddressRequest1 => write!(f, "Source Address Request 1"),
            Address::Unknown(num) => write!(f, "Unknown({num})"),
        }
    }
}

/// Represents the source address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceAddress {
    /// No source address.
    None,
    /// Source address with a specific value.
    Some(u8),
}

/// Represents the destination address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestinationAddress {
    /// No destination address.
    None,
    /// Destination address with a specific value.
    Some(u8),
}

impl SourceAddress {
    /// Lookup and translate the [`SourceAddress`] object.
    ///
    /// # Returns
    /// - `Some(Address)`: If generic J1939 address is known.
    /// - `None`: If the pdu specific bits do not contain a destination address.
    #[must_use]
    pub fn lookup(self) -> Option<Address> {
        match self {
            SourceAddress::Some(value) => Some(value.into()),
            SourceAddress::None => None,
        }
    }
}

impl DestinationAddress {
    /// Lookup and translate the [`DestinationAddress`] object.
    ///
    /// # Returns
    /// - `Some(Address)`: If generic J1939 address is known.
    /// - `None`: If the pdu specific bits do not contain a destination address.
    #[must_use]
    pub fn lookup(self) -> Option<Address> {
        match self {
            DestinationAddress::Some(value) => Some(value.into()),
            DestinationAddress::None => None,
        }
    }
}
