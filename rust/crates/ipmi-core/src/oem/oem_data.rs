//! Generated from `lib/ipmi_strings.c` (`ipmi_oem_product_info`) and the
//! `IPMI_OEM_*` enterprise numbers in `include/ipmitool/ipmi.h`.
//!
//! Do not edit by hand; regenerate with the port's tooling.

/// A single `(manufacturer_id, product_id, product_name)` record.
pub struct OemProduct {
    pub oem: u32,
    pub product: u16,
    pub name: &'static str,
}

/// PICMG enterprise number: matches any manufacturer, as in C `oemval2str`.
pub const IPMI_OEM_PICMG: u32 = 12634;

/// Known product names keyed by manufacturer and product id.
pub static OEM_PRODUCT_INFO: &[OemProduct] = &[
    OemProduct {
        oem: 16777214,
        product: 4660,
        name: "Great Debuggable BMC",
    },
    OemProduct {
        oem: 343,
        product: 12,
        name: "TSRLT2",
    },
    OemProduct {
        oem: 343,
        product: 27,
        name: "TIGPR2U",
    },
    OemProduct {
        oem: 343,
        product: 34,
        name: "TIGI2U",
    },
    OemProduct {
        oem: 343,
        product: 38,
        name: "Bridgeport",
    },
    OemProduct {
        oem: 343,
        product: 40,
        name: "S5000PAL",
    },
    OemProduct {
        oem: 343,
        product: 41,
        name: "S5000PSL",
    },
    OemProduct {
        oem: 343,
        product: 256,
        name: "Tiger4",
    },
    OemProduct {
        oem: 343,
        product: 259,
        name: "McCarran",
    },
    OemProduct {
        oem: 343,
        product: 2048,
        name: "ZT5504",
    },
    OemProduct {
        oem: 343,
        product: 2056,
        name: "MPCBL0001",
    },
    OemProduct {
        oem: 343,
        product: 2065,
        name: "TIGW1U",
    },
    OemProduct {
        oem: 343,
        product: 17169,
        name: "NSI2U",
    },
    OemProduct {
        oem: 15000,
        product: 4000,
        name: "AM4000 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 4001,
        name: "AM4001 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 4002,
        name: "AM4002 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 4010,
        name: "AM4010 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 5503,
        name: "AM4500/4520 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 5504,
        name: "AM4300 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 5507,
        name: "AM4301 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 5508,
        name: "AM4330 AdvancedMC",
    },
    OemProduct {
        oem: 15000,
        product: 5520,
        name: "KTC5520/EATX",
    },
    OemProduct {
        oem: 15000,
        product: 5703,
        name: "RTM8020",
    },
    OemProduct {
        oem: 15000,
        product: 5704,
        name: "RTM8030",
    },
    OemProduct {
        oem: 15000,
        product: 5705,
        name: "RTM8050",
    },
    OemProduct {
        oem: 15000,
        product: 6000,
        name: "CP6000",
    },
    OemProduct {
        oem: 15000,
        product: 6006,
        name: "DT-64",
    },
    OemProduct {
        oem: 15000,
        product: 6010,
        name: "CP6010",
    },
    OemProduct {
        oem: 15000,
        product: 6011,
        name: "CP6011",
    },
    OemProduct {
        oem: 15000,
        product: 6012,
        name: "CP6012",
    },
    OemProduct {
        oem: 15000,
        product: 6014,
        name: "CP6014",
    },
    OemProduct {
        oem: 15000,
        product: 5002,
        name: "AT8001",
    },
    OemProduct {
        oem: 15000,
        product: 5003,
        name: "AT8010",
    },
    OemProduct {
        oem: 15000,
        product: 5004,
        name: "AT8020",
    },
    OemProduct {
        oem: 15000,
        product: 5006,
        name: "AT8030 IPMC",
    },
    OemProduct {
        oem: 15000,
        product: 2025,
        name: "AT8030 MMC",
    },
    OemProduct {
        oem: 15000,
        product: 5007,
        name: "AT8050",
    },
    OemProduct {
        oem: 15000,
        product: 5301,
        name: "AT8400",
    },
    OemProduct {
        oem: 15000,
        product: 5303,
        name: "AT8901",
    },
    OemProduct {
        oem: 4413,
        product: 5725,
        name: "BCM5725",
    },
    OemProduct {
        oem: 193,
        product: 84,
        name: "Phantom",
    },
    OemProduct {
        oem: 10297,
        product: 13203,
        name: "MIC-3393",
    },
    OemProduct {
        oem: 10297,
        product: 13205,
        name: "MIC-3395",
    },
    OemProduct {
        oem: 10297,
        product: 13206,
        name: "MIC-3396",
    },
    OemProduct {
        oem: 10297,
        product: 21250,
        name: "MIC-5302",
    },
    OemProduct {
        oem: 10297,
        product: 21252,
        name: "MIC-5304",
    },
    OemProduct {
        oem: 10297,
        product: 21280,
        name: "MIC-5320",
    },
    OemProduct {
        oem: 10297,
        product: 21281,
        name: "MIC-5321",
    },
    OemProduct {
        oem: 10297,
        product: 21282,
        name: "MIC-5322",
    },
    OemProduct {
        oem: 10297,
        product: 21298,
        name: "MIC-5332",
    },
    OemProduct {
        oem: 10297,
        product: 21299,
        name: "MIC-5333",
    },
    OemProduct {
        oem: 10297,
        product: 21314,
        name: "MIC-5342",
    },
    OemProduct {
        oem: 10297,
        product: 21315,
        name: "MIC-5343",
    },
    OemProduct {
        oem: 10297,
        product: 21316,
        name: "MIC-5344",
    },
    OemProduct {
        oem: 10297,
        product: 21317,
        name: "MIC-5345",
    },
    OemProduct {
        oem: 10297,
        product: 20993,
        name: "MIC-5201 Dual 10GE AMC",
    },
    OemProduct {
        oem: 10297,
        product: 20995,
        name: "MIC-5203 Quad GbE AMC",
    },
    OemProduct {
        oem: 10297,
        product: 21010,
        name: "MIC-5212 Dual 10GE AMC",
    },
    OemProduct {
        oem: 10297,
        product: 21505,
        name: "MIC-5401",
    },
    OemProduct {
        oem: 10297,
        product: 22017,
        name: "MIC-5601",
    },
    OemProduct {
        oem: 10297,
        product: 22018,
        name: "MIC-5602",
    },
    OemProduct {
        oem: 10297,
        product: 22020,
        name: "MIC-5604",
    },
    OemProduct {
        oem: 10297,
        product: 22019,
        name: "MIC-5603",
    },
    OemProduct {
        oem: 10297,
        product: 25361,
        name: "MIC-6311",
    },
    OemProduct {
        oem: 10297,
        product: 25363,
        name: "MIC-6313",
    },
    OemProduct {
        oem: 10297,
        product: 33537,
        name: "MIC-8301",
    },
    OemProduct {
        oem: 10297,
        product: 33538,
        name: "MIC-8302",
    },
    OemProduct {
        oem: 10297,
        product: 33540,
        name: "MIC-8304",
    },
    OemProduct {
        oem: 10297,
        product: 20737,
        name: "RTM-5101",
    },
    OemProduct {
        oem: 10297,
        product: 20738,
        name: "RTM-5102",
    },
    OemProduct {
        oem: 10297,
        product: 20742,
        name: "RTM-5106",
    },
    OemProduct {
        oem: 10297,
        product: 20743,
        name: "RTM-5107",
    },
    OemProduct {
        oem: 10297,
        product: 21008,
        name: "RTM-5210",
    },
    OemProduct {
        oem: 10297,
        product: 21024,
        name: "RTM-5220",
    },
    OemProduct {
        oem: 10297,
        product: 20740,
        name: "RTM-5104",
    },
    OemProduct {
        oem: 10297,
        product: 21760,
        name: "UTCA-5500",
    },
    OemProduct {
        oem: 10297,
        product: 21763,
        name: "UTCA-5503",
    },
    OemProduct {
        oem: 10297,
        product: 21764,
        name: "UTCA-5504",
    },
    OemProduct {
        oem: 10297,
        product: 22529,
        name: "UTCA-5801",
    },
    OemProduct {
        oem: 10297,
        product: 8720,
        name: "NCPB-2210",
    },
    OemProduct {
        oem: 10297,
        product: 8965,
        name: "NCPB-2305",
    },
    OemProduct {
        oem: 10297,
        product: 8992,
        name: "NCPB-2320",
    },
    OemProduct {
        oem: 10297,
        product: 12553,
        name: "NCP-3109",
    },
    OemProduct {
        oem: 10297,
        product: 12560,
        name: "NCP-3110",
    },
    OemProduct {
        oem: 10297,
        product: 12800,
        name: "NCP-3200",
    },
    OemProduct {
        oem: 10297,
        product: 20576,
        name: "SMM-5060",
    },
    OemProduct {
        oem: 10297,
        product: 12816,
        name: "FWA-3210",
    },
    OemProduct {
        oem: 10297,
        product: 12832,
        name: "FWA-3220",
    },
    OemProduct {
        oem: 10297,
        product: 12833,
        name: "FWA-3221",
    },
    OemProduct {
        oem: 10297,
        product: 12848,
        name: "FWA-3230",
    },
    OemProduct {
        oem: 10297,
        product: 12849,
        name: "FWA-3231",
    },
    OemProduct {
        oem: 10297,
        product: 12851,
        name: "FWA-3233",
    },
    OemProduct {
        oem: 10297,
        product: 12880,
        name: "FWA-3250",
    },
    OemProduct {
        oem: 10297,
        product: 12896,
        name: "FWA-3260",
    },
    OemProduct {
        oem: 10297,
        product: 20512,
        name: "FWA-5020",
    },
    OemProduct {
        oem: 10297,
        product: 25872,
        name: "FWA-6510",
    },
    OemProduct {
        oem: 10297,
        product: 25873,
        name: "FWA-6511",
    },
    OemProduct {
        oem: 10297,
        product: 25874,
        name: "FWA-6512",
    },
    OemProduct {
        oem: 10297,
        product: 25888,
        name: "FWA-6520",
    },
    OemProduct {
        oem: 10297,
        product: 25889,
        name: "FWA-6521",
    },
    OemProduct {
        oem: 10297,
        product: 25890,
        name: "FWA-6522",
    },
    OemProduct {
        oem: 10297,
        product: 29456,
        name: "ATCA-7310",
    },
    OemProduct {
        oem: 10297,
        product: 29488,
        name: "ATCA-7330",
    },
    OemProduct {
        oem: 10297,
        product: 29712,
        name: "ATCA-7410",
    },
    OemProduct {
        oem: 10297,
        product: 36899,
        name: "ATCA-9023",
    },
    OemProduct {
        oem: 10297,
        product: 37138,
        name: "ATCA-9112",
    },
    OemProduct {
        oem: 10297,
        product: 16897,
        name: "AMC-4201",
    },
    OemProduct {
        oem: 10297,
        product: 16898,
        name: "AMC-4202",
    },
    OemProduct {
        oem: 10297,
        product: 12817,
        name: "NAMB-3211",
    },
    OemProduct {
        oem: 10297,
        product: 4615,
        name: "CPCI-1207",
    },
    OemProduct {
        oem: 10297,
        product: 4622,
        name: "CPCI-1207 Test Board",
    },
    OemProduct {
        oem: 10297,
        product: 4868,
        name: "CPCI-1304",
    },
    OemProduct {
        oem: 10297,
        product: 28673,
        name: "CPCI-7001",
    },
    OemProduct {
        oem: 10297,
        product: 33312,
        name: "CPCI-8220",
    },
    OemProduct {
        oem: 10297,
        product: 36865,
        name: "ESP-9001",
    },
    OemProduct {
        oem: 10297,
        product: 36866,
        name: "ESP-9002",
    },
    OemProduct {
        oem: 10297,
        product: 36882,
        name: "ESP-9012",
    },
    OemProduct {
        oem: 10297,
        product: 37394,
        name: "ESP-9212",
    },
    OemProduct {
        oem: 10297,
        product: 24576,
        name: "CGS-6000",
    },
    OemProduct {
        oem: 10297,
        product: 24592,
        name: "CGS-6010",
    },
    OemProduct {
        oem: 24339,
        product: 12544,
        name: "aTCA-3100",
    },
    OemProduct {
        oem: 24339,
        product: 12560,
        name: "aTCA-3110",
    },
    OemProduct {
        oem: 24339,
        product: 12624,
        name: "aTCA-3150",
    },
    OemProduct {
        oem: 24339,
        product: 13344,
        name: "aTCA-3420",
    },
    OemProduct {
        oem: 24339,
        product: 14096,
        name: "aTCA-3710",
    },
    OemProduct {
        oem: 24339,
        product: 24832,
        name: "aTCA-6100",
    },
    OemProduct {
        oem: 24339,
        product: 25088,
        name: "aTCA-6200",
    },
    OemProduct {
        oem: 24339,
        product: 25168,
        name: "aTCA-6250/6250STW",
    },
    OemProduct {
        oem: 24339,
        product: 25200,
        name: "aTCA-R6270",
    },
    OemProduct {
        oem: 24339,
        product: 25216,
        name: "aTCA-R6280",
    },
    OemProduct {
        oem: 24339,
        product: 26768,
        name: "aTCA-6890",
    },
    OemProduct {
        oem: 24339,
        product: 26769,
        name: "aTCA-6891",
    },
    OemProduct {
        oem: 24339,
        product: 26880,
        name: "aTCA-6900",
    },
    OemProduct {
        oem: 24339,
        product: 26885,
        name: "aTCA-R6905",
    },
    OemProduct {
        oem: 24339,
        product: 26890,
        name: "aTCA-R6900",
    },
    OemProduct {
        oem: 24339,
        product: 33300,
        name: "aTCA-8214",
    },
    OemProduct {
        oem: 24339,
        product: 34310,
        name: "aTCA-8606",
    },
    OemProduct {
        oem: 24339,
        product: 37632,
        name: "aTCA-9300",
    },
    OemProduct {
        oem: 24339,
        product: 38656,
        name: "aTCA-9700",
    },
    OemProduct {
        oem: 24339,
        product: 38656,
        name: "aTCA-R9700",
    },
    OemProduct {
        oem: 24339,
        product: 38669,
        name: "aTCA-9700D",
    },
    OemProduct {
        oem: 24339,
        product: 38672,
        name: "aTCA-9710",
    },
    OemProduct {
        oem: 24339,
        product: 38672,
        name: "aTCA-R9710",
    },
    OemProduct {
        oem: 24339,
        product: 61441,
        name: "aTCA-FN001",
    },
    OemProduct {
        oem: 24339,
        product: 62112,
        name: "aTCA-F2AX",
    },
    OemProduct {
        oem: 24339,
        product: 62880,
        name: "aTCA-F5AX",
    },
    OemProduct {
        oem: 24339,
        product: 13584,
        name: "cPCI-3510",
    },
    OemProduct {
        oem: 24339,
        product: 14704,
        name: "cPCI-3970",
    },
    OemProduct {
        oem: 24339,
        product: 24592,
        name: "cPCI-6010",
    },
    OemProduct {
        oem: 24339,
        product: 25104,
        name: "cPCI-6210",
    },
    OemProduct {
        oem: 24339,
        product: 25872,
        name: "cPCI-6510",
    },
    OemProduct {
        oem: 24339,
        product: 25888,
        name: "cPCI-6520",
    },
    OemProduct {
        oem: 24339,
        product: 25893,
        name: "cPCI-6525",
    },
    OemProduct {
        oem: 24339,
        product: 25904,
        name: "cPCI-6530/6530BL",
    },
    OemProduct {
        oem: 24339,
        product: 26112,
        name: "cPCI-6600",
    },
    OemProduct {
        oem: 24339,
        product: 26688,
        name: "cPCI-6840",
    },
    OemProduct {
        oem: 24339,
        product: 26736,
        name: "cPCI-6870",
    },
    OemProduct {
        oem: 24339,
        product: 26752,
        name: "cPCI-6880",
    },
    OemProduct {
        oem: 24339,
        product: 26896,
        name: "cPCI-6910",
    },
    OemProduct {
        oem: 24339,
        product: 26912,
        name: "cPCI-6920",
    },
    OemProduct {
        oem: 24339,
        product: 26928,
        name: "cPCI-6930",
    },
    OemProduct {
        oem: 24339,
        product: 26944,
        name: "cPCI-6940",
    },
    OemProduct {
        oem: 24339,
        product: 12288,
        name: "VPX3000",
    },
    OemProduct {
        oem: 24339,
        product: 12289,
        name: "VPX3001",
    },
    OemProduct {
        oem: 24339,
        product: 12290,
        name: "VPX3002",
    },
    OemProduct {
        oem: 24339,
        product: 12304,
        name: "VPX3010",
    },
    OemProduct {
        oem: 24339,
        product: 16144,
        name: "VPX3G10",
    },
    OemProduct {
        oem: 24339,
        product: 24576,
        name: "VPX6000",
    },
    OemProduct {
        oem: 24339,
        product: 1040,
        name: "MXN-0410",
    },
    OemProduct {
        oem: 24339,
        product: 9728,
        name: "MCN-2600",
    },
    OemProduct {
        oem: 24339,
        product: 5376,
        name: "MCN-1500",
    },
    OemProduct {
        oem: 10876,
        product: 1,
        name: "X8DT3/i/-LN4",
    },
    OemProduct {
        oem: 10876,
        product: 2,
        name: "C2SLM-OC012",
    },
    OemProduct {
        oem: 10876,
        product: 3,
        name: "X8DTW-3/i/-LN4",
    },
    OemProduct {
        oem: 10876,
        product: 6,
        name: "X8DTL-3/i",
    },
    OemProduct {
        oem: 10876,
        product: 7,
        name: "X8DTT-IBX",
    },
    OemProduct {
        oem: 10876,
        product: 8,
        name: "X8DTT-B-SG",
    },
    OemProduct {
        oem: 10876,
        product: 9,
        name: "X8STi-3/-LN4",
    },
    OemProduct {
        oem: 10876,
        product: 10,
        name: "B8DT3",
    },
    OemProduct {
        oem: 10876,
        product: 256,
        name: "X8DAH+",
    },
    OemProduct {
        oem: 10876,
        product: 257,
        name: "B8DTL",
    },
    OemProduct {
        oem: 10876,
        product: 1024,
        name: "X8DTH-6/i",
    },
    OemProduct {
        oem: 10876,
        product: 1025,
        name: "X8DTH-RC014",
    },
    OemProduct {
        oem: 10876,
        product: 1027,
        name: "X8DA6/i-X",
    },
    OemProduct {
        oem: 10876,
        product: 1028,
        name: "X8DT6/E",
    },
    OemProduct {
        oem: 10876,
        product: 1033,
        name: "H8QI6/E(+)(-F)",
    },
    OemProduct {
        oem: 10876,
        product: 1035,
        name: "X8DTW-6+/i+",
    },
    OemProduct {
        oem: 10876,
        product: 1037,
        name: "X8SII/X8SI6",
    },
    OemProduct {
        oem: 10876,
        product: 1040,
        name: "X8DTL-6/L",
    },
    OemProduct {
        oem: 10876,
        product: 1536,
        name: "X8DTU",
    },
    OemProduct {
        oem: 10876,
        product: 1537,
        name: "X8DAL-3/i",
    },
    OemProduct {
        oem: 10876,
        product: 1538,
        name: "X7SLA",
    },
    OemProduct {
        oem: 10876,
        product: 1539,
        name: "C2SEM-Q",
    },
    OemProduct {
        oem: 10876,
        product: 1540,
        name: "winbond IPMI (Aten)",
    },
    OemProduct {
        oem: 10876,
        product: 1541,
        name: "X8SIL",
    },
    OemProduct {
        oem: 10876,
        product: 1542,
        name: "X8DTG-4",
    },
    OemProduct {
        oem: 10876,
        product: 1543,
        name: "C2G41",
    },
    OemProduct {
        oem: 10876,
        product: 1544,
        name: "X8DTG-DF",
    },
    OemProduct {
        oem: 10876,
        product: 1545,
        name: "X8DTS",
    },
    OemProduct {
        oem: 10876,
        product: 1546,
        name: "X7SPA",
    },
    OemProduct {
        oem: 10876,
        product: 1547,
        name: "X8SIA",
    },
    OemProduct {
        oem: 10876,
        product: 1548,
        name: "X8DTU+",
    },
    OemProduct {
        oem: 10876,
        product: 1548,
        name: "X8DTU-LN4F+",
    },
    OemProduct {
        oem: 10876,
        product: 1549,
        name: "X8DTU-6TF+",
    },
    OemProduct {
        oem: 10876,
        product: 1550,
        name: "X8SIT",
    },
    OemProduct {
        oem: 10876,
        product: 1551,
        name: "X8DTN+-F",
    },
    OemProduct {
        oem: 10876,
        product: 1552,
        name: "X8QB6/E",
    },
    OemProduct {
        oem: 10876,
        product: 1553,
        name: "AOC-STGN-I2S",
    },
    OemProduct {
        oem: 10876,
        product: 1554,
        name: "B8DTT",
    },
    OemProduct {
        oem: 10876,
        product: 1555,
        name: "X8DTT-C-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 1556,
        name: "X8DTU-F-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1562,
        name: "X8SIU",
    },
    OemProduct {
        oem: 10876,
        product: 1563,
        name: "X8DTT-HA",
    },
    OemProduct {
        oem: 10876,
        product: 1564,
        name: "B8DT6-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1565,
        name: "C7SIA",
    },
    OemProduct {
        oem: 10876,
        product: 1566,
        name: "B8DTT-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1566,
        name: "X9SCL",
    },
    OemProduct {
        oem: 10876,
        product: 1567,
        name: "B8DTP",
    },
    OemProduct {
        oem: 10876,
        product: 1568,
        name: "B8DTG",
    },
    OemProduct {
        oem: 10876,
        product: 1569,
        name: "X8OBN",
    },
    OemProduct {
        oem: 10876,
        product: 1570,
        name: "C7P67",
    },
    OemProduct {
        oem: 10876,
        product: 1571,
        name: "C7Q67",
    },
    OemProduct {
        oem: 10876,
        product: 1572,
        name: "X9SCM",
    },
    OemProduct {
        oem: 10876,
        product: 1573,
        name: "B9DRi",
    },
    OemProduct {
        oem: 10876,
        product: 1574,
        name: "X9DR3-LN4F+",
    },
    OemProduct {
        oem: 10876,
        product: 1575,
        name: "X7SPT-DF-D525",
    },
    OemProduct {
        oem: 10876,
        product: 1576,
        name: "X9DR3/i-F",
    },
    OemProduct {
        oem: 10876,
        product: 1577,
        name: "X9SRT-F",
    },
    OemProduct {
        oem: 10876,
        product: 1578,
        name: "X9SRA",
    },
    OemProduct {
        oem: 10876,
        product: 1579,
        name: "X9SRi",
    },
    OemProduct {
        oem: 10876,
        product: 1580,
        name: "X9DAi",
    },
    OemProduct {
        oem: 10876,
        product: 1581,
        name: "X8SIL-SIOM-FI005",
    },
    OemProduct {
        oem: 10876,
        product: 1582,
        name: "X8STE-DS018",
    },
    OemProduct {
        oem: 10876,
        product: 1583,
        name: "X9DRT-HF",
    },
    OemProduct {
        oem: 10876,
        product: 1584,
        name: "X9DRW-3TF+",
    },
    OemProduct {
        oem: 10876,
        product: 1585,
        name: "X9SCA/X9SCI",
    },
    OemProduct {
        oem: 10876,
        product: 1586,
        name: "X8DTG-QF+",
    },
    OemProduct {
        oem: 10876,
        product: 1587,
        name: "X9SCV-Q",
    },
    OemProduct {
        oem: 10876,
        product: 1588,
        name: "X9SCD",
    },
    OemProduct {
        oem: 10876,
        product: 1589,
        name: "X9DAL-6/i",
    },
    OemProduct {
        oem: 10876,
        product: 1590,
        name: "X9DRW-3F",
    },
    OemProduct {
        oem: 10876,
        product: 1591,
        name: "X9DBU-3F",
    },
    OemProduct {
        oem: 10876,
        product: 1592,
        name: "X8QB6-LF",
    },
    OemProduct {
        oem: 10876,
        product: 1600,
        name: "X9SCL/SCM-II",
    },
    OemProduct {
        oem: 10876,
        product: 1601,
        name: "X9SCV-LN4F+",
    },
    OemProduct {
        oem: 10876,
        product: 1602,
        name: "C7Q67-H",
    },
    OemProduct {
        oem: 10876,
        product: 1604,
        name: "X9SAE",
    },
    OemProduct {
        oem: 10876,
        product: 1605,
        name: "C7H61",
    },
    OemProduct {
        oem: 10876,
        product: 1606,
        name: "X9SCFF",
    },
    OemProduct {
        oem: 10876,
        product: 1607,
        name: "X9SCAA",
    },
    OemProduct {
        oem: 10876,
        product: 1607,
        name: "X9SCE-F",
    },
    OemProduct {
        oem: 10876,
        product: 1608,
        name: "C7B75",
    },
    OemProduct {
        oem: 10876,
        product: 1616,
        name: "X9SPV-M4",
    },
    OemProduct {
        oem: 10876,
        product: 1617,
        name: "X9SBAA",
    },
    OemProduct {
        oem: 10876,
        product: 1618,
        name: "AOC-CGP-i2",
    },
    OemProduct {
        oem: 10876,
        product: 1619,
        name: "C7Z87",
    },
    OemProduct {
        oem: 10876,
        product: 1620,
        name: "C7Q87",
    },
    OemProduct {
        oem: 10876,
        product: 1621,
        name: "AOC-XEH-IN2",
    },
    OemProduct {
        oem: 10876,
        product: 1622,
        name: "AOC-SGP-i2",
    },
    OemProduct {
        oem: 10876,
        product: 1623,
        name: "C7Z87-OCE",
    },
    OemProduct {
        oem: 10876,
        product: 1624,
        name: "AOC-STG-B4S",
    },
    OemProduct {
        oem: 10876,
        product: 1625,
        name: "AOC-STGN-I1S",
    },
    OemProduct {
        oem: 10876,
        product: 1632,
        name: "X9DRD-3/iF",
    },
    OemProduct {
        oem: 10876,
        product: 1633,
        name: "B1SA4-F",
    },
    OemProduct {
        oem: 10876,
        product: 1633,
        name: "X9SRG",
    },
    OemProduct {
        oem: 10876,
        product: 1634,
        name: "X9DRG-QF+",
    },
    OemProduct {
        oem: 10876,
        product: 1635,
        name: "X9SRW",
    },
    OemProduct {
        oem: 10876,
        product: 1636,
        name: "X9DRH-7F",
    },
    OemProduct {
        oem: 10876,
        product: 1637,
        name: "X9DRG-QF",
    },
    OemProduct {
        oem: 10876,
        product: 1638,
        name: "X9DRT-D-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 1639,
        name: "X9DRL-3/iF",
    },
    OemProduct {
        oem: 10876,
        product: 1640,
        name: "X9SRD",
    },
    OemProduct {
        oem: 10876,
        product: 1641,
        name: "X9DRX+-F",
    },
    OemProduct {
        oem: 10876,
        product: 1642,
        name: "B9DRT",
    },
    OemProduct {
        oem: 10876,
        product: 1643,
        name: "X9SRL-F",
    },
    OemProduct {
        oem: 10876,
        product: 1644,
        name: "B9DRG",
    },
    OemProduct {
        oem: 10876,
        product: 1645,
        name: "X9DR6+-F-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1646,
        name: "X9DR6-LN4F+-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1647,
        name: "X9DRW-LN4F+-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1680,
        name: "2208",
    },
    OemProduct {
        oem: 10876,
        product: 1681,
        name: "2308",
    },
    OemProduct {
        oem: 10876,
        product: 1792,
        name: "X9DRT-HF-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1793,
        name: "X9DRT-HIBQF-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1794,
        name: "X9DRD-7LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 1795,
        name: "X9DBL-6F",
    },
    OemProduct {
        oem: 10876,
        product: 1796,
        name: "X9DR7/E-TF+",
    },
    OemProduct {
        oem: 10876,
        product: 1797,
        name: "X9DR7/E-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 1798,
        name: "X9DRT-HF+",
    },
    OemProduct {
        oem: 10876,
        product: 1799,
        name: "X9QR7-TF+",
    },
    OemProduct {
        oem: 10876,
        product: 1800,
        name: "X9DRT-F",
    },
    OemProduct {
        oem: 10876,
        product: 1801,
        name: "X9DA7/E",
    },
    OemProduct {
        oem: 10876,
        product: 1802,
        name: "X9DRF-F",
    },
    OemProduct {
        oem: 10876,
        product: 1803,
        name: "X9SRL-CF",
    },
    OemProduct {
        oem: 10876,
        product: 1804,
        name: "X9SRD-CD",
    },
    OemProduct {
        oem: 10876,
        product: 1805,
        name: "B9DR7",
    },
    OemProduct {
        oem: 10876,
        product: 1806,
        name: "X9DAX-7",
    },
    OemProduct {
        oem: 10876,
        product: 1807,
        name: "X9QR7-TF",
    },
    OemProduct {
        oem: 10876,
        product: 1808,
        name: "X9SRE",
    },
    OemProduct {
        oem: 10876,
        product: 1809,
        name: "X9DRG-HTF+",
    },
    OemProduct {
        oem: 10876,
        product: 1810,
        name: "B9DRG-X",
    },
    OemProduct {
        oem: 10876,
        product: 1811,
        name: "X9DRFF-7",
    },
    OemProduct {
        oem: 10876,
        product: 1812,
        name: "X9SRH-7F",
    },
    OemProduct {
        oem: 10876,
        product: 1813,
        name: "X9DRW-7TPF",
    },
    OemProduct {
        oem: 10876,
        product: 1814,
        name: "X9DBU-3F-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 1815,
        name: "X9DRFF-7+/i+",
    },
    OemProduct {
        oem: 10876,
        product: 1816,
        name: "X9DRFF-7G+",
    },
    OemProduct {
        oem: 10876,
        product: 1817,
        name: "B9DRP",
    },
    OemProduct {
        oem: 10876,
        product: 1824,
        name: "X9DRW-7TPF+",
    },
    OemProduct {
        oem: 10876,
        product: 1825,
        name: "X9DBS-F",
    },
    OemProduct {
        oem: 10876,
        product: 1826,
        name: "X9DB3-TPF",
    },
    OemProduct {
        oem: 10876,
        product: 1827,
        name: "X9DRL-7F/EF",
    },
    OemProduct {
        oem: 10876,
        product: 1828,
        name: "X9DRH-if-NV",
    },
    OemProduct {
        oem: 10876,
        product: 1829,
        name: "X9DRG+HF+II",
    },
    OemProduct {
        oem: 10876,
        product: 1830,
        name: "X10QBi",
    },
    OemProduct {
        oem: 10876,
        product: 1831,
        name: "X9DRW-CF31/CTF31",
    },
    OemProduct {
        oem: 10876,
        product: 1832,
        name: "X10DRI/T",
    },
    OemProduct {
        oem: 10876,
        product: 1833,
        name: "B9DRG-E",
    },
    OemProduct {
        oem: 10876,
        product: 1840,
        name: "X9DAX-HFT",
    },
    OemProduct {
        oem: 10876,
        product: 1841,
        name: "X9DRT-P",
    },
    OemProduct {
        oem: 10876,
        product: 1842,
        name: "X9DRG-OF",
    },
    OemProduct {
        oem: 10876,
        product: 1843,
        name: "X9DB3-CPU-AI026",
    },
    OemProduct {
        oem: 10876,
        product: 1844,
        name: "AOC-STG-I2T",
    },
    OemProduct {
        oem: 10876,
        product: 1859,
        name: "X9SPU",
    },
    OemProduct {
        oem: 10876,
        product: 2049,
        name: "X10SLL",
    },
    OemProduct {
        oem: 10876,
        product: 2050,
        name: "X10SLA",
    },
    OemProduct {
        oem: 10876,
        product: 2051,
        name: "X10SLH",
    },
    OemProduct {
        oem: 10876,
        product: 2052,
        name: "X10SL7-F",
    },
    OemProduct {
        oem: 10876,
        product: 2053,
        name: "X10SAE",
    },
    OemProduct {
        oem: 10876,
        product: 2054,
        name: "X10SLM-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 2055,
        name: "X10SLE",
    },
    OemProduct {
        oem: 10876,
        product: 2056,
        name: "LSI3008",
    },
    OemProduct {
        oem: 10876,
        product: 2057,
        name: "LSI3108",
    },
    OemProduct {
        oem: 10876,
        product: 2064,
        name: "X10SLD",
    },
    OemProduct {
        oem: 10876,
        product: 2065,
        name: "X10SLM-F",
    },
    OemProduct {
        oem: 10876,
        product: 2066,
        name: "X10SLL+-F",
    },
    OemProduct {
        oem: 10876,
        product: 2067,
        name: "X10SEAA-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 2068,
        name: "X10SLX-TF",
    },
    OemProduct {
        oem: 10876,
        product: 2069,
        name: "X10SLV",
    },
    OemProduct {
        oem: 10876,
        product: 2070,
        name: "X10SBA",
    },
    OemProduct {
        oem: 10876,
        product: 2071,
        name: "X10SLE-DF",
    },
    OemProduct {
        oem: 10876,
        product: 2072,
        name: "X10SAT",
    },
    OemProduct {
        oem: 10876,
        product: 2073,
        name: "X10SLV-Q",
    },
    OemProduct {
        oem: 10876,
        product: 2080,
        name: "A1SAM",
    },
    OemProduct {
        oem: 10876,
        product: 2080,
        name: "X9SKV",
    },
    OemProduct {
        oem: 10876,
        product: 2081,
        name: "X10DRW-i(T)",
    },
    OemProduct {
        oem: 10876,
        product: 2082,
        name: "X10DRFF(-C)",
    },
    OemProduct {
        oem: 10876,
        product: 2083,
        name: "AOC-CTG-I1S",
    },
    OemProduct {
        oem: 10876,
        product: 2084,
        name: "X10DRT-P",
    },
    OemProduct {
        oem: 10876,
        product: 2085,
        name: "X10SRE-DS018",
    },
    OemProduct {
        oem: 10876,
        product: 2086,
        name: "X10SRE-EXP-SI030",
    },
    OemProduct {
        oem: 10876,
        product: 2087,
        name: "X9QRH-TF",
    },
    OemProduct {
        oem: 10876,
        product: 2088,
        name: "A1SA2-2750F",
    },
    OemProduct {
        oem: 10876,
        product: 2089,
        name: "C7Z97-OCE",
    },
    OemProduct {
        oem: 10876,
        product: 2096,
        name: "X10DRFR",
    },
    OemProduct {
        oem: 10876,
        product: 2097,
        name: "X10DRL-i",
    },
    OemProduct {
        oem: 10876,
        product: 2098,
        name: "X10SRL",
    },
    OemProduct {
        oem: 10876,
        product: 2099,
        name: "X10SRW-F",
    },
    OemProduct {
        oem: 10876,
        product: 2100,
        name: "X10SRI",
    },
    OemProduct {
        oem: 10876,
        product: 2101,
        name: "X10DRC/i-LN4+/T4+",
    },
    OemProduct {
        oem: 10876,
        product: 2102,
        name: "X10DAi",
    },
    OemProduct {
        oem: 10876,
        product: 2103,
        name: "X10DRT-D-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2104,
        name: "X10SRH-F",
    },
    OemProduct {
        oem: 10876,
        product: 2105,
        name: "AOC-SLG3-2E4",
    },
    OemProduct {
        oem: 10876,
        product: 2106,
        name: "X10DBT-T",
    },
    OemProduct {
        oem: 10876,
        product: 2107,
        name: "X10DDW-iT",
    },
    OemProduct {
        oem: 10876,
        product: 2108,
        name: "A1SA7-2750F",
    },
    OemProduct {
        oem: 10876,
        product: 2109,
        name: "LSI 2116",
    },
    OemProduct {
        oem: 10876,
        product: 2110,
        name: "B10DRi",
    },
    OemProduct {
        oem: 10876,
        product: 2111,
        name: "B9QR7",
    },
    OemProduct {
        oem: 10876,
        product: 2112,
        name: "B9DRG-3M",
    },
    OemProduct {
        oem: 10876,
        product: 2113,
        name: "X9DRD-iT+",
    },
    OemProduct {
        oem: 10876,
        product: 2114,
        name: "X10SLH-N6-ST031",
    },
    OemProduct {
        oem: 10876,
        product: 2115,
        name: "X10DRG-H(T)",
    },
    OemProduct {
        oem: 10876,
        product: 2116,
        name: "X10DRU-i+",
    },
    OemProduct {
        oem: 10876,
        product: 2117,
        name: "X10DDW-iN",
    },
    OemProduct {
        oem: 10876,
        product: 2118,
        name: "B1DRi",
    },
    OemProduct {
        oem: 10876,
        product: 2119,
        name: "AOC-UR-i4XT",
    },
    OemProduct {
        oem: 10876,
        product: 2120,
        name: "AOC-2UR68-i4G",
    },
    OemProduct {
        oem: 10876,
        product: 2121,
        name: "AOC-UR-i4G",
    },
    OemProduct {
        oem: 10876,
        product: 2122,
        name: "AOC-S40G-i2Q",
    },
    OemProduct {
        oem: 10876,
        product: 2123,
        name: "X10SRG",
    },
    OemProduct {
        oem: 10876,
        product: 2124,
        name: "C7Z97",
    },
    OemProduct {
        oem: 10876,
        product: 2124,
        name: "C7Z97-MF",
    },
    OemProduct {
        oem: 10876,
        product: 2125,
        name: "AOC-STGN-i1SF",
    },
    OemProduct {
        oem: 10876,
        product: 2127,
        name: "AOC-STGN-i2SF",
    },
    OemProduct {
        oem: 10876,
        product: 2128,
        name: "AOC-PG-i4M",
    },
    OemProduct {
        oem: 10876,
        product: 2129,
        name: "PMC PM8076",
    },
    OemProduct {
        oem: 10876,
        product: 2130,
        name: "X10DRG-Q",
    },
    OemProduct {
        oem: 10876,
        product: 2131,
        name: "X10QBL",
    },
    OemProduct {
        oem: 10876,
        product: 2132,
        name: "X10DRU-X",
    },
    OemProduct {
        oem: 10876,
        product: 2133,
        name: "H9SKV-420",
    },
    OemProduct {
        oem: 10876,
        product: 2134,
        name: "C7X99",
    },
    OemProduct {
        oem: 10876,
        product: 2135,
        name: "X10SRA",
    },
    OemProduct {
        oem: 10876,
        product: 2136,
        name: "X10DRT-H",
    },
    OemProduct {
        oem: 10876,
        product: 2137,
        name: "X10DRH-C",
    },
    OemProduct {
        oem: 10876,
        product: 2138,
        name: "A1SRM-LN7F",
    },
    OemProduct {
        oem: 10876,
        product: 2138,
        name: "A1SRM-LN7F",
    },
    OemProduct {
        oem: 10876,
        product: 2139,
        name: "X10DRT-HA/HB-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2140,
        name: "X10DRU-XLL",
    },
    OemProduct {
        oem: 10876,
        product: 2141,
        name: "AOC-UR-i2XT",
    },
    OemProduct {
        oem: 10876,
        product: 2142,
        name: "AOC-2UR6-i4XT",
    },
    OemProduct {
        oem: 10876,
        product: 2143,
        name: "AOC-2UR68-i2XT",
    },
    OemProduct {
        oem: 10876,
        product: 2144,
        name: "X10DRL-C/CT",
    },
    OemProduct {
        oem: 10876,
        product: 2145,
        name: "B10DRT",
    },
    OemProduct {
        oem: 10876,
        product: 2146,
        name: "B1SL1-F",
    },
    OemProduct {
        oem: 10876,
        product: 2147,
        name: "RSC-W2-66",
    },
    OemProduct {
        oem: 10876,
        product: 2148,
        name: "X10DRW-E/NT",
    },
    OemProduct {
        oem: 10876,
        product: 2149,
        name: "X10DRD-i/LTP",
    },
    OemProduct {
        oem: 10876,
        product: 2150,
        name: "AOC-SLG3-2E4R",
    },
    OemProduct {
        oem: 10876,
        product: 2151,
        name: "AOC-SLG3-4E4R",
    },
    OemProduct {
        oem: 10876,
        product: 2152,
        name: "AOC-SLG3-2E4",
    },
    OemProduct {
        oem: 10876,
        product: 2153,
        name: "AOC-URN2-i4XT",
    },
    OemProduct {
        oem: 10876,
        product: 2154,
        name: "AOC-URN2-i2XT",
    },
    OemProduct {
        oem: 10876,
        product: 2155,
        name: "X10DRS",
    },
    OemProduct {
        oem: 10876,
        product: 2156,
        name: "X10DAL-i",
    },
    OemProduct {
        oem: 10876,
        product: 2157,
        name: "X10SDV-TLN4F",
    },
    OemProduct {
        oem: 10876,
        product: 2158,
        name: "X10DDW-iN",
    },
    OemProduct {
        oem: 10876,
        product: 2159,
        name: "X10DRT-L",
    },
    OemProduct {
        oem: 10876,
        product: 2160,
        name: "AOC-URN2-ii2SX",
    },
    OemProduct {
        oem: 10876,
        product: 2161,
        name: "AOC-2UR68-i2XS",
    },
    OemProduct {
        oem: 10876,
        product: 2162,
        name: "X10QRH+",
    },
    OemProduct {
        oem: 10876,
        product: 2163,
        name: "AOC-2URN4-i4XT",
    },
    OemProduct {
        oem: 10876,
        product: 2164,
        name: "AOC-2UR8N4-i2XT",
    },
    OemProduct {
        oem: 10876,
        product: 2165,
        name: "AOC-2UR66-i4G",
    },
    OemProduct {
        oem: 10876,
        product: 2166,
        name: "A2SCL-IN001",
    },
    OemProduct {
        oem: 10876,
        product: 2167,
        name: "X10DRT-HB-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2168,
        name: "RSC-S2-88",
    },
    OemProduct {
        oem: 10876,
        product: 2169,
        name: "X10DRD-I/L/NT",
    },
    OemProduct {
        oem: 10876,
        product: 2170,
        name: "B1SD2-TF",
    },
    OemProduct {
        oem: 10876,
        product: 2171,
        name: "AOC-XEH-i2M",
    },
    OemProduct {
        oem: 10876,
        product: 2172,
        name: "AOC-XEH-i4M",
    },
    OemProduct {
        oem: 10876,
        product: 2173,
        name: "X10SRD-F",
    },
    OemProduct {
        oem: 10876,
        product: 2174,
        name: "AOC-STG-i4S",
    },
    OemProduct {
        oem: 10876,
        product: 2175,
        name: "X10DRFF-iG",
    },
    OemProduct {
        oem: 10876,
        product: 2176,
        name: "X10DRT-XA-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2177,
        name: "X10DRT-XB-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2178,
        name: "X10DRX",
    },
    OemProduct {
        oem: 10876,
        product: 2179,
        name: "K1SPI",
    },
    OemProduct {
        oem: 10876,
        product: 2180,
        name: "X11SSH-F/-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 2181,
        name: "X11SSL-CF/-NF",
    },
    OemProduct {
        oem: 10876,
        product: 2182,
        name: "X11SSA(-F)/X11SSi-LN4(F)",
    },
    OemProduct {
        oem: 10876,
        product: 2183,
        name: "AOC-S8076L-L16e",
    },
    OemProduct {
        oem: 10876,
        product: 2184,
        name: "X11SSZ",
    },
    OemProduct {
        oem: 10876,
        product: 2185,
        name: "B10DRG",
    },
    OemProduct {
        oem: 10876,
        product: 2186,
        name: "AOC-URN6-i2XT",
    },
    OemProduct {
        oem: 10876,
        product: 2187,
        name: "C7Z170-SQ",
    },
    OemProduct {
        oem: 10876,
        product: 2188,
        name: "AOM-S3108-H8S-SB",
    },
    OemProduct {
        oem: 10876,
        product: 2189,
        name: "AOC-S100G-i2C",
    },
    OemProduct {
        oem: 10876,
        product: 2190,
        name: "AOC-PIO-i2G",
    },
    OemProduct {
        oem: 10876,
        product: 2191,
        name: "AOM-PIO-i2XT",
    },
    OemProduct {
        oem: 10876,
        product: 2192,
        name: "X11SSQ",
    },
    OemProduct {
        oem: 10876,
        product: 2193,
        name: "X11SBA",
    },
    OemProduct {
        oem: 10876,
        product: 2194,
        name: "X10DRG-O+-CPU",
    },
    OemProduct {
        oem: 10876,
        product: 2195,
        name: "X10DGQ",
    },
    OemProduct {
        oem: 10876,
        product: 2196,
        name: "X11SSV-Q",
    },
    OemProduct {
        oem: 10876,
        product: 2197,
        name: "X11SAE",
    },
    OemProduct {
        oem: 10876,
        product: 2198,
        name: "X11SSL/M-F",
    },
    OemProduct {
        oem: 10876,
        product: 2199,
        name: "B1SD1-TF",
    },
    OemProduct {
        oem: 10876,
        product: 2200,
        name: "X11SSE",
    },
    OemProduct {
        oem: 10876,
        product: 2201,
        name: "X10DRH",
    },
    OemProduct {
        oem: 10876,
        product: 2202,
        name: "X11SLL",
    },
    OemProduct {
        oem: 10876,
        product: 2203,
        name: "X11SSW-F",
    },
    OemProduct {
        oem: 10876,
        product: 2204,
        name: "X11SSH-CTF",
    },
    OemProduct {
        oem: 10876,
        product: 2205,
        name: "X10DSC+",
    },
    OemProduct {
        oem: 10876,
        product: 2206,
        name: "AOC-URN2-i4GXS",
    },
    OemProduct {
        oem: 10876,
        product: 2207,
        name: "X11SAE-M",
    },
    OemProduct {
        oem: 10876,
        product: 2305,
        name: "X11SSM",
    },
    OemProduct {
        oem: 10876,
        product: 2306,
        name: "C7Z170-OCE",
    },
    OemProduct {
        oem: 10876,
        product: 2307,
        name: "AOC-CTGS-i2T",
    },
    OemProduct {
        oem: 10876,
        product: 2308,
        name: "AOC-2UR6N4-i4XT",
    },
    OemProduct {
        oem: 10876,
        product: 2309,
        name: "K1SPI-KA-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2310,
        name: "K1SPi-KB-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2311,
        name: "X11SAT(-F)",
    },
    OemProduct {
        oem: 10876,
        product: 2312,
        name: "B2SS2",
    },
    OemProduct {
        oem: 10876,
        product: 2313,
        name: "X11SSD-F",
    },
    OemProduct {
        oem: 10876,
        product: 2314,
        name: "AOC-SLG3-4E4",
    },
    OemProduct {
        oem: 10876,
        product: 2315,
        name: "AOC-MTG-i4S",
    },
    OemProduct {
        oem: 10876,
        product: 2316,
        name: "AOC-MTG-I2s",
    },
    OemProduct {
        oem: 10876,
        product: 2317,
        name: "X11SRE-DS018",
    },
    OemProduct {
        oem: 10876,
        product: 2318,
        name: "K1SPE(-T)",
    },
    OemProduct {
        oem: 10876,
        product: 2319,
        name: "A2SDi",
    },
    OemProduct {
        oem: 10876,
        product: 2320,
        name: "A2SDi-H",
    },
    OemProduct {
        oem: 10876,
        product: 2321,
        name: "X10DRT-PS",
    },
    OemProduct {
        oem: 10876,
        product: 2322,
        name: "AOC-MG-i2",
    },
    OemProduct {
        oem: 10876,
        product: 2323,
        name: "AOC-MG-i4",
    },
    OemProduct {
        oem: 10876,
        product: 2324,
        name: "AOC-STGS-i1T",
    },
    OemProduct {
        oem: 10876,
        product: 2325,
        name: "AOC-STGS-i2T",
    },
    OemProduct {
        oem: 10876,
        product: 2326,
        name: "X10DSC-TP4S",
    },
    OemProduct {
        oem: 10876,
        product: 2327,
        name: "X11DPi-N(T)",
    },
    OemProduct {
        oem: 10876,
        product: 2328,
        name: "AOC-UR-i4GXS",
    },
    OemProduct {
        oem: 10876,
        product: 2329,
        name: "X10SDD-F",
    },
    OemProduct {
        oem: 10876,
        product: 2330,
        name: "AOC-2UR6-i4xT+",
    },
    OemProduct {
        oem: 10876,
        product: 2331,
        name: "AOC-UR-i4XT+",
    },
    OemProduct {
        oem: 10876,
        product: 2332,
        name: "X11DPU",
    },
    OemProduct {
        oem: 10876,
        product: 2333,
        name: "X11SSW-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 2334,
        name: "X10SD7-AC107",
    },
    OemProduct {
        oem: 10876,
        product: 2335,
        name: "AOC-MTG-i4T",
    },
    OemProduct {
        oem: 10876,
        product: 2336,
        name: "AOC-MTG-i2T",
    },
    OemProduct {
        oem: 10876,
        product: 2337,
        name: "X10SDV-7TP8F",
    },
    OemProduct {
        oem: 10876,
        product: 2338,
        name: "X10SRW-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 2339,
        name: "AOM-X10OBI-PCH",
    },
    OemProduct {
        oem: 10876,
        product: 2340,
        name: "X10SRM-(T)F",
    },
    OemProduct {
        oem: 10876,
        product: 2341,
        name: "AOC-MHIBF-2QG",
    },
    OemProduct {
        oem: 10876,
        product: 2342,
        name: "AOC-MHIBF-1QG",
    },
    OemProduct {
        oem: 10876,
        product: 2343,
        name: "X11DPT-PS",
    },
    OemProduct {
        oem: 10876,
        product: 2344,
        name: "AOC-S25G-b2S",
    },
    OemProduct {
        oem: 10876,
        product: 2345,
        name: "AOC-2UR66-i4XTF",
    },
    OemProduct {
        oem: 10876,
        product: 2346,
        name: "AOC-M25G-b2S",
    },
    OemProduct {
        oem: 10876,
        product: 2347,
        name: "X10DSN-TS",
    },
    OemProduct {
        oem: 10876,
        product: 2348,
        name: "AOC-S8069L-16xR",
    },
    OemProduct {
        oem: 10876,
        product: 2349,
        name: "X10SDE-DF",
    },
    OemProduct {
        oem: 10876,
        product: 2350,
        name: "AOC-2UR68-B2TS",
    },
    OemProduct {
        oem: 10876,
        product: 2351,
        name: "AOC-S40G-i1Q",
    },
    OemProduct {
        oem: 10876,
        product: 2352,
        name: "X11SSQ-L",
    },
    OemProduct {
        oem: 10876,
        product: 2353,
        name: "B2SS1-CF",
    },
    OemProduct {
        oem: 10876,
        product: 2354,
        name: "X10DRL-LN4",
    },
    OemProduct {
        oem: 10876,
        product: 2355,
        name: "P8DTU",
    },
    OemProduct {
        oem: 10876,
        product: 2356,
        name: "AOC-MHFI-i1C",
    },
    OemProduct {
        oem: 10876,
        product: 2357,
        name: "AOC-STG-i4T",
    },
    OemProduct {
        oem: 10876,
        product: 2358,
        name: "AOC-UR-i4XTF",
    },
    OemProduct {
        oem: 10876,
        product: 2359,
        name: "X11DPT",
    },
    OemProduct {
        oem: 10876,
        product: 2360,
        name: "X11DPT",
    },
    OemProduct {
        oem: 10876,
        product: 2361,
        name: "X11SSV-M4",
    },
    OemProduct {
        oem: 10876,
        product: 2362,
        name: "X11SSV-M4F",
    },
    OemProduct {
        oem: 10876,
        product: 2363,
        name: "AOC-STGF-i2S",
    },
    OemProduct {
        oem: 10876,
        product: 2364,
        name: "A2SAV",
    },
    OemProduct {
        oem: 10876,
        product: 2365,
        name: "AOC-S3216L-L16iT",
    },
    OemProduct {
        oem: 10876,
        product: 2366,
        name: "X10DRT-B+",
    },
    OemProduct {
        oem: 10876,
        product: 2367,
        name: "C7Z270-CG-L",
    },
    OemProduct {
        oem: 10876,
        product: 2368,
        name: "C7Z270-CG",
    },
    OemProduct {
        oem: 10876,
        product: 2369,
        name: "X11SPi-TF",
    },
    OemProduct {
        oem: 10876,
        product: 2370,
        name: "K1SPE",
    },
    OemProduct {
        oem: 10876,
        product: 2371,
        name: "C7Z270-PG",
    },
    OemProduct {
        oem: 10876,
        product: 2372,
        name: "C7H270-CG-ML",
    },
    OemProduct {
        oem: 10876,
        product: 2373,
        name: "C7Q270-CB-ML",
    },
    OemProduct {
        oem: 10876,
        product: 2374,
        name: "X10SDV",
    },
    OemProduct {
        oem: 10876,
        product: 2375,
        name: "A2SAN",
    },
    OemProduct {
        oem: 10876,
        product: 2376,
        name: "AOC-C25G-m2S",
    },
    OemProduct {
        oem: 10876,
        product: 2377,
        name: "AOC-MH25G-m2S2T",
    },
    OemProduct {
        oem: 10876,
        product: 2378,
        name: "AOC-2UR68-m2TS",
    },
    OemProduct {
        oem: 10876,
        product: 2379,
        name: "AOC-S100G-m2C",
    },
    OemProduct {
        oem: 10876,
        product: 2380,
        name: "AOC-S25G-m2S",
    },
    OemProduct {
        oem: 10876,
        product: 2381,
        name: "AOC-MHIBF",
    },
    OemProduct {
        oem: 10876,
        product: 2382,
        name: "AOC-MH25G-b2S2G",
    },
    OemProduct {
        oem: 10876,
        product: 2383,
        name: "P8DNU",
    },
    OemProduct {
        oem: 10876,
        product: 2384,
        name: "X10DRW-SE",
    },
    OemProduct {
        oem: 10876,
        product: 2385,
        name: "B2SS1(2)-MTF",
    },
    OemProduct {
        oem: 10876,
        product: 2386,
        name: "X11SPX-TF",
    },
    OemProduct {
        oem: 10876,
        product: 2387,
        name: "X11SPW-(c)TF",
    },
    OemProduct {
        oem: 10876,
        product: 2388,
        name: "AOC-C25G-m1S",
    },
    OemProduct {
        oem: 10876,
        product: 2389,
        name: "X11SSW",
    },
    OemProduct {
        oem: 10876,
        product: 2390,
        name: "AOC-MHIBF-m1Q2G",
    },
    OemProduct {
        oem: 10876,
        product: 2391,
        name: "AOC-MHIBF-m2Q2G",
    },
    OemProduct {
        oem: 10876,
        product: 2392,
        name: "X11SSH",
    },
    OemProduct {
        oem: 10876,
        product: 2393,
        name: "R10SR2",
    },
    OemProduct {
        oem: 10876,
        product: 2394,
        name: "X10DGO",
    },
    OemProduct {
        oem: 10876,
        product: 2395,
        name: "X11DPU-XLL",
    },
    OemProduct {
        oem: 10876,
        product: 2396,
        name: "X11DPU-X",
    },
    OemProduct {
        oem: 10876,
        product: 2397,
        name: "X11SPM-T(P)F",
    },
    OemProduct {
        oem: 10876,
        product: 2398,
        name: "X11SPH-NCTPF",
    },
    OemProduct {
        oem: 10876,
        product: 2399,
        name: "X11SAA",
    },
    OemProduct {
        oem: 10876,
        product: 2400,
        name: "X11SAN",
    },
    OemProduct {
        oem: 10876,
        product: 2401,
        name: "P9DSC",
    },
    OemProduct {
        oem: 10876,
        product: 2402,
        name: "X11DPT-B+",
    },
    OemProduct {
        oem: 10876,
        product: 2403,
        name: "H11DSU-i",
    },
    OemProduct {
        oem: 10876,
        product: 2404,
        name: "H11DSi-NT",
    },
    OemProduct {
        oem: 10876,
        product: 2405,
        name: "C7B250-CB-ML",
    },
    OemProduct {
        oem: 10876,
        product: 2406,
        name: "B11DPT",
    },
    OemProduct {
        oem: 10876,
        product: 2407,
        name: "X11DDW",
    },
    OemProduct {
        oem: 10876,
        product: 2408,
        name: "X11DPL-i",
    },
    OemProduct {
        oem: 10876,
        product: 2409,
        name: "A2SDi-HLN4F",
    },
    OemProduct {
        oem: 10876,
        product: 2410,
        name: "C7C232-CB-ML",
    },
    OemProduct {
        oem: 10876,
        product: 2411,
        name: "X11DPT-P-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 2412,
        name: "X11DPU-Z+",
    },
    OemProduct {
        oem: 10876,
        product: 2413,
        name: "X11DPG-Qt",
    },
    OemProduct {
        oem: 10876,
        product: 2414,
        name: "X11SPL-F",
    },
    OemProduct {
        oem: 10876,
        product: 2415,
        name: "AOC-IBH-X4ED",
    },
    OemProduct {
        oem: 10876,
        product: 2416,
        name: "C7Z270-CG-M",
    },
    OemProduct {
        oem: 10876,
        product: 2417,
        name: "X11SRE-EXP-SI030",
    },
    OemProduct {
        oem: 10876,
        product: 2418,
        name: "SAS3216",
    },
    OemProduct {
        oem: 10876,
        product: 2419,
        name: "SAS3508",
    },
    OemProduct {
        oem: 10876,
        product: 2420,
        name: "SAS3516",
    },
    OemProduct {
        oem: 10876,
        product: 2421,
        name: "SAS3316",
    },
    OemProduct {
        oem: 10876,
        product: 2422,
        name: "SAS3324",
    },
    OemProduct {
        oem: 10876,
        product: 2423,
        name: "X11DPFR-SN",
    },
    OemProduct {
        oem: 10876,
        product: 2424,
        name: "AOC-S25G-i2S",
    },
    OemProduct {
        oem: 10876,
        product: 2425,
        name: "BMB-IBD-E3616",
    },
    OemProduct {
        oem: 10876,
        product: 2426,
        name: "X11DAI-N",
    },
    OemProduct {
        oem: 10876,
        product: 2427,
        name: "C7X299",
    },
    OemProduct {
        oem: 10876,
        product: 2428,
        name: "AOC-IBH_X4ES",
    },
    OemProduct {
        oem: 10876,
        product: 2429,
        name: "C7B250-CB-i",
    },
    OemProduct {
        oem: 10876,
        product: 2430,
        name: "X11SPG-F",
    },
    OemProduct {
        oem: 10876,
        product: 2431,
        name: "X11QPH+",
    },
    OemProduct {
        oem: 10876,
        product: 2432,
        name: "X11OPi",
    },
    OemProduct {
        oem: 10876,
        product: 2433,
        name: "X11DPH-i",
    },
    OemProduct {
        oem: 10876,
        product: 2434,
        name: "A2SDV-TLN5F",
    },
    OemProduct {
        oem: 10876,
        product: 2435,
        name: "AOC-M25G-m4S",
    },
    OemProduct {
        oem: 10876,
        product: 2436,
        name: "X11SRM-(V)F",
    },
    OemProduct {
        oem: 10876,
        product: 2437,
        name: "P9DSU-C",
    },
    OemProduct {
        oem: 10876,
        product: 2438,
        name: "X11SDV",
    },
    OemProduct {
        oem: 10876,
        product: 2439,
        name: "X11SDV-TLN2F",
    },
    OemProduct {
        oem: 10876,
        product: 2440,
        name: "X11DGQ",
    },
    OemProduct {
        oem: 10876,
        product: 2441,
        name: "B10DRE",
    },
    OemProduct {
        oem: 10876,
        product: 2442,
        name: "H11DST-B",
    },
    OemProduct {
        oem: 10876,
        product: 2443,
        name: "X11SSN",
    },
    OemProduct {
        oem: 10876,
        product: 2444,
        name: "X11DPX-T",
    },
    OemProduct {
        oem: 10876,
        product: 2445,
        name: "A2SAP-H/E/L",
    },
    OemProduct {
        oem: 10876,
        product: 2446,
        name: "X11SRA-R/F",
    },
    OemProduct {
        oem: 10876,
        product: 2447,
        name: "B11QPi",
    },
    OemProduct {
        oem: 10876,
        product: 2449,
        name: "C7Z370-CG-L",
    },
    OemProduct {
        oem: 10876,
        product: 2450,
        name: "B11SPi-CPU-TF",
    },
    OemProduct {
        oem: 10876,
        product: 2451,
        name: "AOC-URN4-i2TS",
    },
    OemProduct {
        oem: 10876,
        product: 2452,
        name: "AOC-B25G-M2S",
    },
    OemProduct {
        oem: 10876,
        product: 2453,
        name: "X11DPG-SN",
    },
    OemProduct {
        oem: 10876,
        product: 2454,
        name: "AOC-MTG-b2T",
    },
    OemProduct {
        oem: 10876,
        product: 2455,
        name: "B11DPE",
    },
    OemProduct {
        oem: 10876,
        product: 2456,
        name: "C7Z370-CG-iW",
    },
    OemProduct {
        oem: 10876,
        product: 2457,
        name: "AOC-MHIBE-m1CG",
    },
    OemProduct {
        oem: 10876,
        product: 2458,
        name: "X11DPS-R",
    },
    OemProduct {
        oem: 10876,
        product: 2459,
        name: "AOC-OPH-WFR",
    },
    OemProduct {
        oem: 10876,
        product: 2460,
        name: "SAS 3616",
    },
    OemProduct {
        oem: 10876,
        product: 2460,
        name: "SAS 3616",
    },
    OemProduct {
        oem: 10876,
        product: 2461,
        name: "X11DSC+",
    },
    OemProduct {
        oem: 10876,
        product: 2462,
        name: "C7Z390-PGW",
    },
    OemProduct {
        oem: 10876,
        product: 2463,
        name: "C7Z390-CG-IW",
    },
    OemProduct {
        oem: 10876,
        product: 4369,
        name: "H8DCR-3",
    },
    OemProduct {
        oem: 10876,
        product: 4625,
        name: "H8DC8",
    },
    OemProduct {
        oem: 10876,
        product: 4881,
        name: "H8QM8-2",
    },
    OemProduct {
        oem: 10876,
        product: 5137,
        name: "H8DA8-2",
    },
    OemProduct {
        oem: 10876,
        product: 5393,
        name: "H8DMR-82",
    },
    OemProduct {
        oem: 10876,
        product: 5649,
        name: "H8DM8-2",
    },
    OemProduct {
        oem: 10876,
        product: 5905,
        name: "H8SMI-2",
    },
    OemProduct {
        oem: 10876,
        product: 6161,
        name: "H8SSL-I2",
    },
    OemProduct {
        oem: 10876,
        product: 6417,
        name: "H8DMU",
    },
    OemProduct {
        oem: 10876,
        product: 6657,
        name: "C7B360-CB-M",
    },
    OemProduct {
        oem: 10876,
        product: 6658,
        name: "C7C242-CB-M",
    },
    OemProduct {
        oem: 10876,
        product: 6659,
        name: "H11SSL-i/c/nc",
    },
    OemProduct {
        oem: 10876,
        product: 6660,
        name: "B10KNM",
    },
    OemProduct {
        oem: 10876,
        product: 6661,
        name: "AOM-M25G-i2S",
    },
    OemProduct {
        oem: 10876,
        product: 6662,
        name: "X11DPG-OT-CPU",
    },
    OemProduct {
        oem: 10876,
        product: 6663,
        name: "B11SPE-CPU-TF/25G",
    },
    OemProduct {
        oem: 10876,
        product: 6664,
        name: "AOC-STG-b2T",
    },
    OemProduct {
        oem: 10876,
        product: 6665,
        name: "AOC-URN4-b2XT",
    },
    OemProduct {
        oem: 10876,
        product: 6682,
        name: "A2SD1-3750F/3955F",
    },
    OemProduct {
        oem: 10876,
        product: 6683,
        name: "X11SCE-F",
    },
    OemProduct {
        oem: 10876,
        product: 6684,
        name: "X11DSF",
    },
    OemProduct {
        oem: 10876,
        product: 6685,
        name: "X11SCA-F/W",
    },
    OemProduct {
        oem: 10876,
        product: 6686,
        name: "X11DAC",
    },
    OemProduct {
        oem: 10876,
        product: 6687,
        name: "X11SCL-F",
    },
    OemProduct {
        oem: 10876,
        product: 6913,
        name: "X11DSN-TS",
    },
    OemProduct {
        oem: 10876,
        product: 6914,
        name: "X11DGO-T",
    },
    OemProduct {
        oem: 10876,
        product: 6915,
        name: "AOC-2UR688-i4XTF",
    },
    OemProduct {
        oem: 10876,
        product: 6916,
        name: "X11DPT-LC019",
    },
    OemProduct {
        oem: 10876,
        product: 6917,
        name: "X11SCQ",
    },
    OemProduct {
        oem: 10876,
        product: 6918,
        name: "X11SCQ-L",
    },
    OemProduct {
        oem: 10876,
        product: 6919,
        name: "X11SCW-F",
    },
    OemProduct {
        oem: 10876,
        product: 6920,
        name: "C9X299-PG300",
    },
    OemProduct {
        oem: 10876,
        product: 6921,
        name: "X11SCZ-F",
    },
    OemProduct {
        oem: 10876,
        product: 6922,
        name: "B10DRT-IBF2",
    },
    OemProduct {
        oem: 10876,
        product: 6923,
        name: "AOC-STG-b2T",
    },
    OemProduct {
        oem: 10876,
        product: 6924,
        name: "AOC-B25G-6X4D",
    },
    OemProduct {
        oem: 10876,
        product: 6925,
        name: "X11SCV-Q/L",
    },
    OemProduct {
        oem: 10876,
        product: 6926,
        name: "X11SCL-IF",
    },
    OemProduct {
        oem: 10876,
        product: 6927,
        name: "X11SCM",
    },
    OemProduct {
        oem: 10876,
        product: 6928,
        name: "A2SDV-LN8F/LN10PF",
    },
    OemProduct {
        oem: 10876,
        product: 6929,
        name: "X11SCH-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 6930,
        name: "AOC-SGP-i4",
    },
    OemProduct {
        oem: 10876,
        product: 6931,
        name: "AOC-4UR66-i4XTF",
    },
    OemProduct {
        oem: 10876,
        product: 6932,
        name: "C9X299-PG300F",
    },
    OemProduct {
        oem: 10876,
        product: 6933,
        name: "X11SCL-LN4F",
    },
    OemProduct {
        oem: 10876,
        product: 6934,
        name: "AOM-PICO-3L",
    },
    OemProduct {
        oem: 10876,
        product: 6935,
        name: "X11DPFF-SN",
    },
    OemProduct {
        oem: 10876,
        product: 6936,
        name: "C7B360-CB-ML",
    },
    OemProduct {
        oem: 10876,
        product: 6937,
        name: "H11SSW-NT/iN",
    },
    OemProduct {
        oem: 10876,
        product: 6938,
        name: "C9Z390-CG(W)",
    },
    OemProduct {
        oem: 10876,
        product: 6939,
        name: "AOC-CTG-i2S",
    },
    OemProduct {
        oem: 10876,
        product: 6940,
        name: "X11SDD",
    },
    OemProduct {
        oem: 10876,
        product: 6941,
        name: "X11DPT-L",
    },
    OemProduct {
        oem: 10876,
        product: 6942,
        name: "AOC-URN4-B2ts",
    },
    OemProduct {
        oem: 10876,
        product: 6943,
        name: "X11DPG-HGX2",
    },
    OemProduct {
        oem: 10876,
        product: 6944,
        name: "AOC-SLG2-2TM2",
    },
    OemProduct {
        oem: 10876,
        product: 6945,
        name: "AOC-SLG2-4TM2",
    },
    OemProduct {
        oem: 10876,
        product: 6946,
        name: "AOC-SMG2-2TM2",
    },
    OemProduct {
        oem: 10876,
        product: 6947,
        name: "RSC-UM2R-8+-NI22",
    },
    OemProduct {
        oem: 10876,
        product: 6948,
        name: "BPN-6S3008S6",
    },
    OemProduct {
        oem: 10876,
        product: 6949,
        name: "X11DPU-ZE+",
    },
    OemProduct {
        oem: 10876,
        product: 6950,
        name: "AOC-2UR68-b2TS",
    },
    OemProduct {
        oem: 10876,
        product: 6951,
        name: "AOC-S100G-b1C",
    },
    OemProduct {
        oem: 10876,
        product: 6952,
        name: "X11SPA-TF",
    },
    OemProduct {
        oem: 10876,
        product: 6953,
        name: "B2SC1",
    },
    OemProduct {
        oem: 10876,
        product: 6954,
        name: "AOC-2MG-i8",
    },
    OemProduct {
        oem: 10876,
        product: 6955,
        name: "H12SSW-iN",
    },
    OemProduct {
        oem: 10876,
        product: 6956,
        name: "H12SSW-NT",
    },
    OemProduct {
        oem: 10876,
        product: 6957,
        name: "M11SDV",
    },
    OemProduct {
        oem: 10876,
        product: 6958,
        name: "AOC-MG-b2",
    },
    OemProduct {
        oem: 10876,
        product: 6959,
        name: "AOM-SMF-TP4F",
    },
    OemProduct {
        oem: 10876,
        product: 6961,
        name: "AOC-M25G-m2S",
    },
    OemProduct {
        oem: 10876,
        product: 6962,
        name: "B11SRE-CPU",
    },
    OemProduct {
        oem: 10876,
        product: 6963,
        name: "B11SRE",
    },
    OemProduct {
        oem: 10876,
        product: 6964,
        name: "X11SPA-T",
    },
    OemProduct {
        oem: 10876,
        product: 6965,
        name: "X11SDW",
    },
    OemProduct {
        oem: 10876,
        product: 6966,
        name: "X11SCD-F",
    },
    OemProduct {
        oem: 10876,
        product: 6967,
        name: "X11DPU-V",
    },
    OemProduct {
        oem: 10876,
        product: 6968,
        name: "X11DPT-BV",
    },
    OemProduct {
        oem: 10876,
        product: 6969,
        name: "B11SCG-ZTF",
    },
    OemProduct {
        oem: 10876,
        product: 6970,
        name: "H12DST-B",
    },
    OemProduct {
        oem: 10876,
        product: 6971,
        name: "X11SDS-8C",
    },
    OemProduct {
        oem: 10876,
        product: 6972,
        name: "AOM-SDW-B16X2",
    },
    OemProduct {
        oem: 10876,
        product: 6973,
        name: "B11SCG-CTF",
    },
    OemProduct {
        oem: 10876,
        product: 6974,
        name: "AOC-S100G-b2C",
    },
    OemProduct {
        oem: 10876,
        product: 6975,
        name: "X11SPD-F",
    },
    OemProduct {
        oem: 10876,
        product: 6977,
        name: "X11SRL-F",
    },
    OemProduct {
        oem: 10876,
        product: 8209,
        name: "H8SMU",
    },
    OemProduct {
        oem: 10876,
        product: 8465,
        name: "M8QME",
    },
    OemProduct {
        oem: 10876,
        product: 8721,
        name: "H8DMT",
    },
    OemProduct {
        oem: 10876,
        product: 8977,
        name: "BHDME",
    },
    OemProduct {
        oem: 10876,
        product: 9233,
        name: "H8DMT-IBX",
    },
    OemProduct {
        oem: 10876,
        product: 9489,
        name: "H8DME-2-LS006",
    },
    OemProduct {
        oem: 10876,
        product: 32896,
        name: "X7DB8",
    },
    OemProduct {
        oem: 10876,
        product: 33152,
        name: "X7DA8",
    },
    OemProduct {
        oem: 10876,
        product: 33408,
        name: "PDSMI-LN4",
    },
    OemProduct {
        oem: 10876,
        product: 33664,
        name: "X7DBP-X",
    },
    OemProduct {
        oem: 10876,
        product: 33920,
        name: "X7DBR-8+",
    },
    OemProduct {
        oem: 10876,
        product: 34176,
        name: "PDSML",
    },
    OemProduct {
        oem: 10876,
        product: 34432,
        name: "X7DVL",
    },
    OemProduct {
        oem: 10876,
        product: 34688,
        name: "PDSBA",
    },
    OemProduct {
        oem: 10876,
        product: 34944,
        name: "X7DBX",
    },
    OemProduct {
        oem: 10876,
        product: 35200,
        name: "X7DBP-8",
    },
    OemProduct {
        oem: 10876,
        product: 36992,
        name: "X7DB8-X",
    },
    OemProduct {
        oem: 10876,
        product: 37248,
        name: "PDSME+",
    },
    OemProduct {
        oem: 10876,
        product: 37504,
        name: "X7DBR-3",
    },
    OemProduct {
        oem: 10876,
        product: 37760,
        name: "PDSLM",
    },
    OemProduct {
        oem: 10876,
        product: 38016,
        name: "X7DAL-E",
    },
    OemProduct {
        oem: 10876,
        product: 38272,
        name: "X6QT8",
    },
    OemProduct {
        oem: 10876,
        product: 38528,
        name: "X7DBN",
    },
    OemProduct {
        oem: 10876,
        product: 38784,
        name: "X7DBU",
    },
    OemProduct {
        oem: 10876,
        product: 39040,
        name: "PDSMU+",
    },
    OemProduct {
        oem: 10876,
        product: 39296,
        name: "X7DBT",
    },
    OemProduct {
        oem: 10876,
        product: 40977,
        name: "H8DM3-2",
    },
    OemProduct {
        oem: 10876,
        product: 41088,
        name: "X7DVL-3",
    },
    OemProduct {
        oem: 10876,
        product: 41233,
        name: "H8DA3-2",
    },
    OemProduct {
        oem: 10876,
        product: 41489,
        name: "H8QM3-2",
    },
    OemProduct {
        oem: 10876,
        product: 41600,
        name: "X7DW3+",
    },
    OemProduct {
        oem: 10876,
        product: 41745,
        name: "H8QM3-EX",
    },
    OemProduct {
        oem: 10876,
        product: 41856,
        name: "X7QC3",
    },
    OemProduct {
        oem: 10876,
        product: 42112,
        name: "X7DCL-3",
    },
    OemProduct {
        oem: 10876,
        product: 42257,
        name: "H8DI3+",
    },
    OemProduct {
        oem: 10876,
        product: 42368,
        name: "AOC-USAS-L8I",
    },
    OemProduct {
        oem: 10876,
        product: 42513,
        name: "BHQIE",
    },
    OemProduct {
        oem: 10876,
        product: 42624,
        name: "X7DCA-3/i",
    },
    OemProduct {
        oem: 10876,
        product: 42769,
        name: "H8SGL",
    },
    OemProduct {
        oem: 10876,
        product: 42880,
        name: "B7DC3",
    },
    OemProduct {
        oem: 10876,
        product: 43025,
        name: "H8DGU",
    },
    OemProduct {
        oem: 10876,
        product: 43025,
        name: "H8DGU",
    },
    OemProduct {
        oem: 10876,
        product: 43136,
        name: "X8DA3/i",
    },
    OemProduct {
        oem: 10876,
        product: 43281,
        name: "H8DGG-QF",
    },
    OemProduct {
        oem: 10876,
        product: 43392,
        name: "X7SB3",
    },
    OemProduct {
        oem: 10876,
        product: 43537,
        name: "H8DGT",
    },
    OemProduct {
        oem: 10876,
        product: 43648,
        name: "X7DWT-C-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 43648,
        name: "X7SBL-LN1/LN2",
    },
    OemProduct {
        oem: 10876,
        product: 43793,
        name: "H8QG6",
    },
    OemProduct {
        oem: 10876,
        product: 43904,
        name: "C2SBM-Q",
    },
    OemProduct {
        oem: 10876,
        product: 44049,
        name: "H8DGU-LN4",
    },
    OemProduct {
        oem: 10876,
        product: 44160,
        name: "X7DWU",
    },
    OemProduct {
        oem: 10876,
        product: 44305,
        name: "BHQME",
    },
    OemProduct {
        oem: 10876,
        product: 44305,
        name: "H8DGT-HIBQF-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 44561,
        name: "H8QGI-F-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 44672,
        name: "PDSMP-JN001",
    },
    OemProduct {
        oem: 10876,
        product: 44817,
        name: "H8DGU-LN4F+-AI034",
    },
    OemProduct {
        oem: 10876,
        product: 44928,
        name: "AOC-UTG-I2",
    },
    OemProduct {
        oem: 10876,
        product: 45184,
        name: "B7DBE",
    },
    OemProduct {
        oem: 10876,
        product: 45440,
        name: "X7DBI+",
    },
    OemProduct {
        oem: 10876,
        product: 45696,
        name: "X7DVL-L",
    },
    OemProduct {
        oem: 10876,
        product: 45952,
        name: "X7DWN",
    },
    OemProduct {
        oem: 10876,
        product: 46208,
        name: "AOC-UG-i4",
    },
    OemProduct {
        oem: 10876,
        product: 46464,
        name: "PDSBM-LN2",
    },
    OemProduct {
        oem: 10876,
        product: 46720,
        name: "X7DGT-ATOKAP",
    },
    OemProduct {
        oem: 10876,
        product: 46976,
        name: "C2SBA",
    },
    OemProduct {
        oem: 10876,
        product: 46976,
        name: "X7DWT-CD-SG007",
    },
    OemProduct {
        oem: 10876,
        product: 47232,
        name: "C2SEA",
    },
    OemProduct {
        oem: 10876,
        product: 47488,
        name: "Winbond IPMI (AMI)",
    },
    OemProduct {
        oem: 10876,
        product: 47633,
        name: "H8SCM-F",
    },
    OemProduct {
        oem: 10876,
        product: 47744,
        name: "C2SBX",
    },
    OemProduct {
        oem: 10876,
        product: 47744,
        name: "X7SLM-L",
    },
    OemProduct {
        oem: 10876,
        product: 47889,
        name: "BHDGT",
    },
    OemProduct {
        oem: 10876,
        product: 48000,
        name: "B7DCE",
    },
    OemProduct {
        oem: 10876,
        product: 48145,
        name: "BHQG6/E",
    },
    OemProduct {
        oem: 10876,
        product: 48145,
        name: "H8DG6/i-F",
    },
    OemProduct {
        oem: 10876,
        product: 48256,
        name: "X7DWT/-INF",
    },
    OemProduct {
        oem: 10876,
        product: 48401,
        name: "H8DCT-IBQF",
    },
    OemProduct {
        oem: 10876,
        product: 48401,
        name: "H8DGT-C",
    },
    OemProduct {
        oem: 10876,
        product: 48512,
        name: "C2SBM",
    },
    OemProduct {
        oem: 10876,
        product: 48657,
        name: "H8DCT",
    },
    OemProduct {
        oem: 10876,
        product: 48768,
        name: "C2SBC",
    },
    OemProduct {
        oem: 10876,
        product: 48768,
        name: "C2SBC",
    },
    OemProduct {
        oem: 10876,
        product: 48913,
        name: "H8DME-EX-LS006",
    },
    OemProduct {
        oem: 10876,
        product: 49024,
        name: "C2SBi",
    },
    OemProduct {
        oem: 10876,
        product: 49280,
        name: "AOC-USAS-H4i",
    },
    OemProduct {
        oem: 10876,
        product: 49536,
        name: "AOC-USAS-H8i",
    },
    OemProduct {
        oem: 10876,
        product: 49792,
        name: "B7DW3",
    },
    OemProduct {
        oem: 10876,
        product: 50048,
        name: "AOC-USASLP-H8iR",
    },
    OemProduct {
        oem: 10876,
        product: 51729,
        name: "H8QGL",
    },
    OemProduct {
        oem: 10876,
        product: 51985,
        name: "H8DGT-CM",
    },
    OemProduct {
        oem: 10876,
        product: 52497,
        name: "H8DCL-6F",
    },
    OemProduct {
        oem: 10876,
        product: 52753,
        name: "H8DCT-HLN4F",
    },
    OemProduct {
        oem: 10876,
        product: 53009,
        name: "H8SML-7/i(F)",
    },
    OemProduct {
        oem: 10876,
        product: 53265,
        name: "H8QG7",
    },
    OemProduct {
        oem: 10876,
        product: 53376,
        name: "X7DWP",
    },
    OemProduct {
        oem: 10876,
        product: 53521,
        name: "H8SME-F",
    },
    OemProduct {
        oem: 10876,
        product: 53632,
        name: "X7SBi",
    },
    OemProduct {
        oem: 10876,
        product: 53888,
        name: "X7SB4/E",
    },
    OemProduct {
        oem: 10876,
        product: 54144,
        name: "X7SBA",
    },
    OemProduct {
        oem: 10876,
        product: 54400,
        name: "X7SBi-LN4",
    },
    OemProduct {
        oem: 10876,
        product: 54656,
        name: "X7SBF",
    },
    OemProduct {
        oem: 10876,
        product: 54912,
        name: "X7DCA-3/i",
    },
    OemProduct {
        oem: 10876,
        product: 55936,
        name: "C2SBM-Q",
    },
    OemProduct {
        oem: 10876,
        product: 56192,
        name: "X7DCU",
    },
    OemProduct {
        oem: 10876,
        product: 56448,
        name: "X7DCA-L",
    },
    OemProduct {
        oem: 10876,
        product: 56704,
        name: "X7SBX",
    },
    OemProduct {
        oem: 10876,
        product: 56960,
        name: "X7DCT-10G",
    },
    OemProduct {
        oem: 10876,
        product: 57216,
        name: "X7DWE",
    },
    OemProduct {
        oem: 10876,
        product: 61568,
        name: "X7SBT",
    },
    OemProduct {
        oem: 10876,
        product: 61824,
        name: "X7SBU",
    },
    OemProduct {
        oem: 10876,
        product: 62080,
        name: "X8DTN+",
    },
    OemProduct {
        oem: 10876,
        product: 62336,
        name: "X8SAX",
    },
    OemProduct {
        oem: 10876,
        product: 62592,
        name: "X7DCX",
    },
    OemProduct {
        oem: 10876,
        product: 62848,
        name: "X8ST3",
    },
    OemProduct {
        oem: 49769,
        product: 1,
        name: "VESNIN BMC",
    },
    OemProduct {
        oem: 49769,
        product: 10,
        name: "TATLIN.UNIFIED Storage Controller BMC",
    },
    OemProduct {
        oem: 49769,
        product: 20,
        name: "VEGMAN Series BMC",
    },
    OemProduct {
        oem: 49769,
        product: 21,
        name: "TATLIN Series Storage Controller BMC",
    },
];
