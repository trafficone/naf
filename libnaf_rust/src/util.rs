//- Utility functions for the handling of NAF objects
/*
    - put magic number
    - read variable length encoded number
    - write variable length encoded number
    - init tables
    - out of memory (Error)
    - incomplete (Error)
    - (die) (panic handle function)
    - (err) (print error function)
    - (msg) (print message function)
    - (malloc or die)
    - (fgetc or incomplete)
*/

use phf::phf_map;
use std::char;
use std::fs::File;
use std::io::Write;

/// Constant Lookup Table for u4 to character to map four-bit
/// encoding values to their associated nucleotide/amino acid
const NUC_LUT: [char; 16] = [
    '-', 'T', 'G', 'K', 'C', 'Y', 'S', 'B', 'A', 'W', 'R', 'D', 'M', 'H', 'V', 'N',
];

//const fn nuc_revlut(charinput: &char) -> u8 {
//match charinput {

/// Constant lookup for character to u4 (represented as u8)
/// which represents the four-bit encoding for nucleotides/amino acids
const NUC_REVLUT: phf::Map<char, u8> = phf_map! {
        '-' => 0,
        'T' => 1,
        'G' => 2,
        'K' => 3,
        'C' => 4,
        'Y' => 5,
        'S' => 6,
        'B' => 7,
        'A' => 8,
        'W' => 9,
        'R' => 10,
        'D' => 11,
        'M' => 12,
        'H' => 13,
        'V' => 14,
        'N' => 15
};

/// Constant lookup for u8 to tuple (char,char)
const NUC_TO_FOURBIT: phf::Map<&'static str, u8> = phf_map! {
    "--"=>0,"-T"=>16,"-G"=>32,"-K"=>48,"-C"=>64,"-Y"=>80,"-S"=>96,"-B"=>112,
    "-A"=>128,"-W"=>144,"-R"=>160,"-D"=>176,"-M"=>192,"-H"=>208,"-V"=>224,
    "-N"=>240,"T-"=>1,"TT"=>17,"TG"=>33,"TK"=>49,"TC"=>65,"TY"=>81,"TS"=>97,"TB"=>113,
    "TA"=>129,"TW"=>145,"TR"=>161,"TD"=>177,"TM"=>193,"TH"=>209,"TV"=>225,"TN"=>241,
    "G-"=>2,"GT"=>18,"GG"=>34,"GK"=>50,"GC"=>66,"GY"=>82,"GS"=>98,"GB"=>114,"GA"=>130,
    "GW"=>146,"GR"=>162,"GD"=>178,"GM"=>194,"GH"=>210,"GV"=>226,"GN"=>242,"K-"=>3,"KT"=>19,
    "KG"=>35,"KK"=>51,"KC"=>67,"KY"=>83,"KS"=>99,"KB"=>115,"KA"=>131,"KW"=>147,"KR"=>163,
    "KD"=>179,"KM"=>195,"KH"=>211,"KV"=>227,"KN"=>243,"C-"=>4,"CT"=>20,"CG"=>36,"CK"=>52,
    "CC"=>68,"CY"=>84,"CS"=>100,"CB"=>116,"CA"=>132,"CW"=>148,"CR"=>164,"CD"=>180,"CM"=>196,
    "CH"=>212,"CV"=>228,"CN"=>244,"Y-"=>5,"YT"=>21,"YG"=>37,"YK"=>53,"YC"=>69,"YY"=>85,"YS"=>101,
    "YB"=>117,"YA"=>133,"YW"=>149,"YR"=>165,"YD"=>181,"YM"=>197,"YH"=>213,"YV"=>229,"YN"=>245,
    "S-"=>6,"ST"=>22,"SG"=>38,"SK"=>54,"SC"=>70,"SY"=>86,"SS"=>102,"SB"=>118,"SA"=>134,"SW"=>150,
    "SR"=>166,"SD"=>182,"SM"=>198,"SH"=>214,"SV"=>230,"SN"=>246,"B-"=>7,"BT"=>23,"BG"=>39,"BK"=>55,
    "BC"=>71,"BY"=>87,"BS"=>103,"BB"=>119,"BA"=>135,"BW"=>151,"BR"=>167,"BD"=>183,"BM"=>199,
    "BH"=>215,"BV"=>231,"BN"=>247,"A-"=>8,"AT"=>24,"AG"=>40,"AK"=>56,"AC"=>72,"AY"=>88,
    "AS"=>104,"AB"=>120,"AA"=>136,"AW"=>152,"AR"=>168,"AD"=>184,"AM"=>200,"AH"=>216,"AV"=>232,
    "AN"=>248,"W-"=>9,"WT"=>25,"WG"=>41,"WK"=>57,"WC"=>73,"WY"=>89,"WS"=>105,"WB"=>121,
    "WA"=>137,"WW"=>153,"WR"=>169,"WD"=>185,"WM"=>201,"WH"=>217,"WV"=>233,"WN"=>249,"R-"=>10,
    "RT"=>26,"RG"=>42,"RK"=>58,"RC"=>74,"RY"=>90,"RS"=>106,"RB"=>122,"RA"=>138,"RW"=>154,
    "RR"=>170,"RD"=>186,"RM"=>202,"RH"=>218,"RV"=>234,"RN"=>250,"D-"=>11,"DT"=>27,"DG"=>43,
    "DK"=>59,"DC"=>75,"DY"=>91,"DS"=>107,"DB"=>123,"DA"=>139,"DW"=>155,"DR"=>171,"DD"=>187,
    "DM"=>203,"DH"=>219,"DV"=>235,"DN"=>251,"M-"=>12,"MT"=>28,"MG"=>44,"MK"=>60,"MC"=>76,
    "MY"=>92,"MS"=>108,"MB"=>124,"MA"=>140,"MW"=>156,"MR"=>172,"MD"=>188,"MM"=>204,"MH"=>220,
    "MV"=>236,"MN"=>252,"H-"=>13,"HT"=>29,"HG"=>45,"HK"=>61,"HC"=>77,"HY"=>93,"HS"=>109,
    "HB"=>125,"HA"=>141,"HW"=>157,"HR"=>173,"HD"=>189,"HM"=>205,"HH"=>221,"HV"=>237,
    "HN"=>253,"V-"=>14,"VT"=>30,"VG"=>46,"VK"=>62,"VC"=>78,"VY"=>94,"VS"=>110,"VB"=>126,
    "VA"=>142,"VW"=>158,"VR"=>174,"VD"=>190,"VM"=>206,"VH"=>222,"VV"=>238,"VN"=>254,"N-"=>15,
    "NT"=>31,"NG"=>47,"NK"=>63,"NC"=>79,"NY"=>95,"NS"=>111,"NB"=>127,"NA"=>143,"NW"=>159,
    "NR"=>175,"ND"=>191,"NM"=>207,"NH"=>223,"NV"=>239,"NN"=>255
};

const FOURBIT_LUT: [&str; 256] = [
    "--", "T-", "G-", "K-", "C-", "Y-", "S-", "B-", "A-", "W-", "R-", "D-", "M-", "H-", "V-", "N-",
    "-T", "TT", "GT", "KT", "CT", "YT", "ST", "BT", "AT", "WT", "RT", "DT", "MT", "HT", "VT", "NT",
    "-G", "TG", "GG", "KG", "CG", "YG", "SG", "BG", "AG", "WG", "RG", "DG", "MG", "HG", "VG", "NG",
    "-K", "TK", "GK", "KK", "CK", "YK", "SK", "BK", "AK", "WK", "RK", "DK", "MK", "HK", "VK", "NK",
    "-C", "TC", "GC", "KC", "CC", "YC", "SC", "BC", "AC", "WC", "RC", "DC", "MC", "HC", "VC", "NC",
    "-Y", "TY", "GY", "KY", "CY", "YY", "SY", "BY", "AY", "WY", "RY", "DY", "MY", "HY", "VY", "NY",
    "-S", "TS", "GS", "KS", "CS", "YS", "SS", "BS", "AS", "WS", "RS", "DS", "MS", "HS", "VS", "NS",
    "-B", "TB", "GB", "KB", "CB", "YB", "SB", "BB", "AB", "WB", "RB", "DB", "MB", "HB", "VB", "NB",
    "-A", "TA", "GA", "KA", "CA", "YA", "SA", "BA", "AA", "WA", "RA", "DA", "MA", "HA", "VA", "NA",
    "-W", "TW", "GW", "KW", "CW", "YW", "SW", "BW", "AW", "WW", "RW", "DW", "MW", "HW", "VW", "NW",
    "-R", "TR", "GR", "KR", "CR", "YR", "SR", "BR", "AR", "WR", "RR", "DR", "MR", "HR", "VR", "NR",
    "-D", "TD", "GD", "KD", "CD", "YD", "SD", "BD", "AD", "WD", "RD", "DD", "MD", "HD", "VD", "ND",
    "-M", "TM", "GM", "KM", "CM", "YM", "SM", "BM", "AM", "WM", "RM", "DM", "MM", "HM", "VM", "NM",
    "-H", "TH", "GH", "KH", "CH", "YH", "SH", "BH", "AH", "WH", "RH", "DH", "MH", "HH", "VH", "NH",
    "-V", "TV", "GV", "KV", "CV", "YV", "SV", "BV", "AV", "WV", "RV", "DV", "MV", "HV", "VV", "NV",
    "-N", "TN", "GN", "KN", "CN", "YN", "SN", "BN", "AN", "WN", "RN", "DN", "MN", "HN", "VN", "NN",
];

fn _chars_to_fourbits(left: &char, right: &char) -> u8 {
    NUC_TO_FOURBIT[&String::from_iter([left, right])]
}

/// convert bytes of characters to 4bit encoded sequence data
pub fn chars_to_fourbits(seq: Vec<u8>) -> Vec<u8> {
    seq.chunks(2)
        .map(|w| {
            let w_after: char;
            if w.len() == 1 {
                w_after = '-';
            } else {
                w_after = w[1] as char;
            }
            _chars_to_fourbits(&(w[0] as char), &w_after)
        })
        .collect::<Vec<u8>>()
}

/// covert bytes of 4bit encoded sequence data to sequence characters
pub fn fourbits_to_chars(seq: Vec<u8>, seqlen: usize) -> String {
    let mut res = seq
        .iter()
        .map(|&x| FOURBIT_LUT[x as usize])
        .fold(String::new(), |acc: String, x: &str| acc + x);
    if seq.len() * 2 > seqlen {
        res.truncate(seqlen);
    } else if seq.len() * 2 < seqlen {
        res.push_str(
            std::iter::repeat("-")
                .take(seqlen - (seq.len() * 2))
                .collect::<String>()
                .as_str(),
        );
    }
    res.to_string()
}

#[cfg(test)]
mod tests {
    use crate::util::{chars_to_fourbits, fourbits_to_chars};
    #[test]
    fn test_chars_to_fourbits() {
        let sequence = "ACTTAGACATGCAGTAAAAGCTA".as_bytes().to_vec();
        let fourbits = vec![72, 17, 40, 72, 24, 66, 40, 129, 136, 40, 20, 8];
        assert_eq!(chars_to_fourbits(sequence), fourbits)
    }

    #[test]
    fn test_fourbits_to_chars() {
        let sequence = "ACTTAGACATGCAGTAAAAGCTA";
        let fourbits = vec![72, 17, 40, 72, 24, 66, 40, 129, 136, 40, 20, 8];
        assert_eq!(sequence, fourbits_to_chars(fourbits, 23))
    }
}

pub fn put_magic_number(writing_file: &mut File) {
    // Raw string literal for bytes
    writing_file.write_all(r"\x01\xf9\xec".as_bytes()).unwrap();
}
