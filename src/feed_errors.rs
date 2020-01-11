use custom_error::custom_error;

custom_error!{
    pub UnparseableFeed
    DateIsNotRFC2822{value:String} = "Date {value} is not RFC-2822 compliant",
    DateIsNotRFC3339{value:String} = "Date {value} is not RFC-3339 compliant",
    DateIsNeitherRFC2822NorRFC3339{value:String} = "Date {value} is neither RFC-2822 nor RFC-3339 compliant",
    ChronoCantParse{source: chrono::ParseError} = "chrono can't parse date",
    NoDateFound = "absolutly no date field was found in feed",
    CantExtractImages{source: super::message::UnprocessableMessage} = "Seems like it was not possible to read message contained images"
}