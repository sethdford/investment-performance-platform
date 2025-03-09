use chrono::NaiveDate;
use std::collections::HashMap;

// Charitable Giving Types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CharitableVehicleType {
    DonorAdvisedFund,
    PrivateFoundation,
    CharitableRemainder,
    CharitableLeadTrust,
    QualifiedCharitableDistribution,
}

#[derive(Debug, Clone)]
pub struct Charity {
    pub id: String,
    pub name: String,
    pub ein: Option<String>,
    pub mission: String,
    pub category: String,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub is_qualified_501c3: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CharitableVehicle {
    pub id: String,
    pub name: String,
    pub vehicle_type: CharitableVehicleType,
    pub balance: f64,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub account_id: Option<String>,
    pub market_value: f64,
    pub annual_contribution: f64,
    pub annual_distribution: f64,
    pub beneficiary_charities: Vec<(String, f64)>,
    pub creation_date: NaiveDate,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DonationType {
    Cash,
    Securities,
    InKind,
    QualifiedCharitableDistribution,
}

impl From<String> for DonationType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "cash" => DonationType::Cash,
            "securities" => DonationType::Securities,
            "in-kind" | "inkind" | "in kind" => DonationType::InKind,
            "qcd" | "qualified charitable distribution" => DonationType::QualifiedCharitableDistribution,
            _ => DonationType::Cash,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CharitableDonation {
    pub id: String,
    pub charity_id: String,
    pub amount: f64,
    pub donation_type: DonationType,
    pub donation_date: NaiveDate,
    pub vehicle_id: Option<String>,
    pub security_id: Option<String>,
    pub tax_year: i32,
    pub receipt_received: bool,
    pub notes: Option<String>,
    pub asset_type: String,
    pub cost_basis: Option<f64>,
    pub fair_market_value: f64,
}

#[derive(Debug, Clone)]
pub struct CharitableTaxImpact {
    pub donation_id: String,
    pub tax_year: i32,
    pub federal_deduction: f64,
    pub state_deduction: f64,
    pub estimated_tax_savings: f64,
}

#[derive(Debug, Clone)]
pub struct DonationStrategy {
    pub id: String,
    pub description: String,
    pub estimated_tax_savings: f64,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub struct CharitableGivingPlan {
    pub annual_target: f64,
    pub strategies: Vec<DonationStrategy>,
    pub tax_impact: f64,
}

#[derive(Debug, Clone)]
pub struct CharitableGivingReport {
    pub total_donations: f64,
    pub donations_by_charity: HashMap<String, f64>,
    pub tax_impact: f64,
    pub strategies: Vec<DonationStrategy>,
}

// Estate Planning Types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EstatePlanType {
    Will,
    RevocableTrust,
    IrrevocableTrust,
    PowerOfAttorney,
    HealthcareDirective,
    BeneficiaryDesignation,
    CharitableRemainder,
    QualifiedPersonalResidence,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BeneficiaryType {
    Primary,
    Contingent,
    Charity,
}

#[derive(Debug, Clone)]
pub enum DocumentStatus {
    Draft,
    Executed,
    NeedsUpdate,
    Expired,
}

#[derive(Debug, Clone)]
pub struct EstatePlan {
    pub id: String,
    pub plan_type: EstatePlanType,
    pub status: DocumentStatus,
    pub created_at: NaiveDate,
    pub last_reviewed: NaiveDate,
    pub attorney: Option<String>,
    pub location: String,
    pub notes: Option<String>,
    pub name: String,
    pub creation_date: NaiveDate,
    pub last_updated: NaiveDate,
    pub assets: Vec<String>,
    pub beneficiaries: Vec<Beneficiary>,
    pub executor: Option<String>,
    pub trustee: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Beneficiary {
    pub id: String,
    pub name: String,
    pub relationship: String,
    pub allocation_percentage: f64,
    pub contingent: bool,
    pub beneficiary_type: BeneficiaryType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaxJurisdiction {
    Federal,
    State(String),
}

#[derive(Debug, Clone)]
pub struct EstateTaxAnalysis {
    pub gross_estate_value: f64,
    pub deductions: f64,
    pub taxable_estate: f64,
    pub federal_exemption_used: f64,
    pub taxable_after_exemption: f64,
    pub estimated_taxes: HashMap<TaxJurisdiction, f64>,
    pub lifetime_exemption_used: f64,
    pub lifetime_exemption_remaining: f64,
    pub estimated_estate_tax: f64,
    pub total_estimated_tax: f64,
    pub effective_tax_rate: f64,
    pub tax_reduction_strategies: Vec<EstateTaxStrategy>,
}

#[derive(Debug, Clone)]
pub struct EstateTaxStrategy {
    pub id: String,
    pub description: String,
    pub estimated_tax_savings: f64,
    pub complexity: i32,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub struct EstateDistributionAnalysis {
    pub total_estate_value: f64,
    pub after_tax_value: f64,
    pub distributions_by_beneficiary: HashMap<String, f64>,
    pub after_tax_estate_value: f64,
    pub beneficiary_distributions: HashMap<String, f64>,
    pub charitable_distributions: f64,
    pub trust_distributions: HashMap<String, f64>,
    pub distribution_timeline: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub enum EstatePlanningRecommendationType {
    DocumentUpdate,
    TaxStrategy,
    BeneficiaryChange,
    TrustCreation,
    DocumentCreation,
    ProbateAvoidance,
    BeneficiaryReview,
    TaxReduction,
}

impl From<String> for EstatePlanningRecommendationType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Document Update" => EstatePlanningRecommendationType::DocumentUpdate,
            "Tax Strategy" | "Tax Reduction" => EstatePlanningRecommendationType::TaxStrategy,
            "Beneficiary Change" => EstatePlanningRecommendationType::BeneficiaryChange,
            "Trust Creation" => EstatePlanningRecommendationType::TrustCreation,
            "Document Creation" => EstatePlanningRecommendationType::DocumentCreation,
            "Probate Avoidance" => EstatePlanningRecommendationType::ProbateAvoidance,
            "Beneficiary Review" => EstatePlanningRecommendationType::BeneficiaryReview,
            _ => EstatePlanningRecommendationType::DocumentUpdate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EstatePlanningRecommendation {
    pub id: String,
    pub recommendation_type: EstatePlanningRecommendationType,
    pub description: String,
    pub estimated_benefit: Option<f64>,
    pub priority: i32,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct EstatePlanningReport {
    pub estate_tax_analysis: EstateTaxAnalysis,
    pub estate_plans: Vec<EstatePlan>,
    pub beneficiary_designations: HashMap<String, Vec<Beneficiary>>,
    pub tax_analysis: EstateTaxAnalysis,
    pub recommendations: Vec<EstatePlanningRecommendation>,
    pub household_id: String,
    pub household_name: String,
    pub total_estate_value: f64,
    pub distribution_analysis: EstateDistributionAnalysis,
    pub document_status: Vec<EstateDocumentStatus>,
}

// Financial Goals Types
#[derive(Debug, Clone)]
pub enum GoalType {
    Retirement,
    Education,
    HomePurchase,
    DebtPayoff,
    MajorPurchase,
    Emergency,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
}

impl From<GoalPriority> for i32 {
    fn from(priority: GoalPriority) -> Self {
        match priority {
            GoalPriority::Low => 1,
            GoalPriority::Medium => 2,
            GoalPriority::High => 3,
        }
    }
}

impl From<i32> for GoalPriority {
    fn from(value: i32) -> Self {
        match value {
            1 => GoalPriority::Low,
            2 => GoalPriority::Medium,
            _ => GoalPriority::High,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalStatus {
    Active,
    OnTrack,
    AtRisk,
    OffTrack,
    Achieved,
    Abandoned,
}

#[derive(Debug, Clone)]
pub struct GoalContribution {
    pub id: String,
    pub amount: f64,
    pub contribution_date: NaiveDate,
    pub source: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FinancialGoal {
    pub id: String,
    pub name: String,
    pub goal_type: GoalType,
    pub target_amount: f64,
    pub current_amount: f64,
    pub target_date: NaiveDate,
    pub status: GoalStatus,
    pub priority: i32,
    pub created_at: NaiveDate,
    pub updated_at: NaiveDate,
    pub contributions: Vec<GoalContribution>,
    pub linked_accounts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GoalProgress {
    pub goal: FinancialGoal,
    pub percent_complete: f64,
    pub monthly_contribution_needed: Option<f64>,
    pub on_track: bool,
    pub months_remaining: i32,
    pub recent_contributions: Vec<GoalContribution>,
    pub status: GoalStatus,
    pub projected_completion_date: Option<NaiveDate>,
    pub recommendations: Vec<GoalRecommendation>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
}

impl From<GoalPriority> for RecommendationPriority {
    fn from(priority: GoalPriority) -> Self {
        match priority {
            GoalPriority::Low => RecommendationPriority::Low,
            GoalPriority::Medium => RecommendationPriority::Medium,
            GoalPriority::High => RecommendationPriority::High,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GoalRecommendation {
    pub id: String,
    pub description: String,
    pub estimated_impact: f64,
    pub priority: RecommendationPriority,
}

#[derive(Debug, Clone)]
pub struct HouseholdGoalsReport {
    pub goals: Vec<GoalProgress>,
    pub recommendations: Vec<GoalRecommendation>,
    pub total_goal_amount: f64,
    pub total_current_amount: f64,
    pub overall_progress: f64,
    pub priority_goals_at_risk: Vec<GoalProgress>,
}

// Risk Analysis Types
#[derive(Debug, Clone)]
pub struct ConcentrationRisk {
    pub risk_type: ConcentrationRiskType,
    pub severity: RiskSeverity,
    pub percentage: f64,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConcentrationRiskType {
    SingleSecurity,
    AssetClass,
    Sector,
    Geography,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct HouseholdRiskAnalysis {
    pub portfolio_volatility: f64,
    pub value_at_risk: f64,
    pub conditional_var: f64,
    pub volatility: f64,
    pub value_at_risk_95: f64,
    pub conditional_var_95: f64,
    pub security_concentration: Vec<ConcentrationRisk>,
    pub asset_class_concentration: Vec<ConcentrationRisk>,
    pub sector_concentration: Vec<ConcentrationRisk>,
}

#[derive(Debug, Clone)]
pub struct RiskReductionRecommendation {
    pub description: String,
    pub estimated_return_impact: f64,
    pub priority: RecommendationPriority,
}

// Withdrawal Types
#[derive(Debug, Clone, PartialEq)]
pub enum WithdrawalReason {
    RetirementIncome,
    RMD,
    Education,
    MajorPurchase,
    Emergency,
    TaxEfficient,
    LastResort,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WithdrawalRecommendationType {
    AccountSequencing,
    TaxLotSelection,
    TaxLossHarvesting,
    RothConversion,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WithdrawalRecommendationPriority {
    Low,
    Medium,
    High,
}

impl From<WithdrawalRecommendationPriority> for i32 {
    fn from(priority: WithdrawalRecommendationPriority) -> Self {
        match priority {
            WithdrawalRecommendationPriority::Low => 1,
            WithdrawalRecommendationPriority::Medium => 2,
            WithdrawalRecommendationPriority::High => 3,
        }
    }
}

impl From<i32> for WithdrawalRecommendationPriority {
    fn from(value: i32) -> Self {
        match value {
            1 => WithdrawalRecommendationPriority::Low,
            2 => WithdrawalRecommendationPriority::Medium,
            _ => WithdrawalRecommendationPriority::High,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccountWithdrawal {
    pub account_id: String,
    pub amount: f64,
    pub tax_impact: f64,
    pub holdings_to_sell: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WithdrawalPlan {
    pub total_amount: f64,
    pub withdrawals: Vec<AccountWithdrawal>,
    pub total_tax_impact: f64,
    pub after_tax_amount: f64,
    pub estimated_tax_impact: f64,
    pub tax_efficiency_score: f64,
}

#[derive(Debug, Clone)]
pub struct WithdrawalRecommendation {
    pub id: String,
    pub description: String,
    pub estimated_tax_savings: f64,
    pub priority: i32,
}

// Asset Location Types
#[derive(Debug, Clone)]
pub struct AssetLocationRecommendation {
    pub security_id: String,
    pub source_account_id: String,
    pub target_account_id: String,
    pub amount: f64,
    pub tax_efficiency_score: f64,
    pub estimated_tax_savings: f64,
    pub market_value: f64,
    pub reason: String,
    pub priority: i32,
}

// Household Report Types
#[derive(Debug, Clone)]
pub struct HouseholdReport {
    pub cash_balance: f64,
    pub total_cash_balance: f64,
    pub account_summary: Vec<String>,
    pub tlh_efficiency_score: f64,
    pub risk_analysis: HouseholdRiskAnalysis,
}

/// Withdrawal timeframe
#[derive(Debug, Clone, PartialEq)]
pub enum WithdrawalTimeframe {
    /// Monthly withdrawals
    Monthly,
    /// Quarterly withdrawals
    Quarterly,
    /// Annual withdrawals
    Annual,
    /// One-time withdrawal
    OneTime,
}

/// Beneficiary designation for an account
#[derive(Debug, Clone)]
pub struct BeneficiaryDesignation {
    /// Account identifier
    pub account_id: String,
    /// Beneficiaries for this account
    pub beneficiaries: Vec<Beneficiary>,
    /// Last reviewed date
    pub last_reviewed: NaiveDate,
}

/// Estate document status
#[derive(Debug, Clone)]
pub struct EstateDocumentStatus {
    /// Document type
    pub document_type: String,
    /// Whether the document exists
    pub exists: bool,
    /// Last updated date
    pub last_updated: Option<NaiveDate>,
    /// Document location
    pub location: Option<String>,
    /// Notes
    pub notes: Option<String>,
} 