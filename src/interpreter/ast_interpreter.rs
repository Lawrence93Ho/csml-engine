//TODO: make a better error system
use std::io::{Error, ErrorKind, Result};
use crate::parser::ast::*;
use crate::interpreter:: {
    builtins::*,
    message::*,
    json_to_rust::*,
    csml_rules::*
};

pub struct AstInterpreter<'a> {
    pub memory: &'a Memory,
    pub event: &'a Option<Event>,
}

impl<'a> AstInterpreter<'a> {
    fn match_builtin(&self, builtin: &Ident, args: &Expr) -> Result<MessageType> {
        match builtin {
            Ident(arg) if arg == "Typing"=> Ok(MessageType::Msg(Message::new(typing(args), arg.to_string() ))),
            Ident(arg) if arg == "Wait"  => Ok(MessageType::Msg(Message::new(wait(args), arg.to_string() ))),
            Ident(arg) if arg == "Text"  => Ok(MessageType::Msg(Message::new(text(args), arg.to_string() ))),
            Ident(arg) if arg == "Url"   => Ok(MessageType::Msg(Message::new(url(args), arg.to_string() ))),
            Ident(arg) if arg == "Image" => Ok(MessageType::Msg(Message::new(img(args), arg.to_string() ))),
            Ident(arg) if arg == "OneOf" => Ok(MessageType::Msg(Message::new(one_of(args), "text".to_string()))),
            Ident(arg) if arg == "Button"=> Ok(button(args)),
            Ident(_arg)                  => Err(Error::new(ErrorKind::Other, "Error no builtin found")),
        }
    }

    // Match reserved Actions ###############################################################

    fn match_action(&self, action: &Expr) -> Result<MessageType> {
        match action {
            Expr::Action { builtin, args }  => self.match_builtin(builtin, args),
            Expr::LitExpr(_)                => Ok(MessageType::Msg(Message::new(action, "text".to_string()))),
            //NOTE: ONLY Work for LITERAR::STRINGS for now
            Expr::BuilderExpr(..)           =>
                match self.get_var_from_ident(action) {
                    Ok(val) => Ok(MessageType::Msg(Message::new(&Expr::LitExpr(val), "text".to_string()))),
                    Err(e)  => Err(e)
                }
            ,
            //NOTE: ONLY Work for LITERAR::STRINGS for now
            Expr::ComplexLiteral(vec)       => {
                match self.get_string_from_complexstring(vec) {
                    Ok(val) => Ok(MessageType::Msg(Message::new(&Expr::LitExpr(val), "text".to_string()))),
                    Err(e)  => Err(e)
                }
            },
            Expr::Empty                     => Ok(MessageType::Empty),
            _                               => Err(Error::new(ErrorKind::Other, "Error must be a valid action")),
        }
    }

    fn match_reserved(&self, reserved: &Ident, arg: &Expr) -> Result<MessageType> {
        match reserved {
            Ident(ident) if ident == "ask" || ident == "respond" => { 
                if let Expr::VecExpr(block) = arg {
                    match self.match_block(block) {
                        Ok(root)  => Ok(MessageType::Msgs(root.messages)),
                        Err(e)     => Err(e)
                    }
                } else {
                    Err(Error::new(ErrorKind::Other, "Error ask/respond must have a block"))
                }
            }
            Ident(ident) if ident == "say"      => {
                self.match_action(arg)
            }
            Ident(ident) if ident == "retry"    => {
                //NOTE: check nbr of retrys
                // save new option if exist
                self.match_action(arg)
            }
            _                                   => {
                Err(Error::new(ErrorKind::Other, "Error block must start with a reserved keyword"))
            }
        }
    }

    // Match reserved Actions ###############################################################
    fn cmp_lit(&self, infix: &Infix, lit1: &Literal, lit2: &Literal) -> bool {
        match infix {
            Infix::Equal                => lit1 == lit2,
            Infix::GreaterThanEqual     => lit1 >= lit2,
            Infix::LessThanEqual        => lit1 <= lit2,
            Infix::GreaterThan          => lit1 > lit2,
            Infix::LessThan             => lit1 < lit2,
            Infix::And                  => true,
            Infix::Or                   => true,
        }
    }

    fn cmp_bool(&self, infix: &Infix, b1: bool, b2: bool) -> bool {
        match infix {
            Infix::Equal                => b1 == b2,
            Infix::GreaterThanEqual     => b1 >= b2,
            Infix::LessThanEqual        => b1 <= b2,
            Infix::GreaterThan          => b1 & !b2,
            Infix::LessThan             => !b1 & b2,
            Infix::And                  => b1 && b2,
            Infix::Or                   => b1 || b2,
        }
    }

    // VARS ##################################################################################
    fn gen_literal_form_event(&self) -> Result<Literal> {
        match self.event {
            Some(event)        => {
                match event.payload {
                    PayLoad{content_type: ref t, content: ref c} if t == "text" => Ok(Literal::StringLiteral(c.text.to_string())),
                    _                                                           => Err(Error::new(ErrorKind::Other, "event type is unown")),
                }
            },
            None               => Err(Error::new(ErrorKind::Other, "no event is received"))
        }
    }

    fn search_var_memory(&self, name: &Ident) -> Result<Literal> {
        match name {
            Ident(var) if self.memory.metadata.contains_key(var) => self.memorytype_to_literal(self.memory.metadata.get(var)),
            Ident(var) if self.memory.current.contains_key(var)  => self.memorytype_to_literal(self.memory.current.get(var)),
            Ident(var) if self.memory.past.contains_key(var)     => self.memorytype_to_literal(self.memory.past.get(var)),
            _                                                    => Err(Error::new(ErrorKind::Other, "unown variable in memory V2")),
        }
    }

    fn get_var(&self, name: &Ident) -> Result<Literal> {
        match name {
            Ident(var) if var == "event"    => self.gen_literal_form_event(),
            Ident(_)                        => self.search_var_memory(name),
        }
    }

    fn get_string_from_complexstring(&self, exprs: &Vec<Expr>) -> Result<Literal> {
        let mut new_string = String::new();

        for elem in exprs.iter() {
            match self.get_var_from_ident(elem) {
                Ok(val)     => new_string.push_str(&val.to_string()),
                Err(e)      => return Err(e),
            }
        }
        Ok(Literal::StringLiteral(new_string))
    }

    fn get_var_from_ident(&self, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::LitExpr(lit)           => Ok(lit.clone()),
            Expr::IdentExpr(ident)       => self.get_var(ident),
            Expr::BuilderExpr(..)        => self.gen_literal_form_builder(expr),
            Expr::ComplexLiteral(..)     => self.gen_literal_form_builder(expr),
            _                            => Err(Error::new(ErrorKind::Other, "unown variable in Ident err n#1"))
        }
    }

    fn memorytype_to_literal(&self, memtype: Option<&MemoryType>) -> Result<Literal> {
        if let Some(elem) = memtype {
            return Ok(Literal::StringLiteral(elem.value.to_string()));
        } else {
            return Err(Error::new(ErrorKind::Other, "Error in memory action"));
        }
    }

    // VARS ##################################################################################

    // MEMORY ------------------------------------------------------------------
    fn search_str(&self, name: &str, expr: &Expr) -> bool {
        match expr {
            Expr::IdentExpr(Ident(ident)) if ident == name  => true,
            _                                               => false
        }
    }

    fn memory_get(&self, name: &Expr, expr: &Expr) -> Option<&MemoryType> {
        match (name, expr) {
            (Expr::IdentExpr(Ident(ident)), Expr::LitExpr(Literal::StringLiteral(lit))) if ident == "past"    => self.memory.past.get(lit),
            (Expr::IdentExpr(Ident(ident)), Expr::LitExpr(Literal::StringLiteral(lit))) if ident == "memory"  => self.memory.current.get(lit),
            (_, Expr::LitExpr(Literal::StringLiteral(lit)))                                                   => self.memory.metadata.get(lit),
            _                                                                                                 => None,
        }
    }

    // fn memory_getvalue(&self, name: &Expr, expr: &Expr) {
    //     match name {
    //         Expr::IdentExpr(Ident(ident)) if ident == "past"    => {},
    //         Expr::IdentExpr(Ident(ident)) if ident == "memory"  => {},
    //         _                                                   => {},
    //     };
    // }

    // fn memory_all(&self, name: &Expr, expr: &Expr) {
    //     match name {
    //         Expr::IdentExpr(Ident(ident)) if ident == "past"    => {},
    //         Expr::IdentExpr(Ident(ident)) if ident == "memory"  => {},
    //         _                                                   => {},
    //     };
    // }

    // fn memory_allvalue(&self, name: &Expr, expr: &Expr) {
    //     match name {
    //         Expr::IdentExpr(Ident(ident)) if ident == "past"    => {},
    //         Expr::IdentExpr(Ident(ident)) if ident == "memory"  => {},
    //         _                                                   => {},
    //     };
    // }

    //TODO: RM UNWRAP
    fn memory_first(&self, name: &Expr, expr: &Expr) -> Option<&MemoryType> {
        match (name, expr) {
            (Expr::IdentExpr(Ident(ident)), Expr::LitExpr(Literal::StringLiteral(lit))) if ident == "past"    => self.memory.past.get_vec(lit).unwrap().last(),
            (Expr::IdentExpr(Ident(ident)), Expr::LitExpr(Literal::StringLiteral(lit))) if ident == "memory"  => self.memory.current.get_vec(lit).unwrap().last(),
            (_, Expr::LitExpr(Literal::StringLiteral(lit)))                                                   => self.memory.metadata.get_vec(lit).unwrap().last(),
            _                                                                                                 => None,
        }
    }

    // fn memory_firstvalue(&self, name: &Expr, expr: &Expr) {
    //     match name {
    //         Expr::IdentExpr(Ident(ident)) if ident == "past"    => {},
    //         Expr::IdentExpr(Ident(ident)) if ident == "memory"  => {},
    //         _                                                   => {},
    //     };
    // }

    //NOTE:Only work with Strings for now 
    fn get_memory_action(&self, name: &Expr, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::FunctionExpr(Ident(ident), exp) if ident == "get"         => {
                self.memorytype_to_literal(self.memory_get(name, exp))
            },
            // Expr::FunctionExpr(Ident(ident), exp) if ident == "getvalue"    => Err(Error::new(ErrorKind::Other, "Error in memory action")),
            // Expr::FunctionExpr(Ident(ident), exp) if ident == "all"         => Err(Error::new(ErrorKind::Other, "Error in memory action")),
            // Expr::FunctionExpr(Ident(ident), exp) if ident == "allvalue"    => Err(Error::new(ErrorKind::Other, "Error in memory action")),
            Expr::FunctionExpr(Ident(ident), exp) if ident == "first"       => {
                self.memorytype_to_literal(self.memory_first(name, exp))
            },
            // Expr::FunctionExpr(Ident(ident), exp) if ident == "firstvalue"  => Err(Error::new(ErrorKind::Other, "Error in memory action")),
            _                                                               => Err(Error::new(ErrorKind::Other, "Error in memory action")),
        }
    }

    fn gen_literal_form_builder(&self, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::BuilderExpr(elem , exp) if self.search_str("past", elem)      => self.get_memory_action(elem, exp),
            Expr::BuilderExpr(elem , exp) if self.search_str("memory", elem)    => self.get_memory_action(elem, exp),
            Expr::BuilderExpr(elem , exp) if self.search_str("metadata", elem)  => self.get_memory_action(elem, exp),
            Expr::ComplexLiteral(vec)                                           => self.get_string_from_complexstring(vec),
            Expr::IdentExpr(ident)                                              => self.get_var(ident),
            _                                                                   => Err(Error::new(ErrorKind::Other, "Error in Exprecion builder"))
        }
    }

    // MEMORY ------------------------------------------------------------------
    // TODO: return Result<&Literal>
    fn gen_literal_form_exp(&self, expr: &Expr) -> Result<Literal> {
        match expr {
            Expr::LitExpr(lit)      => Ok(lit.clone()),
            Expr::IdentExpr(ident)  => self.get_var(ident),
            _                       => Err(Error::new(ErrorKind::Other, "Expression must be a literal or an identifier"))
        }
    }

    // NOTE: see if IDENT CAN HAVE BUILDER_IDENT inside and LitExpr can have ComplexLiteral
    fn check_if_ident(&self, expr: &Expr) -> bool {
        match expr {
            Expr::LitExpr(..)         => true,
            Expr::IdentExpr(..)       => true,
            Expr::BuilderExpr(..)     => true,
            Expr::ComplexLiteral(..)  => true,
            _                         => false
        }
    }

    fn evaluate_condition(&self, infix: &Infix, expr1: &Expr, expr2: &Expr) -> Result<bool> {
        match (expr1, expr2) {
            (Expr::LitExpr(l1), Expr::LitExpr(l2))          => {
                Ok(self.cmp_lit(infix, l1, l2))
            },
            (exp1, exp2) if self.check_if_ident(exp1) && self.check_if_ident(exp2)    => {
                match (self.get_var_from_ident(exp1), self.get_var_from_ident(exp2)) {
                    (Ok(l1), Ok(l2))   => Ok(self.cmp_lit(infix, &l1, &l2)),
                    _                  => Err(Error::new(ErrorKind::Other, "error in evaluation between ident and ident"))
                }
            },
            (exp, Expr::LitExpr(l2)) if self.check_if_ident(exp)  => {
                match self.get_var_from_ident(exp) {
                    Ok(l1) => Ok(self.cmp_lit(infix, &l1, l2)),
                    _      => Err(Error::new(ErrorKind::Other, "error in evaluation between ident and lit"))
                }
            },
            (Expr::LitExpr(l1), exp) if self.check_if_ident(exp)   => {
                match self.get_var_from_ident(exp) {
                    Ok(l2) => Ok(self.cmp_lit(infix, l1, &l2)),
                    _      => Err(Error::new(ErrorKind::Other, "error in evaluation between lit and ident"))
                }
            },
            (Expr::InfixExpr(i1, ex1, ex2), Expr::InfixExpr(i2, exp1, exp2))      => {
                match (self.evaluate_condition(i1, ex1, ex2), self.evaluate_condition(i2, exp1, exp2)) {
                    (Ok(l1), Ok(l2))   => Ok(self.cmp_bool(infix, l1, l2)),
                    _                  => Err(Error::new(ErrorKind::Other, "error in evaluation between InfixExpr and InfixExpr"))
                }
            },
            (Expr::InfixExpr(i1, ex1, ex2), exp)                => {
                match (self.evaluate_condition(i1, ex1, ex2), self.gen_literal_form_exp(exp)) {
                    (Ok(e1), Ok(_)) => {
                        match infix {
                            Infix::And                  => Ok(e1),
                            Infix::Or                   => Ok(true),
                            _                           => Err(Error::new(ErrorKind::Other, "error need && or ||"))
                        }
                    },
                    _                  => Err(Error::new(ErrorKind::Other, "error in evaluation between InfixExpr and expr"))
                }
            },
            (exp, Expr::InfixExpr(i1, ex1, ex2))                => {
                match (self.gen_literal_form_exp(exp), self.evaluate_condition(i1, ex1, ex2)) {
                    (Ok(_), Ok(e2)) => {
                        match infix {
                            Infix::And                  => Ok(e2),
                            Infix::Or                   => Ok(true),
                            _                           => Err(Error::new(ErrorKind::Other, "error need && or ||"))
                        }
                    },
                    _                  => Err(Error::new(ErrorKind::Other, "error in evaluation between expr and InfixExpr"))
                }
            }
            (_, _)                                              => Err(Error::new(ErrorKind::Other, "error in evaluate_condition function")),
        }
    }

    // return Result<Expr, error>
    fn valid_condition(&self, expr: &Expr) -> bool {
        match expr {
            Expr::InfixExpr(inf, exp1, exp2)    => {
                match self.evaluate_condition(inf, exp1, exp2) {
                    Ok(rep) => rep,
                    Err(e)  =>{
                        println!("----------------------------------------- error {:?} ====================", e);
                        false
                    }
                }
            },
            Expr::LitExpr(_lit)                 => true,
            Expr::BuilderExpr(..)               => self.get_var_from_ident(expr).is_ok(), // error
            Expr::IdentExpr(ident)              => self.get_var(ident).is_ok(), // error
            _                                   => false, // return error
        }
    }

    fn add_to_message(&self, root: &mut RootInterface, action: MessageType) {
        match action {
            MessageType::Msg(msg)            => root.add_message(msg),
            MessageType::Msgs(msgs)          => root.messages.extend(msgs),
            MessageType::Assign{name, value} => root.add_to_memory(name, value),
            MessageType::Empty               => {},
        }
    }

    fn match_sub_block(&self, arg: &Expr, root: RootInterface) -> Result<RootInterface>{
        if let Expr::VecExpr(vec) = arg {
            match self.match_block(vec) {
                Ok(action)  => {
                    Ok(root + action)
                },
                Err(_err)   => return Err(Error::new(ErrorKind::Other, "error in if consequence "))
            }
        } else {
            return Err(Error::new(ErrorKind::Other, "error sub block arg must be of type  Expr::VecExpr"))
        }
    }

    // NOTE: TMP implementation of block for an action
    pub fn match_block(&self, actions: &[Expr]) -> Result<RootInterface> {
        // self.check_valid_step(actions);
        let mut root = RootInterface {memories: None, messages: vec![], next_flow: None, next_step: None};
        // event
        for action in actions {
            if root.next_step.is_some() {
                return Ok(root)
            }

            match action {
                Expr::Reserved { fun, arg } => {
                    match (fun, self.event) {
                        (Ident(ref name), None) if name == "ask"                    => {
                            return self.match_sub_block(arg, root);
                        },
                        (Ident(ref name), Some(..)) if name == "respond"            => {
                            return self.match_sub_block(arg, root);
                        },
                        (Ident(ref name), _) if name == "respond" || name == "ask"  => continue,
                        (_, _)                                                      => {
                            match self.match_reserved(fun, arg) {
                                Ok(action)  => self.add_to_message(&mut root, action),
                                Err(err)    => return Err(err)
                            }
                        }
                    };
                },
                Expr::IfExpr { cond, consequence } => {
                    if self.valid_condition(cond) { 
                        match self.match_block(consequence) {
                            Ok(action)  => root = root + action,
                            Err(_err)   => return Err(Error::new(ErrorKind::Other, "error in if consequence "))
                        }
                    } 
                    // else {
                    //     return Err(Error::new(ErrorKind::Other, "error in if condition, it does not reduce to a boolean expression "));
                    // }
                },
                Expr::Goto(Ident(ident))    => {
                    root.add_next_step(ident)
                },
                Expr::Remember(Ident(name), expr) if self.check_if_ident(expr) => {
                        if let Ok(Literal::StringLiteral(var)) = self.get_var_from_ident(expr) {
                            self.add_to_message(&mut root, remember(name.to_string(), var.to_string()));
                        } else {
                            return Err(Error::new(ErrorKind::Other, "Error Assign value must be valid"));
                        }
                    },
                _                           => return Err(Error::new(ErrorKind::Other, "Block must start with a reserved keyword")),
            };
        }
        Ok(root)
    }
}