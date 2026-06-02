use std::{
    convert::Infallible,
    fmt::Debug,
    ops::{ControlFlow, FromResidual, Try},
};
use bevy::prelude::*;

pub enum UnwrapResult<T> {
    Ok(T),
    Return,
    Warn(String),
    Error(String),
}

impl<T> FromResidual for UnwrapResult<T> {
    fn from_residual(unwrap_result: UnwrapResult<Infallible>) -> Self {
        match unwrap_result {
            UnwrapResult::Return => UnwrapResult::Return,
            UnwrapResult::Warn(warn) => UnwrapResult::Warn(warn),
            UnwrapResult::Error(error) => UnwrapResult::Error(error),
        }
    }
}

impl<T> Try for UnwrapResult<T> {
    type Output = T;
    type Residual = UnwrapResult<Infallible>;

    fn from_output(output: T) -> Self {
        UnwrapResult::Ok(output)
    }

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            UnwrapResult::Ok(value) => ControlFlow::Continue(value),
            UnwrapResult::Return => ControlFlow::Break(UnwrapResult::Return),
            UnwrapResult::Warn(warn) => ControlFlow::Break(UnwrapResult::Warn(warn)),
            UnwrapResult::Error(error) => ControlFlow::Break(UnwrapResult::Error(error)),
        }
    }
}

impl FromResidual<UnwrapResult<Infallible>> for () {
    #[track_caller]
    fn from_residual(unwrap_result: UnwrapResult<Infallible>) -> Self {
        match unwrap_result {
            UnwrapResult::Return => (),
            UnwrapResult::Warn(warn) => {
                let location = std::panic::Location::caller();
                warn!("(In {location}) {warn}");
            }
            UnwrapResult::Error(error) => {
                let location = std::panic::Location::caller();
                error!("(In {location}) {error}");
            }
        }
    }
}

pub trait ToUnwrapResult {
    type Inner;

    #[must_use]
    fn else_return(self) -> UnwrapResult<Self::Inner>;
    #[must_use]
    fn else_warn(self, warn: impl ToString) -> UnwrapResult<Self::Inner>;
    #[must_use]
    fn else_error(self, error: impl ToString) -> UnwrapResult<Self::Inner>;
}

impl<T> ToUnwrapResult for Option<T> {
    type Inner = T;

    fn else_return(self) -> UnwrapResult<Self::Inner> {
        match self {
            Some(value) => UnwrapResult::Ok(value),
            None => UnwrapResult::Return,
        }
    }
    fn else_warn(self, warn: impl ToString) -> UnwrapResult<Self::Inner> {
        match self {
            Some(value) => UnwrapResult::Ok(value),
            None => UnwrapResult::Warn(warn.to_string()),
        }
    }
    fn else_error(self, error: impl ToString) -> UnwrapResult<Self::Inner> {
        match self {
            Some(value) => UnwrapResult::Ok(value),
            None => UnwrapResult::Error(error.to_string()),
        }
    }
}

impl<T, E: Debug> ToUnwrapResult for Result<T, E> {
    type Inner = T;

    fn else_return(self) -> UnwrapResult<Self::Inner> {
        match self {
            Ok(value) => UnwrapResult::Ok(value),
            Err(_) => UnwrapResult::Return,
        }
    }
    fn else_warn(self, warn: impl ToString) -> UnwrapResult<Self::Inner> {
        match self {
            Ok(value) => UnwrapResult::Ok(value),
            Err(result_error) => {
                UnwrapResult::Warn(format!("{}\n{result_error:?}", warn.to_string()))
            }
        }
    }
    fn else_error(self, error: impl ToString) -> UnwrapResult<Self::Inner> {
        match self {
            Ok(value) => UnwrapResult::Ok(value),
            Err(result_error) => {
                UnwrapResult::Error(format!("{}\n{result_error:?}", error.to_string()))
            }
        }
    }
}

impl ToUnwrapResult for bool {
    type Inner = ();

    fn else_return(self) -> UnwrapResult<Self::Inner> {
        if self {
            UnwrapResult::Ok(())
        } else {
            UnwrapResult::Return
        }
    }
    fn else_warn(self, warn: impl ToString) -> UnwrapResult<Self::Inner> {
        if self {
            UnwrapResult::Ok(())
        } else {
            UnwrapResult::Warn(warn.to_string())
        }
    }
    fn else_error(self, error: impl ToString) -> UnwrapResult<Self::Inner> {
        if self {
            UnwrapResult::Ok(())
        } else {
            UnwrapResult::Error(error.to_string())
        }
    }
}
