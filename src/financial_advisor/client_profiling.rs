use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc, NaiveDate, Datelike};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// -------------------- Client Profile Structures --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskToleranceLevel {
    VeryConservative,
    Conservative,
    Moderate,
    Aggressive,
    VeryAggressive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeHorizon {
    VeryShort,    // < 1 year
    Short,        // 1-3 years
    Medium,       // 3-7 years
    Long,         // 7-15 years
    VeryLong,     // > 15 years
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalType {
    Retirement,
    HomePurchase,
    Education,
    EmergencyFund,
    MajorPurchase,
    DebtRepayment,
    Travel,
    StartBusiness,
    Charitable,
    Legacy,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalPriority {
    Essential,    // Must achieve
    Important,    // Would strongly prefer to achieve
    WantToHave,   // Would be nice, but can be delayed
    Aspirational, // Dream goals
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalStatus {
    NotStarted,
    InProgress,
    OnTrack,
    BehindSchedule,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialGoal {
    pub id: Uuid,
    pub name: String,
    pub goal_type: GoalType,
    pub description: String,
    pub target_amount: f64,
    pub current_amount: f64,
    pub target_date: DateTime<Utc>,
    pub priority: GoalPriority,
    pub time_horizon: TimeHorizon,
    pub status: GoalStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncomeType {
    Salary,
    SelfEmployment,
    Investment,
    Rental,
    Pension,
    SocialSecurity,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeSource {
    pub id: Uuid,
    pub name: String,
    pub income_type: IncomeType,
    pub amount: f64,
    pub frequency: String, // Monthly, Bi-weekly, etc.
    pub is_taxable: bool,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpenseCategory {
    Housing,
    Transportation,
    Food,
    Healthcare,
    Insurance,
    Debt,
    Entertainment,
    Clothing,
    Education,
    Savings,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: Uuid,
    pub name: String,
    pub category: ExpenseCategory,
    pub amount: f64,
    pub frequency: String,
    pub is_essential: bool,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Cash,
    Stocks,
    Bonds,
    RealEstate,
    Retirement,
    Business,
    Vehicle,
    PersonalProperty,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub asset_type: AssetType,
    pub value: f64,
    pub acquisition_date: Option<DateTime<Utc>>,
    pub acquisition_cost: Option<f64>,
    pub is_liquid: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiabilityType {
    Mortgage,
    StudentLoan,
    CarLoan,
    CreditCard,
    PersonalLoan,
    BusinessLoan,
    TaxLiability,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Liability {
    pub id: Uuid,
    pub name: String,
    pub liability_type: LiabilityType,
    pub original_amount: f64,
    pub current_balance: f64,
    pub interest_rate: f64,
    pub minimum_payment: f64,
    pub payment_frequency: String,
    pub origination_date: Option<DateTime<Utc>>,
    pub maturity_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsuranceType {
    Health,
    Life,
    Disability,
    LongTermCare,
    Auto,
    Homeowners,
    RentersInsurance,
    Umbrella,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsurancePolicy {
    pub id: Uuid,
    pub name: String,
    pub insurance_type: InsuranceType,
    pub provider: String,
    pub policy_number: String,
    pub coverage_amount: f64,
    pub premium: f64,
    pub premium_frequency: String,
    pub beneficiaries: Vec<String>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
}

/// Behavioral biases that can affect investment decisions
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum BehavioralBias {
    LossAversion,
    Overconfidence,
    MentalAccounting,
    Anchoring,
    HerdMentality,
    RecencyBias,
    ConfirmationBias,
    StatusQuoBias,
    EndowmentEffect,
    SelfServingBias,
    AvailabilityBias,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProfile {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub date_of_birth: DateTime<Utc>,
    pub tax_bracket: f64,
    pub state_of_residence: String,
    pub retirement_age: Option<u32>,
    pub risk_tolerance: RiskToleranceLevel,
    pub investment_experience: u32, // Years of investment experience
    pub financial_goals: Vec<FinancialGoal>,
    pub income_sources: Vec<IncomeSource>,
    pub expenses: Vec<Expense>,
    pub assets: Vec<Asset>,
    pub liabilities: Vec<Liability>,
    pub insurance_policies: Vec<InsurancePolicy>,
    pub behavioral_biases: HashSet<BehavioralBias>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct ClientProfileService {
    profiles: HashMap<Uuid, ClientProfile>,
}

impl ClientProfileService {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
        }
    }

    pub fn create_profile(&mut self, profile: ClientProfile) -> Uuid {
        let id = profile.id;
        self.profiles.insert(id, profile);
        id
    }

    pub fn get_profile(&self, id: &Uuid) -> Option<&ClientProfile> {
        self.profiles.get(id)
    }

    pub fn update_profile(&mut self, profile: ClientProfile) -> Result<(), String> {
        if !self.profiles.contains_key(&profile.id) {
            return Err(format!("Profile with ID {} not found", profile.id));
        }
        self.profiles.insert(profile.id, profile);
        Ok(())
    }

    pub fn delete_profile(&mut self, id: &Uuid) -> Result<(), String> {
        if !self.profiles.contains_key(id) {
            return Err(format!("Profile with ID {} not found", id));
        }
        self.profiles.remove(id);
        Ok(())
    }

    pub fn add_financial_goal(&mut self, profile_id: &Uuid, goal: FinancialGoal) -> Result<(), String> {
        if let Some(profile) = self.profiles.get_mut(profile_id) {
            profile.financial_goals.push(goal);
            profile.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Profile with ID {} not found", profile_id))
        }
    }

    pub fn calculate_net_worth(&self, profile_id: &Uuid) -> Result<f64, String> {
        if let Some(profile) = self.profiles.get(profile_id) {
            let total_assets: f64 = profile.assets.iter().map(|a| a.value).sum();
            let total_liabilities: f64 = profile.liabilities.iter().map(|l| l.current_balance).sum();
            Ok(total_assets - total_liabilities)
        } else {
            Err(format!("Profile with ID {} not found", profile_id))
        }
    }

    pub fn calculate_monthly_cash_flow(&self, profile_id: &Uuid) -> Result<f64, String> {
        if let Some(profile) = self.profiles.get(profile_id) {
            // This is a simplified calculation that should be enhanced in a real implementation
            let monthly_income: f64 = profile.income_sources.iter()
                .filter(|i| i.frequency == "Monthly")
                .map(|i| i.amount)
                .sum();
            
            let monthly_expenses: f64 = profile.expenses.iter()
                .filter(|e| e.frequency == "Monthly")
                .map(|e| e.amount)
                .sum();
            
            Ok(monthly_income - monthly_expenses)
        } else {
            Err(format!("Profile with ID {} not found", profile_id))
        }
    }

    /// Calculate time horizon based on target date
    fn calculate_time_horizon(&self, target_date: NaiveDate) -> TimeHorizon {
        let today = Utc::now().date_naive();
        
        // Calculate years between dates
        let years_to_goal = (target_date.year_ce().1 as i32 - today.year_ce().1 as i32) as f64;
        
        if years_to_goal < 2.0 {
            TimeHorizon::Short
        } else if years_to_goal < 5.0 {
            TimeHorizon::Medium
        } else if years_to_goal < 10.0 {
            TimeHorizon::Long
        } else {
            TimeHorizon::VeryLong
        }
    }
}
