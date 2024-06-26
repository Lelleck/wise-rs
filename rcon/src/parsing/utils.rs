use std::{num::ParseIntError, time::Duration};

use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1, hex_digit1},
    combinator::{map, map_res},
    sequence::tuple,
    IResult,
};
use uuid::Uuid;

pub fn parse_u64(input: &str) -> Result<u64, ParseIntError> {
    input.parse()
}

pub fn take_u64(input: &str) -> IResult<&str, u64> {
    let (input, num_str) = digit1(input)?;
    let num = num_str.parse().map_err(|_| {
        nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;
    Ok((input, num))
}

pub fn take_uuid(input: &str) -> IResult<&str, Uuid> {
    map_res(
        tuple((
            hex_digit1,
            char('-'),
            hex_digit1,
            char('-'),
            hex_digit1,
            char('-'),
            hex_digit1,
            char('-'),
            hex_digit1,
        )),
        |(part1, _, part2, _, part3, _, part4, _, part5)| {
            Uuid::parse_str(&format!(
                "{}-{}-{}-{}-{}",
                part1, part2, part3, part4, part5
            ))
        },
    )(input)
}

pub fn take_duration(input: &str) -> IResult<&str, Duration> {
    map(
        tuple((take_u64, tag(":"), take_u64, tag(":"), take_u64)),
        |(hours, _, minutes, _, seconds)| {
            Duration::from_secs(hours * 3600 + minutes * 60 + seconds)
        },
    )(input)
}
