use num_derive::FromPrimitive;
use std::fmt;

#[derive(FromPrimitive, Debug)]
pub enum Character {
    Sol = 0,
    Ky = 1,
    May = 2,
    Axl = 3,
    Chipp = 4,
    Potemkin = 5,
    Faust = 6,
    Millia = 7,
    Zato = 8,
    Ramlethal = 9,
    Leo = 10,
    Nagoriyuki = 11,
    Giovanna = 12,
    Anji = 13,
    INo = 14,
    Goldlewis = 15,
    JackO = 16,
    HappyChaos = 17,
    Baiken = 18,
    Testament = 19,
    Bridget = 20,
    Sin = 21,
}

pub fn convert_to_character<T>(num: T) -> Result<Character, CharacterDoesntExist>
where
    u8: From<T>,
{
    let res = match num::FromPrimitive::from_u8(num.into()) {
        Some(character) => Ok(character),
        None => return Err(CharacterDoesntExist),
    };
    res
}

#[derive(Debug, Clone)]
pub struct CharacterDoesntExist;
impl fmt::Display for CharacterDoesntExist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "This character does not exist.")
    }
}
