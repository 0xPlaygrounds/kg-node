use std::collections::HashMap;

use futures::{stream, StreamExt, TryStreamExt};
use sink::kg::grc20;
use sink::system_ids;
use swc::config::SourceMapsConfig;
use swc::PrintArgs;
use swc_common::{sync::Lrc, SourceMap, Span, SyntaxContext};
use swc_sdk::{quote, quote_expr};
use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, BlockStmt, Class, ClassDecl, ClassMember, ClassMethod, ClassProp, Constructor, Decl, EsVersion, Expr, ExprStmt, Function, Ident, IdentName, MemberExpr, MemberProp, MethodKind, Param, ParamOrTsParamProp, Pat, PropName, ReturnStmt, SimpleAssignTarget, Stmt, ThisExpr, Tpl, TsInterfaceBody, TsInterfaceDecl, TsKeywordType, TsPropertySignature, TsType, TsTypeAnn, TsTypeElement
};
use swc_ecma_ast::{Program, Script, TsKeywordTypeKind};
use swc_ecma_codegen::Config;

use crate::{class_prop, constructor, ident, method, param};

pub fn test_class() -> Stmt {
    Stmt::Decl(Decl::Class(ClassDecl {
        ident: ident("Person"),
        declare: false,
        class: Box::new(Class {
            span: Span::default(),
            ctxt: SyntaxContext::from_u32(0),
            decorators: vec![],
            body: vec![
                ClassMember::ClassProp(class_prop("name", TsType::TsKeywordType(TsKeywordType {
                    span: Span::default(),
                    kind: TsKeywordTypeKind::TsStringKeyword,
                }))),
                ClassMember::ClassProp(class_prop("age", TsType::TsKeywordType(TsKeywordType {
                    span: Span::default(),
                    kind: TsKeywordTypeKind::TsNumberKeyword,
                }))),
                ClassMember::Constructor(constructor(
                    vec![
                        param("name", TsType::TsKeywordType(TsKeywordType {
                            span: Span::default(),
                            kind: TsKeywordTypeKind::TsStringKeyword,
                        })),
                        param("age", TsType::TsKeywordType(TsKeywordType {
                            span: Span::default(),
                            kind: TsKeywordTypeKind::TsNumberKeyword,
                        })),
                    ],
                    Some(vec![
                        quote_expr!("this.name = name;"),
                        quote_expr!("this.age = age;"),
                    ]),
                )),
                // ClassMember::Constructor(Constructor {
                //     span: Span::default(),
                //     ctxt: SyntaxContext::from_u32(0),
                //     key: PropName::Ident(IdentName {
                //         span: Span::default(),
                //         sym: "constructor".into(),
                //     }),
                //     params: vec![
                //         ParamOrTsParamProp::Param(param("name", TsType::TsKeywordType(TsKeywordType {
                //             span: Span::default(),
                //             kind: TsKeywordTypeKind::TsStringKeyword,
                //         }))),
                //         ParamOrTsParamProp::Param(param("age", TsType::TsKeywordType(TsKeywordType {
                //             span: Span::default(),
                //             kind: TsKeywordTypeKind::TsNumberKeyword,
                //         }))),
                //     ],
                //     body: Some(BlockStmt {
                //         span: Span::default(),
                //         ctxt: SyntaxContext::from_u32(0),
                //         stmts: vec![
                //             Stmt::Expr(ExprStmt {
                //                 span: Span::default(),
                //                 expr: quote_expr!("this.name = name;"),
                //             }),
                //             Stmt::Expr(ExprStmt {
                //                 span: Span::default(),
                //                 expr: quote_expr!("this.age = age;"),
                //             }),
                //         ],
                //     }),
                //     accessibility: None,
                //     is_optional: false,
                // }),
                // ClassMember::Method(ClassMethod {
                //     span: Span::default(),
                //     key: PropName::Ident(IdentName {
                //         span: Span::default(),
                //         sym: "greet".into(),
                //     }),
                //     function: Box::new(Function {
                //         params: vec![],
                //         decorators: vec![],
                //         span: Span::default(),
                //         ctxt: SyntaxContext::from_u32(0),
                //         body: Some(BlockStmt {
                //             span: Span::default(),
                //             ctxt: SyntaxContext::from_u32(0),
                //             stmts: vec![Stmt::Return(ReturnStmt {
                //                 span: Span::default(),
                //                 arg: Some(quote_expr!("`Hello, my name is ${this.name} and I am ${this.age} years old.`")),
                //             })],
                //         }),
                //         is_generator: false,
                //         is_async: false,
                //         type_params: None,
                //         return_type: None,
                //     }),
                //     kind: MethodKind::Method,
                //     is_static: false,
                //     accessibility: None,
                //     is_abstract: false,
                //     is_optional: false,
                //     is_override: false,
                // }),
                ClassMember::Method(method(
                    "greet",
                    vec![],
                    None,
                    Some(quote_expr!("`Hello, my name is ${this.name} and I am ${this.age} years old.`")),
                )),
            ],
            super_class: None,
            is_abstract: false,
            type_params: None,
            super_type_params: None,
            implements: vec![],
        }),
    }))
}

pub fn test_intf() -> Program {
    let stmts = vec![
        Stmt::Decl(Decl::TsInterface(Box::new(TsInterfaceDecl {
            // span: Span::new(BytePos(2), BytePos(53)),
            span: Span::default(),
            id: Ident {
                // span: Span::new(BytePos(12), BytePos(18)),
                span: Span::default(),
                ctxt: SyntaxContext::from_u32(0),
                sym: "Person".into(),
                optional: false,
            },
            declare: false,
            type_params: None,
            extends: vec![],
            body: TsInterfaceBody {
                // span: Span::new(BytePos(19), BytePos(53)),
                span: Span::default(),
                body: vec![
                    TsTypeElement::TsPropertySignature(TsPropertySignature {
                        // span: Span::new(BytePos(23), BytePos(36)),
                        span: Span::default(),
                        readonly: false,
                        key: Box::new(Expr::Ident(Ident {
                            // span: Span::new(BytePos(23), BytePos(27)),
                            span: Span::default(),
                            ctxt: SyntaxContext::from_u32(0),
                            sym: "name".into(),
                            optional: false,
                        })),
                        computed: false,
                        optional: false,
                        type_ann: Some(Box::new(TsTypeAnn {
                            // span: Span::new(BytePos(27), BytePos(35)),
                            span: Span::default(),
                            type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                                // span: Span::new(BytePos(29), BytePos(35)),
                                span: Span::default(),
                                kind: TsKeywordTypeKind::TsStringKeyword,
                            })),
                        })),
                    }),
                    TsTypeElement::TsPropertySignature(TsPropertySignature {
                        // span: Span::new(BytePos(39), BytePos(51)),
                        span: Span::default(),
                        readonly: false,
                        key: Box::new(Expr::Ident(Ident {
                            // span: Span::new(BytePos(39), BytePos(42)),
                            span: Span::default(),
                            ctxt: SyntaxContext::from_u32(0),
                            sym: "age".into(),
                            optional: false,
                        })),
                        computed: false,
                        optional: false,
                        type_ann: Some(Box::new(TsTypeAnn {
                            // span: Span::new(BytePos(42), BytePos(50)),
                            span: Span::default(),
                            type_ann: Box::new(TsType::TsKeywordType(TsKeywordType {
                                // span: Span::new(BytePos(44), BytePos(50)),
                                span: Span::default(),
                                kind: TsKeywordTypeKind::TsNumberKeyword,
                            })),
                        })),
                    }),
                ],
            },
        }))),
        // Stmt::Empty(EmptyStmt { span: Span::new(BytePos(53), BytePos(54)) }),
    ];

    Program::Script(Script {
        // span: Span::new(BytePos(0), BytePos(54)),
        span: Span::default(),
        body: stmts,
        shebang: None,
    })
}
