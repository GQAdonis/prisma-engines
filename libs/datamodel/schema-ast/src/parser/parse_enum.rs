use super::{
    helpers::{parsing_catch_all, ToIdentifier, Token, TokenExtensions},
    parse_attribute::parse_attribute,
    parse_comments::*,
    Rule,
};
use crate::ast::{Attribute, Comment, Enum, EnumValue, Identifier};
use diagnostics::{DatamodelError, Diagnostics, Span};

pub fn parse_enum(token: &Token<'_>, diagnostics: &mut Diagnostics) -> Enum {
    let mut name: Option<Identifier> = None;
    let mut attributes: Vec<Attribute> = vec![];
    let mut values: Vec<EnumValue> = vec![];
    let mut comment: Option<Comment> = None;

    for current in token.relevant_children() {
        match current.as_rule() {
            Rule::non_empty_identifier => name = Some(current.to_id()),
            Rule::block_level_attribute => attributes.push(parse_attribute(&current, diagnostics)),
            Rule::enum_value_declaration => match parse_enum_value(&current, diagnostics) {
                Ok(enum_value) => values.push(enum_value),
                Err(err) => diagnostics.push_error(err),
            },
            Rule::comment_block => comment = parse_comment_block(&current),
            Rule::BLOCK_LEVEL_CATCH_ALL => diagnostics.push_error(DatamodelError::new_validation_error(
                "This line is not an enum value definition.".to_owned(),
                current.as_span().into(),
            )),
            _ => parsing_catch_all(&current, "enum"),
        }
    }

    match name {
        Some(name) => Enum {
            name,
            values,
            attributes,
            documentation: comment,
            span: Span::from(token.as_span()),
        },
        _ => panic!(
            "Encountered impossible enum declaration during parsing, name is missing: {:?}",
            token.as_str()
        ),
    }
}

fn parse_enum_value(token: &Token<'_>, diagnostics: &mut Diagnostics) -> Result<EnumValue, DatamodelError> {
    let mut name: Option<Identifier> = None;
    let mut attributes: Vec<Attribute> = vec![];
    let mut comments: Vec<String> = vec![];

    // todo validate that the identifier is valid???
    for current in token.relevant_children() {
        match current.as_rule() {
            Rule::non_empty_identifier => name = Some(current.to_id()),
            Rule::maybe_empty_identifier => name = Some(current.to_id()),
            Rule::attribute => attributes.push(parse_attribute(&current, diagnostics)),
            Rule::doc_comment => {
                comments.push(parse_doc_comment(&current));
            }
            Rule::doc_comment_and_new_line => {
                comments.push(parse_doc_comment(&current));
            }
            _ => parsing_catch_all(&current, "enum value"),
        }
    }

    match name {
        Some(name) => Ok(EnumValue {
            name,
            attributes,
            documentation: doc_comments_to_string(&comments),
            span: Span::from(token.as_span()),
            commented_out: false,
        }),
        _ => panic!(
            "Encountered impossible enum value declaration during parsing, name is missing: {:?}",
            token.as_str()
        ),
    }
}
