extern crate nom;
use nom::branch::alt;
use nom::bytes::complete::{take_until1, take_until};
use nom::combinator::fail;
use nom::multi::many0;
use nom::sequence::{preceded, delimited};
use nom::{
  IResult,
  bytes::complete::tag,
  sequence::tuple
};
use std::str;

use crate::parser::html_utils::classify_element_kind;

use super::attributes::parse_attributes;
use super::html_utils::{html_name, space0, ElementKind};
use super::structs::{ElementNode, StartingTag, Node};

pub fn parse_element_starting_tag(input: &str) -> IResult<&str, StartingTag> {
  let (input, (_, tag_name, attributes, _, ending_bracket)) = tuple((
    tag("<"),
    html_name,
    parse_attributes,
    space0,
    alt((tag(">"), tag("/>")))
  ))(input)?;

  #[cfg(dbg_print)]
  {
    println!("Tag name: {:?}", tag_name);
    println!("Attributes: {:?}", attributes);
  }

  Ok((input, StartingTag {
    tag_name,
    attributes,
    is_self_closing: ending_bracket == "/>",
    kind: classify_element_kind(&tag_name)
  }))
}

pub fn parse_element_end_tag(input: &str) -> IResult<&str, &str> {
  // eat any tag, because it may not match the start tag according to spec
  delimited(
    tag("</"),
    html_name,
    preceded(space0, tag(">"))
  )(input)
}

// parses {{ expression }}
fn parse_dynamic_expression(input: &str) -> IResult<&str, &str> {
  delimited(tag("{{"), take_until1("}}"), tag("}}"))(input)
}

pub fn parse_dynamic_expression_node(input: &str) -> IResult<&str, Node> {
  let (input, expression_content) = parse_dynamic_expression(input)?;
  Ok((input, Node::DynamicExpression { value: expression_content.trim(), template_scope: 0 }))
}

// todo implement different processing ways:
// 1: parse node start and then recursively parse children
// 2: parse node start and seek the ending tag
pub fn parse_element_node(input: &str) -> IResult<&str, Node> {
  let (input, starting_tag) = parse_element_starting_tag(input)?;

  let early_return = matches!(starting_tag.kind, ElementKind::Void) || starting_tag.is_self_closing;

  if early_return {
    return Ok((
      input,
      Node::ElementNode(ElementNode {
        starting_tag,
        children: vec![],
        template_scope: 0
      })
    ));
  }

  let (input, children) = parse_node_children(input)?;

  // parse end tag
  let (input, end_tag) = parse_element_end_tag(input)?;

  // todo pass a stack of elements instead of a single tag
  // todo handle the error? soft/hard error -> either return Err or proceed and warn
  if end_tag != starting_tag.tag_name {
    println!("End tag does not match start tag: <{}> </{}>", &starting_tag.tag_name, &end_tag);
  }

  Ok((
    input,
    Node::ElementNode(ElementNode {
      starting_tag,
      children,
      template_scope: 0
    })
  ))
}

fn parse_text_node(input: &str) -> IResult<&str, Node> {
  let mut iter = input.chars().peekable();
  let mut bytes_taken = 0;

  while let Some(ch) = iter.next() {
    if ch == '<' {
      break;
    };
    // todo support other delimiters
    if let ('{', Some('{')) = (ch, iter.peek()) {
      break;
    };
    bytes_taken += ch.len_utf8();
  }

  /* Return error if input length is too short */
  if bytes_taken == 0 {
    return fail(input);
  }

  let (text, input) = input.split_at(bytes_taken);

  Ok((
    input,
    Node::TextNode(text)
  ))
}

fn parse_comment_node(input: &str) -> IResult<&str, Node> {
  let (input, comment) = delimited(
    tag("<!--"),
    take_until("-->"),
    tag("-->")
  )(input)?;

  Ok((input, Node::CommentNode(comment)))
}

pub fn parse_root_block(input: &str) -> IResult<&str, Node> {
  // Remove leading space
  let input = input.trim_start();

  let (input, starting_tag) = parse_element_starting_tag(input)?;

  // Process rawtext nodes
  // TODO move this to parse element node definition???
  // TODO optimize not recalculating starting tag??
  // if let ElementKind::RawText = classify_element_kind(starting_tag.tag_name) {
  //   let (input, rawtext) = parse_rawtext(input)?;
  //   let (input, end_tag) = parse_element_end_tag(input)?; 

  //   // todo dedupe this check
  //   // todo pass a stack of elements instead of a single tag
  //   // todo handle the error? soft/hard error -> either return Err or proceed and warn
  //   if end_tag != starting_tag.tag_name {
  //     println!("End tag does not match start tag: <{}> </{}>", &starting_tag.tag_name, &end_tag);
  //   }

  //   return Ok((
  //     input,
  //     Node::TextNode(rawtext)
  //   ));
  // };

  let (input, children) = parse_node_children(input)?;

  let (input, end_tag) = parse_element_end_tag(input)?;

  // todo pass a stack of elements instead of a single tag
  // todo handle the error? soft/hard error -> either return Err or proceed and warn
  if end_tag != starting_tag.tag_name {
    println!("End tag does not match start tag: <{}> </{}>", &starting_tag.tag_name, &end_tag);
  }

  Ok((
    input,
    Node::ElementNode(ElementNode { starting_tag, children, template_scope: 0 })
  ))
}

/// Parses the Vue Single-File Component
///
/// The Ok variant is a tuple, where:
/// - the `.0` element is the remaining input. It should be any trailing whitespace if parsing succeeded;
/// - the `.1` element is a vector of root blocks, i.e. all `<script>`, `<template>`, `<style>` and custom blocks.
///
/// This function produces untyped and unoptimized `Node`s and
/// it also does not modify whitespace inside the blocks.
///
/// To convert `Node`s into typed blocks, use [`crate::parser::sfc_blocks::convert_node_to_typed`].
///
/// To optimize template node, use [`crate::analyzer::ast_optimizer::optimize_ast`]
pub fn parse_sfc(input: &str) -> IResult<&str, Vec<Node>> {
  many0(parse_root_block)(input)
}

fn parse_node_children(input: &str) -> IResult<&str, Vec<Node>> {
  many0(alt((
    parse_dynamic_expression_node,
    parse_comment_node,
    parse_element_node,
    parse_text_node
  )))(input)
}