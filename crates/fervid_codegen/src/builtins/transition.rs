use fervid_core::{ElementNode, VueImports};
use swc_core::{
    common::DUMMY_SP,
    ecma::ast::{Expr, Ident},
};

use crate::CodegenContext;

impl CodegenContext {
    pub fn generate_transition(&mut self, element_node: &ElementNode) -> Expr {
        let span = DUMMY_SP; // TODO

        // _Transition
        let transition_identifier = Expr::Ident(Ident {
            span,
            sym: self.get_and_add_import_ident(VueImports::Transition),
            optional: false,
        });

        let transition_attrs =
            self.generate_builtin_attrs(&element_node.starting_tag.attributes, span);

        let transition_slots = self.generate_builtin_slots(element_node);

        let patch_flag = 0; // TODO This comes from the attributes

        self.generate_componentlike(
            transition_identifier,
            transition_attrs,
            transition_slots,
            patch_flag,
            false,
            span,
        )
    }
}

#[cfg(test)]
mod tests {
    use fervid_core::{BuiltinType, ElementKind, StartingTag, Node, AttributeOrBinding};

    use crate::test_utils::js;

    use super::*;

    #[test]
    fn it_generates_empty_transition() {
        // <transition></transition>
        test_out(
            ElementNode {
                kind: ElementKind::Builtin(BuiltinType::Transition),
                starting_tag: StartingTag {
                    tag_name: "transition",
                    attributes: vec![],
                    directives: None,
                },
                children: vec![],
                template_scope: 0,
            },
            r#"_createVNode(_Transition)"#,
        )
    }

    #[test]
    fn it_generates_transition_attrs() {
        // <transition foo="bar" :baz="qux"></transition>
        test_out(
            ElementNode {
                kind: ElementKind::Builtin(BuiltinType::Transition),
                starting_tag: StartingTag {
                    tag_name: "transition",
                    attributes: vec![
                        AttributeOrBinding::RegularAttribute {
                            name: "foo",
                            value: "bar",
                        },
                        AttributeOrBinding::VBind(fervid_core::VBindDirective {
                            argument: Some("baz".into()),
                            value: js("qux"),
                            is_camel: false,
                            is_prop: false,
                            is_attr: false,
                        }),
                    ],
                    directives: None,
                },
                children: vec![],
                template_scope: 0,
            },
            r#"_createVNode(_Transition,{foo:"bar",baz:qux})"#,
        )
    }

    #[test]
    fn it_generates_transition_children() {
        // <transition>foobar</transition>
        test_out(
            ElementNode {
                kind: ElementKind::Builtin(BuiltinType::Transition),
                starting_tag: StartingTag {
                    tag_name: "transition",
                    attributes: vec![],
                    directives: None,
                },
                children: vec![Node::Text("foobar")],
                template_scope: 0,
            },
            r#"_createVNode(_Transition,null,{"default":_withCtx(()=>[_createTextVNode("foobar")]),_:1})"#,
        )
    }

    #[test]
    fn it_generates_full_transition() {
        // <transition foo="bar" :baz="qux">foobar</transition>
        test_out(
            ElementNode {
                kind: ElementKind::Builtin(BuiltinType::Transition),
                starting_tag: StartingTag {
                    tag_name: "transition",
                    attributes: vec![
                        AttributeOrBinding::RegularAttribute {
                            name: "foo",
                            value: "bar",
                        },
                        AttributeOrBinding::VBind(fervid_core::VBindDirective {
                            argument: Some("baz".into()),
                            value: js("qux"),
                            is_camel: false,
                            is_prop: false,
                            is_attr: false,
                        }),
                    ],
                    directives: None,
                },
                children: vec![Node::Text("foobar")],
                template_scope: 0,
            },
            r#"_createVNode(_Transition,{foo:"bar",baz:qux},{"default":_withCtx(()=>[_createTextVNode("foobar")]),_:1})"#,
        )
    }

    fn test_out(input: ElementNode, expected: &str) {
        let mut ctx = CodegenContext::default();
        let out = ctx.generate_transition(&input);
        assert_eq!(crate::test_utils::to_str(out), expected)
    }
}