//! Typed failures of the batente algebra.
//!
//! Every variant here is a `Result::Err`, which is **mitigation**. The
//! properties batente claims as *unrepresentable* have no code path at all:
//! a choreography missing a motion is `E0063`, an over-demanding animation is
//! `E0277` via the capability ladder, and a face resolving a raw motion name
//! is an absent method. Nothing in this file is rounded up to that tier.

use thiserror::Error;

use crate::beat::MotionName;
use crate::capability::MotionClass;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum BatenteError {
    /// A motion referenced by a choreography or a beat does not exist.
    #[error("unknown motion `{motion}` in cadence `{cadence}`")]
    UnknownMotion { cadence: String, motion: MotionName },

    /// Beat references form a cycle. Reports the full path, because a bare
    /// "cycle detected" is not actionable.
    #[error("cyclic beat definition: {}", .path.join(" -> "))]
    Cycle { path: Vec<String> },

    /// A duration was negative or did not fit. Carries the offending value —
    /// a diagnostic that cannot say WHICH duration was wrong is not useful.
    #[error("duration {millis}ms is negative or out of range")]
    NegativeDuration { millis: i64 },

    /// A zero-length beat set to repeat forever. That is a spin, not an
    /// animation: it would burn a core producing no visible change.
    #[error("motion `{motion}` repeats forever with zero duration")]
    ZeroLengthForever { motion: MotionName },

    /// A cadence declared the same motion twice.
    #[error("duplicate motion `{motion}` in cadence `{cadence}`")]
    DuplicateMotion { cadence: String, motion: MotionName },

    /// A face was asked for a motion it does not bind.
    #[error("face `{face}` does not bind `{motion}`")]
    UnboundMotion { face: String, motion: String },

    /// A face bound a face-motion it did not declare in `consumes`.
    #[error("face `{face}` binds undeclared face-motion `{motion}`")]
    UndeclaredFaceMotion { face: String, motion: String },

    /// A face was handed an animation it cannot express.
    ///
    /// NOTE ON TIER. At a STATIC call site this is a compile error via
    /// `AtLeast<R>` and this variant is unreachable. It exists for the
    /// DYNAMIC path — resolving an authored spec whose face class is data
    /// rather than a type parameter. Both paths are real; only the static one
    /// is unrepresentable, and conflating them would be tier inflation.
    #[error("face `{face}` is {actual:?} but `{motion}` demands {required:?}")]
    InsufficientCapability {
        face: String,
        motion: String,
        actual: MotionClass,
        required: MotionClass,
    },
}

pub type Result<T> = core::result::Result<T, BatenteError>;
