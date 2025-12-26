#![allow(dead_code)]

use std::collections::HashMap;

use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy)]
pub struct Element {
    pub z: u8,
    pub symbol: &'static str,
    pub name: &'static str,
}

// Public lookup table: Z -> Element.
pub static PERIODIC_TABLE: Lazy<HashMap<u8, Element>> = Lazy::new(|| {
    HashMap::from([
        (1,  Element { z: 1,  symbol: "H",  name: "Hydrogen" }),
        (2,  Element { z: 2,  symbol: "He", name: "Helium" }),
        (3,  Element { z: 3,  symbol: "Li", name: "Lithium" }),
        (4,  Element { z: 4,  symbol: "Be", name: "Beryllium" }),
        (5,  Element { z: 5,  symbol: "B",  name: "Boron" }),
        (6,  Element { z: 6,  symbol: "C",  name: "Carbon" }),
        (7,  Element { z: 7,  symbol: "N",  name: "Nitrogen" }),
        (8,  Element { z: 8,  symbol: "O",  name: "Oxygen" }),
        (9,  Element { z: 9,  symbol: "F",  name: "Fluorine" }),
        (10, Element { z: 10, symbol: "Ne", name: "Neon" }),
        (11, Element { z: 11, symbol: "Na", name: "Sodium" }),
        (12, Element { z: 12, symbol: "Mg", name: "Magnesium" }),
        (13, Element { z: 13, symbol: "Al", name: "Aluminum" }),
        (14, Element { z: 14, symbol: "Si", name: "Silicon" }),
        (15, Element { z: 15, symbol: "P",  name: "Phosphorus" }),
        (16, Element { z: 16, symbol: "S",  name: "Sulfur" }),
        (17, Element { z: 17, symbol: "Cl", name: "Chlorine" }),
        (18, Element { z: 18, symbol: "Ar", name: "Argon" }),
        (19, Element { z: 19, symbol: "K",  name: "Potassium" }),
        (20, Element { z: 20, symbol: "Ca", name: "Calcium" }),
        (21, Element { z: 21, symbol: "Sc", name: "Scandium" }),
        (22, Element { z: 22, symbol: "Ti", name: "Titanium" }),
        (23, Element { z: 23, symbol: "V",  name: "Vanadium" }),
        (24, Element { z: 24, symbol: "Cr", name: "Chromium" }),
        (25, Element { z: 25, symbol: "Mn", name: "Manganese" }),
        (26, Element { z: 26, symbol: "Fe", name: "Iron" }),
        (27, Element { z: 27, symbol: "Co", name: "Cobalt" }),
        (28, Element { z: 28, symbol: "Ni", name: "Nickel" }),
        (29, Element { z: 29, symbol: "Cu", name: "Copper" }),
        (30, Element { z: 30, symbol: "Zn", name: "Zinc" }),
        (31, Element { z: 31, symbol: "Ga", name: "Gallium" }),
        (32, Element { z: 32, symbol: "Ge", name: "Germanium" }),
        (33, Element { z: 33, symbol: "As", name: "Arsenic" }),
        (34, Element { z: 34, symbol: "Se", name: "Selenium" }),
        (35, Element { z: 35, symbol: "Br", name: "Bromine" }),
        (36, Element { z: 36, symbol: "Kr", name: "Krypton" }),
        (37, Element { z: 37, symbol: "Rb", name: "Rubidium" }),
        (38, Element { z: 38, symbol: "Sr", name: "Strontium" }),
        (39, Element { z: 39, symbol: "Y",  name: "Yttrium" }),
        (40, Element { z: 40, symbol: "Zr", name: "Zirconium" }),
        (41, Element { z: 41, symbol: "Nb", name: "Niobium" }),
        (42, Element { z: 42, symbol: "Mo", name: "Molybdenum" }),
        (43, Element { z: 43, symbol: "Tc", name: "Technetium" }),
        (44, Element { z: 44, symbol: "Ru", name: "Ruthenium" }),
        (45, Element { z: 45, symbol: "Rh", name: "Rhodium" }),
        (46, Element { z: 46, symbol: "Pd", name: "Palladium" }),
        (47, Element { z: 47, symbol: "Ag", name: "Silver" }),
        (48, Element { z: 48, symbol: "Cd", name: "Cadmium" }),
        (49, Element { z: 49, symbol: "In", name: "Indium" }),
        (50, Element { z: 50, symbol: "Sn", name: "Tin" }),
        (51, Element { z: 51, symbol: "Sb", name: "Antimony" }),
        (52, Element { z: 52, symbol: "Te", name: "Tellurium" }),
        (53, Element { z: 53, symbol: "I",  name: "Iodine" }),
        (54, Element { z: 54, symbol: "Xe", name: "Xenon" }),
        (55, Element { z: 55, symbol: "Cs", name: "Cesium" }),
        (56, Element { z: 56, symbol: "Ba", name: "Barium" }),
        (57, Element { z: 57, symbol: "La", name: "Lanthanum" }),
        (58, Element { z: 58, symbol: "Ce", name: "Cerium" }),
        (59, Element { z: 59, symbol: "Pr", name: "Praseodymium" }),
        (60, Element { z: 60, symbol: "Nd", name: "Neodymium" }),
        (61, Element { z: 61, symbol: "Pm", name: "Promethium" }),
        (62, Element { z: 62, symbol: "Sm", name: "Samarium" }),
        (63, Element { z: 63, symbol: "Eu", name: "Europium" }),
        (64, Element { z: 64, symbol: "Gd", name: "Gadolinium" }),
        (65, Element { z: 65, symbol: "Tb", name: "Terbium" }),
        (66, Element { z: 66, symbol: "Dy", name: "Dysprosium" }),
        (67, Element { z: 67, symbol: "Ho", name: "Holmium" }),
        (68, Element { z: 68, symbol: "Er", name: "Erbium" }),
        (69, Element { z: 69, symbol: "Tm", name: "Thulium" }),
        (70, Element { z: 70, symbol: "Yb", name: "Ytterbium" }),
        (71, Element { z: 71, symbol: "Lu", name: "Lutetium" }),
        (72, Element { z: 72, symbol: "Hf", name: "Hafnium" }),
        (73, Element { z: 73, symbol: "Ta", name: "Tantalum" }),
        (74, Element { z: 74, symbol: "W",  name: "Tungsten" }),
        (75, Element { z: 75, symbol: "Re", name: "Rhenium" }),
        (76, Element { z: 76, symbol: "Os", name: "Osmium" }),
        (77, Element { z: 77, symbol: "Ir", name: "Iridium" }),
        (78, Element { z: 78, symbol: "Pt", name: "Platinum" }),
        (79, Element { z: 79, symbol: "Au", name: "Gold" }),
        (80, Element { z: 80, symbol: "Hg", name: "Mercury" }),
        (81, Element { z: 81, symbol: "Tl", name: "Thallium" }),
        (82, Element { z: 82, symbol: "Pb", name: "Lead" }),
        (83, Element { z: 83, symbol: "Bi", name: "Bismuth" }),
        (84, Element { z: 84, symbol: "Po", name: "Polonium" }),
        (85, Element { z: 85, symbol: "At", name: "Astatine" }),
        (86, Element { z: 86, symbol: "Rn", name: "Radon" }),
        (87, Element { z: 87, symbol: "Fr", name: "Francium" }),
        (88, Element { z: 88, symbol: "Ra", name: "Radium" }),
        (89, Element { z: 89, symbol: "Ac", name: "Actinium" }),
        (90, Element { z: 90, symbol: "Th", name: "Thorium" }),
        (91, Element { z: 91, symbol: "Pa", name: "Protactinium" }),
        (92, Element { z: 92, symbol: "U",  name: "Uranium" }),
        (93, Element { z: 93, symbol: "Np", name: "Neptunium" }),
        (94, Element { z: 94, symbol: "Pu", name: "Plutonium" }),
        (95, Element { z: 95, symbol: "Am", name: "Americium" }),
        (96, Element { z: 96, symbol: "Cm", name: "Curium" }),
        (97, Element { z: 97, symbol: "Bk", name: "Berkelium" }),
        (98, Element { z: 98, symbol: "Cf", name: "Californium" }),
        (99, Element { z: 99, symbol: "Es", name: "Einsteinium" }),
        (100, Element { z: 100, symbol: "Fm", name: "Fermium" }),
        (101, Element { z: 101, symbol: "Md", name: "Mendelevium" }),
        (102, Element { z: 102, symbol: "No", name: "Nobelium" }),
        (103, Element { z: 103, symbol: "Lr", name: "Lawrencium" }),
        (104, Element { z: 104, symbol: "Rf", name: "Rutherfordium" }),
        (105, Element { z: 105, symbol: "Db", name: "Dubnium" }),
        (106, Element { z: 106, symbol: "Sg", name: "Seaborgium" }),
        (107, Element { z: 107, symbol: "Bh", name: "Bohrium" }),
        (108, Element { z: 108, symbol: "Hs", name: "Hassium" }),
        (109, Element { z: 109, symbol: "Mt", name: "Meitnerium" }),
        (110, Element { z: 110, symbol: "Ds", name: "Darmstadtium" }),
        (111, Element { z: 111, symbol: "Rg", name: "Roentgenium" }),
        (112, Element { z: 112, symbol: "Cn", name: "Copernicium" }),
        (113, Element { z: 113, symbol: "Nh", name: "Nihonium" }),
        (114, Element { z: 114, symbol: "Fl", name: "Flerovium" }),
        (115, Element { z: 115, symbol: "Mc", name: "Moscovium" }),
        (116, Element { z: 116, symbol: "Lv", name: "Livermorium" }),
        (117, Element { z: 117, symbol: "Ts", name: "Tennessine" }),
        (118, Element { z: 118, symbol: "Og", name: "Oganesson" }),
    ])
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_periodic_table_has_expected_length() {
        assert_eq!(PERIODIC_TABLE.len(), 118);
    }

    #[test]
    fn test_first_and_last_elements_are_correct() {
        let first = PERIODIC_TABLE.get(&1).unwrap();
        assert_eq!(first.z, 1);
        assert_eq!(first.symbol, "H");
        assert_eq!(first.name, "Hydrogen");

        let last = PERIODIC_TABLE.get(&118).unwrap();
        assert_eq!(last.z, 118);
        assert_eq!(last.symbol, "Og");
        assert_eq!(last.name, "Oganesson");
    }

    #[test]
    fn test_atomic_numbers_are_sequential_and_unique() {
        for z in 1u8..=118 {
            let element = PERIODIC_TABLE.get(&z).unwrap();
            assert_eq!(element.z, z, "z mismatch at index {}", z);
        }
    }

    #[test]
    fn test_symbols_and_names_match_some_key_elements() {
        let check = |z: u8, symbol: &str, name: &str| {
            let e = PERIODIC_TABLE.get(&z).unwrap();
            assert_eq!(e.symbol, symbol);
            assert_eq!(e.name, name);
        };

        check(1, "H", "Hydrogen");
        check(2, "He", "Helium");
        check(6, "C", "Carbon");
        check(8, "O", "Oxygen");
        check(26, "Fe", "Iron");
        check(79, "Au", "Gold");
        check(82, "Pb", "Lead");
        check(92, "U", "Uranium");
    }
}
