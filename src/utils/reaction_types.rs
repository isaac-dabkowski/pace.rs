#![allow(dead_code)]

use std::collections::HashMap;

use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy)]
pub struct ReactionType {
    pub mt_min: u16,
    pub mt_max: u16,
    pub code: &'static str,
    pub comment: &'static str,
}

impl ReactionType {
    const fn single(mt: u16, code: &'static str, comment: &'static str) -> Self {
        Self {
            mt_min: mt,
            mt_max: mt,
            code,
            comment,
        }
    }

    const fn range(mt_min: u16, mt_max: u16, code: &'static str, comment: &'static str) -> Self {
        Self {
            mt_min,
            mt_max,
            code,
            comment,
        }
    }
}

// Backing table of MT ranges and metadata.
const REACTION_TYPE_ENTRIES: [ReactionType; 79] = [
    ReactionType::single(
        1,
        "(n,total)",
        "incident neutrons only (sum over MTs 2, 4, 5, 11, 16-18, 22-26, 28-37, 41-43, 44-45, 102-117)",
    ),
    ReactionType::single(2, "(z,z0)", "elastic scattering"),
    ReactionType::single(
        3,
        "(z,nonelastic)",
        "nonelastic (sum over MTs 4, 5, 11, 16-18, 22-26, 28-37, 41-42, 44-45, 102-117) - redundant",
    ),
    ReactionType::single(
        4,
        "(z,n)",
        "total inelastic scattering (incident neutrons); production of one neutron in the exit channel (sum over MTs 51 to 91) - redundant",
    ),
    ReactionType::single(
        5,
        "(n,anything)",
        "sum of all reactions not given explicitly in another MT number, used for lumping together multiple reaction modes",
    ),
    ReactionType::single(
        11,
        "(z,2nd)",
        "production of two neutrons and a deuteron, plus a residual if any",
    ),
    ReactionType::single(
        16,
        "(z,2n)",
        "production of two neutrons, plus a residual (if any) - some nuclides miss MT 16 and have only MTs 875-891 instead",
    ),
    ReactionType::single(
        17,
        "(z,3n)",
        "production of three neutrons, plus a residual (if any)",
    ),
    ReactionType::single(
        18,
        "(z,fission)",
        "total fission (sum over MTs 19-21 and 38)",
    ),
    ReactionType::single(19, "(n,f)", "1st-chance neutron-induced fission"),
    ReactionType::single(20, "(n,nf)", "2nd-chance neutron-induced fission"),
    ReactionType::single(21, "(n,2nf)", "3rd-chance neutron-induced fission"),
    ReactionType::single(
        22,
        "(z,nα)",
        "production of a neutron and an alpha particle, plus a residual (if any)",
    ),
    ReactionType::single(
        23,
        "(z,n3α)",
        "production of a neutron and three alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        24,
        "(z,2nα)",
        "production of two neutron and an alpha particle, plus a residual (if any)",
    ),
    ReactionType::single(
        25,
        "(z,3nα)",
        "production of three neutrons and an alpha particle, plus a residual (if any)",
    ),
    ReactionType::single(
        27,
        "(n,abs)",
        "absorption (sum over MTs 18 and 102-117) - redundant",
    ),
    ReactionType::single(
        28,
        "(z,np)",
        "production of a neutron and a proton, plus a residual (if any)",
    ),
    ReactionType::single(
        29,
        "(z,n2α)",
        "production of a neutrons and two alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        30,
        "(z,2n2α)",
        "production of two neutrons and two alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        32,
        "(z,nd)",
        "production of a neutron and a deuteron, plus a residual (if any)",
    ),
    ReactionType::single(
        33,
        "(z,nt)",
        "production of a neutron and a triton, plus a residual (if any)",
    ),
    ReactionType::single(
        34,
        "(z,n3He)",
        "production of a neutron and a 3He particle, plus a residual (if any)",
    ),
    ReactionType::single(
        35,
        "(z,nd2α)",
        "production of a neutron, a deuteron and two alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        36,
        "(z,nt2α)",
        "production of a neutron, a triton, and two alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        37,
        "(z,4n)",
        "production of four neutrons, plus a residual (if any)",
    ),
    ReactionType::single(38, "(n,3nf)", "4th-chance neutron-induced fission"),
    ReactionType::single(
        41,
        "(z,2np)",
        "production of two neutrons and a proton, plus a residual (if any)",
    ),
    ReactionType::single(
        42,
        "(z,3np)",
        "production of three neutrons and a proton, plus a residual (if any)",
    ),
    ReactionType::single(
        44,
        "(z,n2p)",
        "production of a neutron and two protons, plus a residual (if any)",
    ),
    ReactionType::single(
        45,
        "(z,npα)",
        "production of a neutron, a proton and an alpha particle, plus a residual (if any)",
    ),
    ReactionType::range(
        51,
        90,
        "(z,ni)",
        "inelastic scattering to excited states; production of a neutron leaving the residual nucleus in the i-th excited state (i = MT - 50)",
    ),
    ReactionType::single(
        91,
        "(z,nc)",
        "inelastic scattering to continuum; production of a neutron in the continuum not included in the above discrete representation",
    ),
    ReactionType::single(
        101,
        "(n,disap)",
        "total absorption or disappearance (sum over MTs 102-117) - redundant",
    ),
    ReactionType::single(
        102,
        "(z,γ)",
        "radiative capture (102g/102m for transmutation to ground/isomeric state)",
    ),
    ReactionType::single(
        103,
        "(z,p)",
        "production of a proton (sum over MTs 600-647, if present), plus a residual (if any)",
    ),
    ReactionType::single(
        104,
        "(z,d)",
        "production of a deuteron (sum over MTs 650-699, if present), plus a residual (if any)",
    ),
    ReactionType::single(
        105,
        "(z,t)",
        "production of a triton (sum over MTs 700-749, if present), plus a residual (if any)",
    ),
    ReactionType::single(
        106,
        "(z,3He)",
        "production of a 3He particle (sum over MTs 750-799, if present), plus a residual (if any)",
    ),
    ReactionType::single(
        107,
        "(z,α)",
        "production of an alpha particle (sum over MTs 800-849, if present), plus a residual (if any)",
    ),
    ReactionType::single(
        108,
        "(z,2α)",
        "production of two alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        109,
        "(z,3α)",
        "production of three alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        111,
        "(z,2p)",
        "production of two protons, plus a residual (if any)",
    ),
    ReactionType::single(
        112,
        "(z,pα)",
        "production of a proton and an alpha particle, plus a residual (if any)",
    ),
    ReactionType::single(
        113,
        "(z,t2α)",
        "production of a triton and two alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        114,
        "(z,d2α)",
        "production of a deuteron and two alpha particles, plus a residual (if any)",
    ),
    ReactionType::single(
        115,
        "(z,pd)",
        "production of proton and a deuteron, plus a residual (if any)",
    ),
    ReactionType::single(
        116,
        "(z,pt)",
        "production of a proton and a triton, plus a residual (if any)",
    ),
    ReactionType::single(
        117,
        "(z,dα)",
        "production of a deuteron and an alpha particle, plus a residual (if any)",
    ),
    ReactionType::single(201, "(z,Xn)", "total neutron production - redundant"),
    ReactionType::single(202, "(z,Xγ)", "total photon production - redundant"),
    ReactionType::single(203, "(z,Xp)", "total proton production - redundant"),
    ReactionType::single(204, "(z,Xd)", "total deuteron production - redundant"),
    ReactionType::single(205, "(z,Xt)", "total triton production - redundant"),
    ReactionType::single(206, "(z,X3He)", "total 3He production - redundant"),
    ReactionType::single(207, "(z,Xα)", "total alpha production - redundant"),
    ReactionType::single(
        301,
        "(z,totheat)",
        "total heat production; total heating number multiplied by total cross section (note difference to MCNP)",
    ),
    ReactionType::single(
        443,
        "(z,kinkerma)",
        "kinematic KERMA - Note to developers: check if this needs to be multiplied by total xs",
    ),
    ReactionType::single(
        444,
        "(z,damenergy)",
        "damage-energy production - Note to developers: check if this needs to be multiplied by total xs",
    ),
    ReactionType::single(
        600,
        "(z,p0)",
        "production of a proton, leaving the residual nucleus in the ground state - MTs 600-649 can be used to replace MT 103",
    ),
    ReactionType::range(
        601,
        648,
        "(z,pi)",
        "production of a proton, leaving the residual nucleus in the i-th excited state (i = MT - 600)",
    ),
    ReactionType::single(
        649,
        "(z,pc)",
        "production of a proton in the continuum not included in the above discrete representation",
    ),
    ReactionType::single(
        650,
        "(z,d0)",
        "production of a deuteron, leaving the residual nucleus in the ground state - MTs 650-699 can be used to replace MT 104",
    ),
    ReactionType::range(
        651,
        698,
        "(z,di)",
        "production of a deuteron, leaving the residual nucleus in the i-th excited state (i = MT - 650)",
    ),
    ReactionType::single(
        699,
        "(z,dc)",
        "production of a deuteron in the continuum not included in the above discrete representation",
    ),
    ReactionType::single(
        700,
        "(z,t0)",
        "production of a triton, leaving the residual nucleus in the ground state - MTs 700-749 can be used to replace MT 105",
    ),
    ReactionType::range(
        701,
        748,
        "(z,ti)",
        "production of a triton, leaving the residual nucleus in the i-th excited state (i = MT - 700)",
    ),
    ReactionType::single(
        749,
        "(z,tc)",
        "production of a triton in the continuum not included in the above discrete representation",
    ),
    ReactionType::single(
        750,
        "(z,3He0)",
        "production of a 3He particle, leaving the residual nucleus in the ground state - MTs 750-799 can be used to replace MT 106",
    ),
    ReactionType::range(
        751,
        798,
        "(z,3Hei)",
        "production of a 3He particle, leaving the residual nucleus in the i-th excited state (i = MT - 750)",
    ),
    ReactionType::single(
        799,
        "(z,3Hec)",
        "production of a 3He particle in the continuum not included in the above discrete representation",
    ),
    ReactionType::single(
        800,
        "(z,α0)",
        "production of an alpha particle, leaving the residual nucleus in the ground state - MTs 800-849 can be used to replace MT 107",
    ),
    ReactionType::range(
        801,
        848,
        "(z,αi)",
        "production of an alpha particle, leaving the residual nucleus in the i-th excited state (i = MT - 800)",
    ),
    ReactionType::single(
        849,
        "(z,αc)",
        "production of an alpha particle in the continuum not included in the above discrete representation",
    ),
    ReactionType::single(
        875,
        "(z,2n0)",
        "production of a neutron, leaving the residual nucleus in the ground state - MTs 875-891 can be used to replace MT 16",
    ),
    ReactionType::range(
        876,
        890,
        "(z,2ni)",
        "production of a neutron, leaving the residual nucleus in the i-th excited state (i = MT - 875)",
    ),
    ReactionType::single(
        891,
        "(z,2nc)",
        "production of a neutron in the continuum not included in the above discrete representation",
    ),
    ReactionType::single(
        1002,
        "S(α,β)",
        "elastic scattering - not an official ENDF MT number",
    ),
    ReactionType::single(
        1004,
        "S(α,β)",
        "elastic scattering - not an official ENDF MT number",
    ),
];

// Public lookup table: MT -> ReactionType (ranges expanded to each MT).
pub static REACTION_TYPES: Lazy<HashMap<u16, ReactionType>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for rt in REACTION_TYPE_ENTRIES.iter() {
        for mt in rt.mt_min..=rt.mt_max {
            map.insert(mt, *rt);
        }
    }
    map
});

#[cfg(test)]
mod tests {
    use super::*;

    fn find(mt: u16) -> Option<ReactionType> {
        REACTION_TYPES.get(&mt).copied()
    }

    #[test]
    fn reaction_table_has_non_overlapping_ranges() {
        for (i, a) in REACTION_TYPE_ENTRIES.iter().enumerate() {
            assert!(a.mt_min <= a.mt_max);
            for b in REACTION_TYPE_ENTRIES.iter().skip(i + 1) {
                let overlaps = a.mt_min <= b.mt_max && b.mt_min <= a.mt_max;
                if overlaps {
                    assert!(
                        a.mt_min == a.mt_max && b.mt_min == b.mt_max,
                        "overlapping ranges for MTs {:?} and {:?}",
                        a,
                        b
                    );
                }
            }
        }
    }

    #[test]
    fn reaction_table_covers_some_key_mts() {
        let rt1 = find(1).expect("MT 1 present");
        assert_eq!(rt1.code, "(n,total)");
        assert!(rt1.comment.contains("incident neutrons only"));

        let rt18 = find(18).expect("MT 18 present");
        assert_eq!(rt18.code, "(z,fission)");

        let rt91 = find(91).expect("MT 91 present");
        assert_eq!(rt91.code, "(z,nc)");

        let rt102 = find(102).expect("MT 102 present");
        assert_eq!(rt102.code, "(z,γ)");
        assert!(rt102.comment.contains("radiative capture"));

        let rt107 = find(107).expect("MT 107 present");
        assert_eq!(rt107.code, "(z,α)");
    }

    #[test]
    fn reaction_table_covers_example_range_mts() {
        let rt_ni_51 = find(51).expect("MT 51 in (z,ni) range");
        let rt_ni_90 = find(90).expect("MT 90 in (z,ni) range");
        assert_eq!(rt_ni_51.code, "(z,ni)");
        assert_eq!(rt_ni_90.code, "(z,ni)");

        let rt_pi_601 = find(601).expect("MT 601 in (z,pi) range");
        let rt_pi_648 = find(648).expect("MT 648 in (z,pi) range");
        assert_eq!(rt_pi_601.code, "(z,pi)");
        assert_eq!(rt_pi_648.code, "(z,pi)");

        let rt_2ni_876 = find(876).expect("MT 876 in (z,2ni) range");
        let rt_2ni_890 = find(890).expect("MT 890 in (z,2ni) range");
        assert_eq!(rt_2ni_876.code, "(z,2ni)");
        assert_eq!(rt_2ni_890.code, "(z,2ni)");
    }
}
