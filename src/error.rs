use thiserror::Error;
use std::fmt;

/// Error type for the investment platform API
#[derive(Error, Debug)]
pub enum ApiError {
    /// Entity not found
    #[error("Entity not found: {entity_type} with ID {id}")]
    NotFound {
        /// Type of entity that was not found
        entity_type: String,
        /// ID of the entity that was not found
        id: String,
    },

    /// Validation error
    #[error("Validation error: {message}")]
    ValidationError {
        /// Validation error message
        message: String,
    },

    /// Invalid parameter
    #[error("Invalid parameter: {parameter} - {message}")]
    InvalidParameter {
        /// Name of the invalid parameter
        parameter: String,
        /// Error message
        message: String,
    },

    /// Internal server error
    #[error("Internal server error: {message}")]
    InternalError {
        /// Error message
        message: String,
    },

    /// Conflict error
    #[error("Conflict error: {message}")]
    Conflict {
        /// Error message
        message: String,
    },

    /// Unauthorized error
    #[error("Unauthorized: {message}")]
    Unauthorized {
        /// Error message
        message: String,
    },

    /// Forbidden error
    #[error("Forbidden: {message}")]
    Forbidden {
        /// Error message
        message: String,
    },

    /// Model Portfolio specific errors
    #[error("Model Portfolio error: {error_type} - {message}")]
    ModelPortfolioError {
        /// Type of model portfolio error
        error_type: ModelPortfolioErrorType,
        /// Error message
        message: String,
    },

    /// Household management errors
    #[error("Household error: {error_type} - {message}")]
    HouseholdError {
        /// Type of household error
        error_type: HouseholdErrorType,
        /// Error message
        message: String,
    },

    /// Tax optimization errors
    #[error("Tax optimization error: {error_type} - {message}")]
    TaxOptimizationError {
        /// Type of tax optimization error
        error_type: TaxOptimizationErrorType,
        /// Error message
        message: String,
    },

    /// Charitable giving errors
    #[error("Charitable giving error: {error_type} - {message}")]
    CharitableGivingError {
        /// Type of charitable giving error
        error_type: CharitableGivingErrorType,
        /// Error message
        message: String,
    },

    /// Data access errors
    #[error("Data access error: {message}")]
    DataAccessError {
        /// Error message
        message: String,
        /// Source of the error (e.g., "DynamoDB", "S3", etc.)
        source: Option<String>,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError {
        /// Error message
        message: String,
    },

    /// External service errors
    #[error("External service error: {service} - {message}")]
    ExternalServiceError {
        /// Name of the external service
        service: String,
        /// Error message
        message: String,
    },
}

/// Types of errors specific to model portfolios
#[derive(Debug, Clone, PartialEq)]
pub enum ModelPortfolioErrorType {
    /// Error when creating sleeves from a model
    SleeveCreationError,
    /// Error when a child model is not found
    ChildModelNotFound,
    /// Error when model weights don't sum to 1.0
    InvalidWeights,
    /// Error when a model has an invalid structure
    InvalidModelStructure,
    /// Error when a security is not found
    SecurityNotFound,
    /// Error when a model has duplicate securities
    DuplicateSecurities,
    /// Error when a model has invalid allocation
    InvalidAllocation,
}

/// Types of errors specific to household management
#[derive(Debug, Clone, PartialEq)]
pub enum HouseholdErrorType {
    /// Error when a member is not found
    MemberNotFound,
    /// Error when an account is not found
    AccountNotFound,
    /// Error when a financial goal is not found
    GoalNotFound,
    /// Error when a beneficiary is not found
    BeneficiaryNotFound,
    /// Error when a withdrawal plan is invalid
    InvalidWithdrawalPlan,
    /// Error when an estate plan is invalid
    InvalidEstatePlan,
    /// Error when a household has no members
    NoMembers,
    /// Error when a household has no accounts
    NoAccounts,
}

/// Types of errors specific to tax optimization
#[derive(Debug, Clone, PartialEq)]
pub enum TaxOptimizationErrorType {
    /// Error when tax-loss harvesting parameters are invalid
    InvalidTLHParameters,
    /// Error when wash sale detection fails
    WashSaleDetectionFailure,
    /// Error when asset location optimization fails
    AssetLocationOptimizationFailure,
    /// Error when tax-efficient withdrawal planning fails
    WithdrawalPlanningFailure,
    /// Error when tax impact calculation fails
    TaxImpactCalculationFailure,
}

/// Types of errors specific to charitable giving
#[derive(Debug, Clone, PartialEq)]
pub enum CharitableGivingErrorType {
    /// Error when a charity is not found
    CharityNotFound,
    /// Error when a charitable vehicle is not found
    CharitableVehicleNotFound,
    /// Error when a donation is not found
    DonationNotFound,
    /// Error when a donation strategy is invalid
    InvalidDonationStrategy,
    /// Error when a charitable tax impact calculation fails
    TaxImpactCalculationFailure,
}

impl fmt::Display for ModelPortfolioErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SleeveCreationError => write!(f, "SleeveCreationError"),
            Self::ChildModelNotFound => write!(f, "ChildModelNotFound"),
            Self::InvalidWeights => write!(f, "InvalidWeights"),
            Self::InvalidModelStructure => write!(f, "InvalidModelStructure"),
            Self::SecurityNotFound => write!(f, "SecurityNotFound"),
            Self::DuplicateSecurities => write!(f, "DuplicateSecurities"),
            Self::InvalidAllocation => write!(f, "InvalidAllocation"),
        }
    }
}

impl fmt::Display for HouseholdErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemberNotFound => write!(f, "MemberNotFound"),
            Self::AccountNotFound => write!(f, "AccountNotFound"),
            Self::GoalNotFound => write!(f, "GoalNotFound"),
            Self::BeneficiaryNotFound => write!(f, "BeneficiaryNotFound"),
            Self::InvalidWithdrawalPlan => write!(f, "InvalidWithdrawalPlan"),
            Self::InvalidEstatePlan => write!(f, "InvalidEstatePlan"),
            Self::NoMembers => write!(f, "NoMembers"),
            Self::NoAccounts => write!(f, "NoAccounts"),
        }
    }
}

impl fmt::Display for TaxOptimizationErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTLHParameters => write!(f, "InvalidTLHParameters"),
            Self::WashSaleDetectionFailure => write!(f, "WashSaleDetectionFailure"),
            Self::AssetLocationOptimizationFailure => write!(f, "AssetLocationOptimizationFailure"),
            Self::WithdrawalPlanningFailure => write!(f, "WithdrawalPlanningFailure"),
            Self::TaxImpactCalculationFailure => write!(f, "TaxImpactCalculationFailure"),
        }
    }
}

impl fmt::Display for CharitableGivingErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CharityNotFound => write!(f, "CharityNotFound"),
            Self::CharitableVehicleNotFound => write!(f, "CharitableVehicleNotFound"),
            Self::DonationNotFound => write!(f, "DonationNotFound"),
            Self::InvalidDonationStrategy => write!(f, "InvalidDonationStrategy"),
            Self::TaxImpactCalculationFailure => write!(f, "TaxImpactCalculationFailure"),
        }
    }
}

/// Result type for the investment platform API
pub type ApiResult<T> = Result<T, ApiError>;

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        ApiError::InternalError { message }
    }
}

impl From<&str> for ApiError {
    fn from(message: &str) -> Self {
        ApiError::InternalError {
            message: message.to_string(),
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        ApiError::InternalError {
            message: format!("IO error: {}", error),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::InternalError {
            message: format!("JSON error: {}", error),
        }
    }
}

/// Helper function to create a not found error
pub fn not_found(entity_type: &str, id: &str) -> ApiError {
    ApiError::NotFound {
        entity_type: entity_type.to_string(),
        id: id.to_string(),
    }
}

/// Helper function to create a validation error
pub fn validation_error(message: &str) -> ApiError {
    ApiError::ValidationError {
        message: message.to_string(),
    }
}

/// Helper function to create an invalid parameter error
pub fn invalid_parameter(parameter: &str, message: &str) -> ApiError {
    ApiError::InvalidParameter {
        parameter: parameter.to_string(),
        message: message.to_string(),
    }
} 