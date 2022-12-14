use crate::check::Ty;
use crate::eval::{Error, Value};

use parsel::{
    ast::{
        Any, Brace, Ident, LeftAssoc, LitBool, LitInt, LitStr, Many, Maybe, Paren, Punctuated,
        RightAssoc, Token,
    },
    FromStr, Parse, ToTokens,
};

mod kw {
    parsel::custom_keyword!(pass);
    parsel::custom_keyword!(print);
    parsel::custom_keyword!(input);
    parsel::custom_keyword!(int);
    parsel::custom_keyword!(def);
    parsel::custom_keyword!(str);
    parsel::custom_keyword!(not);
    parsel::custom_keyword!(and);
    parsel::custom_keyword!(or);
    parsel::custom_keyword!(None);
    parsel::custom_keyword!(bool);
}

/// <prgm> ::= <blck>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Prgm {
    pub defns: Any<Defn>,
    pub main: Blck,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Defn {
    pub def: kw::def,
    pub name: Ident,
    pub params: Paren<Punctuated<TypedIdent, Token!(,)>>,
    pub ret: Maybe<ReturnType>,
    pub rule: Nest,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Nest {
    pub block: Brace<Blck>,
}

/// <blck> ::= <stmt> EOLN <stmt> OLN
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Blck {
    pub stmts: Many<Stmt>,
}

// <stmt> ::= <name> = <expn>
//          | pass
//          | print ( <expn> )
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Stmt {
    Decl {
        typed_ident: TypedIdent,
        equals: Token!(=),
        expn: Expn,
        end: Token!(;),
    },
    Assgn {
        ident: Ident,
        equals: Token!(=),
        expn: Expn,
        end: Token!(;),
    },
    Updt {
        ident: Ident,
        op: Updt,
        expn: Expn,
        end: Token!(;),
    },
    Pass(kw::pass, Token!(;)),
    Print(kw::print, Paren<Punctuated<Expn, Token!(,)>>, Token!(;)),
    If {
        if_: Token!(if),
        cond: Expn,
        #[parsel(recursive)]
        if_nest: Box<Nest>,

        else_: Token!(else),
        #[parsel(recursive)]
        else_nest: Box<Nest>,
    },
    While {
        while_: Token!(while),
        cond: Expn,
        #[parsel(recursive)]
        nest: Box<Nest>,
    },
    ReturnExpn {
        return_: Token!(return),
        expn: Expn,
        end: Token!(;),
    },
    Return {
        return_: Token!(return),
        end: Token!(;),
    },
    FuncCall {
        name: Ident,
        args: Paren<Punctuated<Expn, Token!(,)>>,
        end: Token!(;),
    },
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Updt {
    Plus(Token!(+=)),
    Minus(Token!(-=)),
}

// <expn> ::= <addn>
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Expn(
    pub  LeftAssoc<
        And,
        LeftAssoc<
            Or,
            RightAssoc<Comp, LeftAssoc<Add, LeftAssoc<Mult, LeftAssoc<Expt, UnExp<Not, Leaf>>>>>,
        >,
    >,
);

pub trait Binop {
    fn check(&self, lhs: Ty, rhs: Ty) -> Result<Ty, Error>;
    fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, Error>;
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Add {
    Plus(Token!(+)),
    Minus(Token!(-)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Mult {
    Times(Token!(*)),
    Div(Token!(/)),
    Mod(Token!(%)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Expt(Token!(^));

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Comp {
    Lt(Token!(<)),
    Leq(Token!(<=)),
    Eq(Token!(==)),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct And(kw::and);

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Or(kw::or);

impl Binop for Add {
    fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        Ok(match self {
            Self::Plus(_) => left + right,
            Self::Minus(_) => left - right,
        }
        .into())
    }

    fn check(&self, lhs: Ty, rhs: Ty) -> Result<Ty, Error> {
        lhs.expect_int()?;
        rhs.expect_int()?;
        Ok(Ty::Int)
    }
}

impl Binop for Mult {
    fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        Ok(match self {
            Self::Times(_) => left * right,
            Self::Div(_) => {
                if right == 0 {
                    return Err("cannot divide by zero");
                }
                left / right
            }
            Self::Mod(_) => {
                if right == 0 {
                    return Err("cannot mod by zero");
                }
                left % right
            }
        }
        .into())
    }

    fn check(&self, lhs: Ty, rhs: Ty) -> Result<Ty, Error> {
        lhs.expect_int()?;
        rhs.expect_int()?;
        Ok(Ty::Int)
    }
}

impl Binop for Expt {
    fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        if let Ok(exp) = right.try_into() {
            Ok(left.pow(exp).into())
        } else {
            Err("negative powers are not supported since there are no floats in dwislpy")
        }
    }

    fn check(&self, lhs: Ty, rhs: Ty) -> Result<Ty, Error> {
        lhs.expect_int()?;
        rhs.expect_int()?;
        Ok(Ty::Int)
    }
}

impl Binop for Comp {
    fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_int()?;
        let right = rhs.expect_int()?;
        Ok(match self {
            Self::Lt(_) => left < right,
            Self::Leq(_) => left <= right,
            Self::Eq(_) => left == right,
        }
        .into())
    }

    fn check(&self, lhs: Ty, rhs: Ty) -> Result<Ty, Error> {
        lhs.expect_int()?;
        rhs.expect_int()?;
        Ok(Ty::Bool)
    }
}

impl Binop for And {
    fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_bool()?;
        let right = rhs.expect_bool()?;
        Ok((left && right).into())
    }

    fn check(&self, lhs: Ty, rhs: Ty) -> Result<Ty, Error> {
        lhs.expect_bool()?;
        rhs.expect_bool()?;
        Ok(Ty::Bool)
    }
}

impl Binop for Or {
    fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, Error> {
        let left = lhs.expect_bool()?;
        let right = rhs.expect_bool()?;
        Ok((left || right).into())
    }

    fn check(&self, lhs: Ty, rhs: Ty) -> Result<Ty, Error> {
        lhs.expect_bool()?;
        rhs.expect_bool()?;
        Ok(Ty::Bool)
    }
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum UnExp<U: Unop, C: ToTokens> {
    Op(U, #[parsel(recursive)] Box<Self>),
    Child(C),
}

pub trait Unop {
    fn check(&self, on: Ty) -> Result<Ty, Error>;
    fn eval(&self, on: Value) -> Result<Value, Error>;
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct Not(kw::not);

impl Unop for Not {
    fn eval(&self, on: Value) -> Result<Value, Error> {
        let on = on.expect_bool()?;
        Ok((!on).into())
    }

    fn check(&self, on: Ty) -> Result<Ty, Error> {
        on.expect_bool()?;
        Ok(Ty::Bool)
    }
}

// <leaf> ::= <name> | <nmbr> | input ( <strg> ) | ( <expn> )
// <name> ::= x | count | _special | y0 | camelWalk | snake_slither | ...
// <nmbr> ::= 0 | 1 | 2 | 3 | ...
// <strg> ::= "hello" | "" | "say \"yo!\n\tyo.\"" | ...
#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Leaf {
    Inpt(kw::input, #[parsel(recursive)] Paren<Box<Expn>>),
    Int(kw::int, #[parsel(recursive)] Paren<Box<Expn>>),
    Str(kw::str, #[parsel(recursive)] Paren<Box<Expn>>),
    FuncCall {
        name: Ident,
        #[parsel(recursive)]
        args: Paren<Punctuated<Box<Expn>, Token!(,)>>,
    },
    Nmbr(LitInt),
    Strg(LitStr),
    Bool(LitBool),
    Name(Ident),
    Unit(kw::None),
    Expn(#[parsel(recursive)] Paren<Box<Expn>>),
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct ReturnType {
    arrow: Token!(->),
    ty: Type,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub struct TypedIdent {
    pub ident: Ident,
    colon: Token!(:),
    pub ty: Type,
}

#[derive(PartialEq, Eq, Debug, Parse, ToTokens, FromStr, Clone)]
pub enum Type {
    Int(kw::int),
    Bool(kw::bool),
    Str(kw::str),
    Unit(kw::None),
}
