mod ru_ru {
    anylang::include_json_dir!("./tests/lang", "ru_RU");
}

mod en_us {
    anylang::include_json_dir!("./tests/lang", "en_US");
}

mod en_uk {
    anylang::include_json_dir!("./tests/lang", "en_UK");
}

mod ru_by {
    anylang::include_json_dir!("./tests/lang", "ru_BY");
}

mod de_de {
    anylang::include_json_dir!("./tests/lang", "de_DE");
}

mod fr_fr {
    anylang::include_json_dir!("./tests/lang", "fr_FR");
}

#[test]
fn check_obj() {
    use crate::ru_ru::*;

    assert_eq!(lang::PING, "понг");
    assert_eq!(lang::dummy::FOO, "базз");
    assert_eq!(lang::dummy::SOME, ["ничего", "или", "0"]);
    assert_eq!(lang::rust::RUST, "раст");
    assert!(lang::rust::IS.is_empty());
    assert_eq!(lang::rust::good::TRUE, ["1", "true"]);
}

#[test]
fn check_arr() {
    use crate::en_us::*;

    assert_eq!(lang::PING, "pong");
    assert_eq!(lang::FOO, "buzz");
    assert_eq!(lang::dummy::SOME, ["none", "or", "0"]);
    assert_eq!(lang::rust::RUST, "rust");
    assert!(lang::rust::IS.is_empty());
    assert_eq!(lang::rust::good::TRUE, ["1", "true"]);
}

#[test]
fn check_null() {
    use crate::en_uk::*;

    assert!(lang::EN_UK.is_empty());
}

#[test]
fn check_big_num() {
    use crate::ru_by::*;

    assert_eq!(lang::RU_BY, "1e37");
}

#[test]
fn check_small_num() {
    use crate::de_de::*;

    assert_eq!(lang::DE_DE, "228.01");
}

#[test]
fn check_bool() {
    use crate::fr_fr::*;

    assert_eq!(lang::FR_FR, "false");
}
