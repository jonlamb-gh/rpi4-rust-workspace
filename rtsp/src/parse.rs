use nom::IResult;

pub trait Parse {
    type Type;

    fn parse(input: &str) -> IResult<&str, Self::Type>;
}
