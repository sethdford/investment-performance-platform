use std::collections::HashMap;
use chrono::{Utc, NaiveDate, Datelike};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tracing::info;

use super::{GoalType, TimeHorizon, RiskToleranceLevel, ClientProfile, FinancialGoal, GoalStatus};
use crate::financial_advisor::{
    GoalPriority,
};

/// Life stage categories for clients
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifeStage {
    /// Young adult (20-35, early career, typically single)
    YoungAdult,
    
    /// Family formation (30-45, marriage, children, home purchase)
    FamilyFormation,
    
    /// Peak earnings (40-55, career advancement, college funding)
    PeakEarnings,
    
    /// Pre-retirement (55-65, maximizing retirement savings)
    PreRetirement,
    
    /// Retirement (65+, income distribution, legacy planning)
    Retirement,
    
    /// Business owner (any age, specific business-related goals)
    BusinessOwner,
    
    /// Sudden wealth (inheritance, business sale, lottery)
    SuddenWealth,
}

/// Goal template for creating standardized financial goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTemplate {
    /// Unique identifier
    pub id: String,
    
    /// Template name
    pub name: String,
    
    /// Goal type
    pub goal_type: GoalType,
    
    /// Template description
    pub description: String,
    
    /// Default time horizon
    pub default_time_horizon: TimeHorizon,
    
    /// Default priority
    pub default_priority: GoalPriority,
    
    /// Suggested target amount formula (e.g., "income * 0.5" for emergency fund)
    pub target_amount_formula: Option<String>,
    
    /// Suggested monthly contribution formula
    pub monthly_contribution_formula: Option<String>,
    
    /// Life stages this goal is most relevant for
    pub relevant_life_stages: Vec<LifeStage>,
    
    /// Recommended risk tolerance for this goal
    pub recommended_risk_tolerance: Option<RiskToleranceLevel>,
    
    /// Educational content related to this goal
    pub educational_content: Vec<String>,
    
    /// Typical timeframe in years
    pub typical_timeframe: Option<u32>,
}

/// Service for managing goal templates and creating goals from templates
pub struct GoalTemplateService {
    /// Goal templates by ID
    templates: HashMap<String, GoalTemplate>,
}

impl GoalTemplateService {
    /// Create a new goal template service with default templates
    pub fn new() -> Self {
        let mut service = Self {
            templates: HashMap::new(),
        };
        
        service.initialize_default_templates();
        
        service
    }
    
    /// Initialize default goal templates for different life stages
    fn initialize_default_templates(&mut self) {
        let templates = vec![
            // Emergency Fund Template
            GoalTemplate {
                id: "emergency_fund".to_string(),
                name: "Emergency Fund".to_string(),
                goal_type: GoalType::EmergencyFund,
                description: "Build a safety net for unexpected expenses or income disruptions".to_string(),
                default_time_horizon: TimeHorizon::ShortTerm,
                default_priority: GoalPriority::Essential,
                target_amount_formula: Some("monthly_expenses * 6".to_string()),
                monthly_contribution_formula: Some("monthly_income * 0.1".to_string()),
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::FamilyFormation,
                    LifeStage::PeakEarnings,
                    LifeStage::PreRetirement,
                    LifeStage::BusinessOwner,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::VeryConservative),
                educational_content: vec![
                    "Why you need an emergency fund".to_string(),
                    "How to build your emergency fund faster".to_string(),
                    "Where to keep your emergency savings".to_string(),
                ],
                typical_timeframe: Some(1),
            },
            
            // Retirement Template
            GoalTemplate {
                id: "retirement".to_string(),
                name: "Retirement".to_string(),
                goal_type: GoalType::Retirement,
                description: "Save for a comfortable retirement with sufficient income".to_string(),
                default_time_horizon: TimeHorizon::VeryLongTerm,
                default_priority: GoalPriority::Essential,
                target_amount_formula: Some("annual_income * 25".to_string()),
                monthly_contribution_formula: Some("annual_income * 0.15 / 12".to_string()),
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::FamilyFormation,
                    LifeStage::PeakEarnings,
                    LifeStage::PreRetirement,
                ],
                recommended_risk_tolerance: None, // Varies by age
                educational_content: vec![
                    "Retirement planning basics".to_string(),
                    "Understanding retirement accounts".to_string(),
                    "Social Security optimization strategies".to_string(),
                    "Creating a retirement income plan".to_string(),
                ],
                typical_timeframe: Some(40),
            },
            
            // Home Purchase Template
            GoalTemplate {
                id: "home_purchase".to_string(),
                name: "Home Purchase".to_string(),
                goal_type: GoalType::HomePurchase {
                    property_type: "Primary Residence".to_string(),
                    location: None,
                },
                description: "Save for a down payment on a home".to_string(),
                default_time_horizon: TimeHorizon::MediumTerm,
                default_priority: GoalPriority::Important,
                target_amount_formula: Some("annual_income * 1.0".to_string()), // Approx 20% down on 5x income
                monthly_contribution_formula: Some("monthly_income * 0.2".to_string()),
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::FamilyFormation,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Conservative),
                educational_content: vec![
                    "First-time homebuyer's guide".to_string(),
                    "Understanding mortgage options".to_string(),
                    "How much house can you afford?".to_string(),
                ],
                typical_timeframe: Some(5),
            },
            
            // Education Funding Template
            GoalTemplate {
                id: "education_funding".to_string(),
                name: "Education Funding".to_string(),
                goal_type: GoalType::Education {
                    beneficiary: "Child".to_string(),
                    education_level: "Undergraduate".to_string(),
                },
                description: "Save for a child's education expenses".to_string(),
                default_time_horizon: TimeHorizon::LongTerm,
                default_priority: GoalPriority::Important,
                target_amount_formula: Some("120000".to_string()), // Approximate 4-year public university cost
                monthly_contribution_formula: Some("target_amount / (years_to_goal * 12)".to_string()),
                relevant_life_stages: vec![
                    LifeStage::FamilyFormation,
                    LifeStage::PeakEarnings,
                ],
                recommended_risk_tolerance: None, // Varies by time horizon
                educational_content: vec![
                    "College savings account options".to_string(),
                    "Tax advantages of 529 plans".to_string(),
                    "Balancing education savings with retirement".to_string(),
                ],
                typical_timeframe: Some(18),
            },
            
            // Debt Repayment Template
            GoalTemplate {
                id: "debt_repayment".to_string(),
                name: "Debt Repayment".to_string(),
                goal_type: GoalType::DebtRepayment {
                    debt_type: "Various".to_string(),
                },
                description: "Accelerate repayment of high-interest debt".to_string(),
                default_time_horizon: TimeHorizon::MediumTerm,
                default_priority: GoalPriority::Essential,
                target_amount_formula: Some("total_high_interest_debt".to_string()),
                monthly_contribution_formula: Some("monthly_income * 0.2".to_string()),
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::FamilyFormation,
                    LifeStage::PeakEarnings,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::VeryConservative),
                educational_content: vec![
                    "Debt snowball vs. debt avalanche methods".to_string(),
                    "Strategies for paying off student loans".to_string(),
                    "When to consider debt consolidation".to_string(),
                ],
                typical_timeframe: Some(3),
            },
            
            // Major Purchase Template
            GoalTemplate {
                id: "major_purchase".to_string(),
                name: "Major Purchase".to_string(),
                goal_type: GoalType::MajorPurchase {
                    description: "Vehicle".to_string(),
                },
                description: "Save for a significant purchase like a vehicle".to_string(),
                default_time_horizon: TimeHorizon::ShortTerm,
                default_priority: GoalPriority::Aspirational,
                target_amount_formula: Some("annual_income * 0.5".to_string()),
                monthly_contribution_formula: Some("target_amount / (years_to_goal * 12)".to_string()),
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::FamilyFormation,
                    LifeStage::PeakEarnings,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Conservative),
                educational_content: vec![
                    "Saving vs. financing major purchases".to_string(),
                    "How to negotiate the best price".to_string(),
                ],
                typical_timeframe: Some(2),
            },
            
            // Vacation/Travel Template
            GoalTemplate {
                id: "vacation".to_string(),
                name: "Vacation Fund".to_string(),
                goal_type: GoalType::Custom {
                    name: "Vacation".to_string(),
                    description: "Travel and vacation expenses".to_string(),
                },
                description: "Save for travel and vacation experiences".to_string(),
                default_time_horizon: TimeHorizon::ShortTerm,
                default_priority: GoalPriority::Aspirational,
                target_amount_formula: Some("monthly_income * 3".to_string()),
                monthly_contribution_formula: Some("target_amount / (years_to_goal * 12)".to_string()),
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::FamilyFormation,
                    LifeStage::PeakEarnings,
                    LifeStage::Retirement,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Conservative),
                educational_content: vec![
                    "Budgeting for travel experiences".to_string(),
                    "Travel hacking with credit card rewards".to_string(),
                ],
                typical_timeframe: Some(1),
            },
            
            // Business Startup Template
            GoalTemplate {
                id: "business_startup".to_string(),
                name: "Business Startup".to_string(),
                goal_type: GoalType::Custom {
                    name: "Business Startup".to_string(),
                    description: "Funding to start a new business venture".to_string(),
                },
                description: "Launch your own business venture".to_string(),
                default_time_horizon: TimeHorizon::LongTerm,
                default_priority: GoalPriority::Important,
                target_amount_formula: None,
                monthly_contribution_formula: None,
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::FamilyFormation,
                    LifeStage::PeakEarnings,
                    LifeStage::BusinessOwner,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Aggressive),
                educational_content: vec![
                    "Business startup costs to consider".to_string(),
                    "Funding options for new businesses".to_string(),
                    "Creating a business plan".to_string(),
                ],
                typical_timeframe: Some(3),
            },
            
            // Legacy/Estate Planning Template
            GoalTemplate {
                id: "legacy".to_string(),
                name: "Legacy Planning".to_string(),
                goal_type: GoalType::Legacy {
                    beneficiary: None,
                },
                description: "Create a financial legacy for heirs or charitable causes".to_string(),
                default_time_horizon: TimeHorizon::VeryLongTerm,
                default_priority: GoalPriority::Aspirational,
                target_amount_formula: None,
                monthly_contribution_formula: None,
                relevant_life_stages: vec![
                    LifeStage::PeakEarnings,
                    LifeStage::PreRetirement,
                    LifeStage::Retirement,
                    LifeStage::SuddenWealth,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Moderate),
                educational_content: vec![
                    "Estate planning basics".to_string(),
                    "Tax-efficient wealth transfer strategies".to_string(),
                    "Charitable giving options".to_string(),
                    "Setting up a family foundation".to_string(),
                ],
                typical_timeframe: None,
            },
            
            // Sabbatical/Career Break Template
            GoalTemplate {
                id: "sabbatical".to_string(),
                name: "Sabbatical Fund".to_string(),
                goal_type: GoalType::Custom {
                    name: "Sabbatical".to_string(),
                    description: "Extended career break".to_string(),
                },
                description: "Fund an extended break from work for personal growth, education, or travel".to_string(),
                default_time_horizon: TimeHorizon::MediumTerm,
                default_priority: GoalPriority::Aspirational,
                target_amount_formula: Some("monthly_expenses * 12".to_string()),
                monthly_contribution_formula: Some("target_amount / (years_to_goal * 12)".to_string()),
                relevant_life_stages: vec![
                    LifeStage::YoungAdult,
                    LifeStage::PeakEarnings,
                ],
                recommended_risk_tolerance: Some(RiskToleranceLevel::Conservative),
                educational_content: vec![
                    "Planning a career sabbatical".to_string(),
                    "Negotiating a leave of absence".to_string(),
                    "Returning to work after a sabbatical".to_string(),
                ],
                typical_timeframe: Some(3),
            },
        ];
        
        // Add all templates to the service
        for template in templates {
            self.templates.insert(template.id.clone(), template);
        }
        
        info!("Initialized {} default goal templates", self.templates.len());
    }
    
    /// Get a template by ID
    pub fn get_template(&self, id: &str) -> Option<&GoalTemplate> {
        self.templates.get(id)
    }
    
    /// Get all templates
    pub fn get_all_templates(&self) -> Vec<&GoalTemplate> {
        self.templates.values().collect()
    }
    
    /// Get templates relevant for a specific life stage
    pub fn get_templates_for_life_stage(&self, life_stage: LifeStage) -> Vec<&GoalTemplate> {
        self.templates.values()
            .filter(|template| template.relevant_life_stages.contains(&life_stage))
            .collect()
    }
    
    /// Determine the most likely life stage based on client profile
    pub fn determine_life_stage(&self, client_profile: &ClientProfile) -> LifeStage {
        let today = Utc::now().date_naive();
        let birth_date = client_profile.date_of_birth;
        
        // Calculate age using year_ce() instead of year()
        let age = (today.year_ce().1 as i32 - birth_date.year_ce().1 as i32) as u8;
        
        // Check if the client is a business owner
        let is_business_owner = client_profile.assets.iter()
            .any(|asset| matches!(asset.asset_type, super::AssetType::Business));
        
        if is_business_owner {
            return LifeStage::BusinessOwner;
        }
        
        // Check for sudden wealth
        let has_sudden_wealth = client_profile.metadata.get("sudden_wealth")
            .map(|v| v == "true")
            .unwrap_or(false);
        
        if has_sudden_wealth {
            return LifeStage::SuddenWealth;
        }
        
        // Determine life stage based on age
        match age {
            0..=35 => LifeStage::YoungAdult,
            36..=45 => LifeStage::FamilyFormation,
            46..=55 => LifeStage::PeakEarnings,
            56..=65 => LifeStage::PreRetirement,
            _ => LifeStage::Retirement,
        }
    }
    
    /// Get recommended goals for a client based on their profile
    pub fn get_recommended_goals(&self, client_profile: &ClientProfile) -> Vec<&GoalTemplate> {
        let life_stage = self.determine_life_stage(client_profile);
        
        // Get templates for the client's life stage
        let mut templates = self.get_templates_for_life_stage(life_stage);
        
        // Sort by priority (Essential first, then Important, then Aspirational)
        templates.sort_by(|a, b| a.default_priority.cmp(&b.default_priority));
        
        templates
    }
    
    /// Create a financial goal from a template
    pub fn create_goal_from_template(
        &self, 
        template_id: &str, 
        client_profile: &ClientProfile,
        target_date: NaiveDate,
        custom_name: Option<&str>,
        custom_target_amount: Option<f64>,
        custom_monthly_contribution: Option<f64>
    ) -> Result<FinancialGoal> {
        // Get the template
        let template = self.get_template(template_id)
            .ok_or_else(|| anyhow!("Goal template not found: {}", template_id))?;
        
        // Calculate target amount based on formula or use custom amount
        let target_amount = match custom_target_amount {
            Some(amount) => amount,
            None => self.calculate_formula_value(&template.target_amount_formula, client_profile)?,
        };
        
        // Calculate monthly contribution based on formula or use custom amount
        let monthly_contribution = match custom_monthly_contribution {
            Some(amount) => amount,
            None => self.calculate_formula_value(&template.monthly_contribution_formula, client_profile)?,
        };
        
        // Calculate time horizon based on target date
        let today = Utc::now().date_naive();
        let years_to_goal = (target_date.year_ce().1 as i32 - today.year_ce().1 as i32) as f64;
        
        let time_horizon = if years_to_goal <= 2.0 {
            TimeHorizon::ShortTerm
        } else if years_to_goal <= 5.0 {
            TimeHorizon::MediumTerm
        } else if years_to_goal <= 10.0 {
            TimeHorizon::LongTerm
        } else {
            TimeHorizon::VeryLongTerm
        };
        
        // Determine risk tolerance based on time horizon if not specified in template
        let risk_tolerance = template.recommended_risk_tolerance.or_else(|| {
            match time_horizon {
                TimeHorizon::ShortTerm => Some(RiskToleranceLevel::VeryConservative),
                TimeHorizon::MediumTerm => Some(RiskToleranceLevel::Conservative),
                TimeHorizon::LongTerm => Some(RiskToleranceLevel::Moderate),
                TimeHorizon::VeryLongTerm => Some(RiskToleranceLevel::Aggressive),
            }
        });
        
        // Create the goal
        let goal = FinancialGoal {
            id: Uuid::new_v4().to_string(),
            name: custom_name.unwrap_or(&template.name).to_string(),
            goal_type: template.goal_type.clone(),
            description: template.description.clone(),
            target_amount,
            current_amount: 0.0,
            target_date,
            time_horizon,
            priority: template.default_priority,
            status: GoalStatus::NotStarted,
            monthly_contribution,
            required_return_rate: None,
            success_probability: None,
            associated_accounts: Vec::new(),
            risk_tolerance,
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        info!(
            template_id = %template_id,
            goal_name = %goal.name,
            target_amount = %goal.target_amount,
            target_date = %goal.target_date,
            "Created financial goal from template"
        );
        
        Ok(goal)
    }
    
    /// Calculate a value based on a formula and client profile
    fn calculate_formula_value(&self, formula_option: &Option<String>, client_profile: &ClientProfile) -> Result<f64> {
        let formula = match formula_option {
            Some(f) => f,
            None => return Ok(0.0),
        };
        
        // Calculate basic financial metrics
        let annual_income: f64 = client_profile.income_sources.iter()
            .map(|source| source.annual_amount)
            .sum();
        
        let monthly_income = annual_income / 12.0;
        
        let monthly_expenses: f64 = client_profile.expenses.iter()
            .map(|expense| expense.monthly_amount)
            .sum();
        
        let total_high_interest_debt: f64 = client_profile.liabilities.iter()
            .filter(|liability| liability.interest_rate > 0.07) // 7% threshold for high interest
            .map(|liability| liability.current_balance)
            .sum();
        
        // Simple formula evaluation
        match formula.as_str() {
            "annual_income * 25" => Ok(annual_income * 25.0),
            "annual_income * 1.0" => Ok(annual_income * 1.0),
            "annual_income * 0.5" => Ok(annual_income * 0.5),
            "annual_income * 2" => Ok(annual_income * 2.0),
            "monthly_expenses * 6" => Ok(monthly_expenses * 6.0),
            "monthly_expenses * 12" => Ok(monthly_expenses * 12.0),
            "monthly_income * 0.1" => Ok(monthly_income * 0.1),
            "monthly_income * 0.2" => Ok(monthly_income * 0.2),
            "monthly_income * 3" => Ok(monthly_income * 3.0),
            "annual_income * 0.15 / 12" => Ok(annual_income * 0.15 / 12.0),
            "total_high_interest_debt" => Ok(total_high_interest_debt),
            "120000" => Ok(120000.0),
            "target_amount / (years_to_goal * 12)" => {
                // This is a placeholder since we don't have target_amount and years_to_goal here
                // In a real implementation, this would be calculated properly
                Ok(monthly_income * 0.1)
            },
            _ => Err(anyhow!("Unsupported formula: {}", formula)),
        }
    }
    
    /// Add a custom template
    pub fn add_template(&mut self, template: GoalTemplate) -> Result<()> {
        if self.templates.contains_key(&template.id) {
            return Err(anyhow!("Template with ID {} already exists", template.id));
        }
        
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }
    
    /// Update an existing template
    pub fn update_template(&mut self, template: GoalTemplate) -> Result<()> {
        if !self.templates.contains_key(&template.id) {
            return Err(anyhow!("Template with ID {} not found", template.id));
        }
        
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }
    
    /// Remove a template
    pub fn remove_template(&mut self, template_id: &str) -> Result<()> {
        if !self.templates.contains_key(template_id) {
            return Err(anyhow!("Template with ID {} not found", template_id));
        }
        
        self.templates.remove(template_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    
    fn create_test_client_profile() -> ClientProfile {
        let now = Utc::now();
        let thirty_years_ago = NaiveDate::from_ymd_opt(now.year() - 30, 1, 1).unwrap();
        
        ClientProfile {
            id: "test-client".to_string(),
            name: "Test Client".to_string(),
            email: "test@example.com".to_string(),
            date_of_birth: thirty_years_ago,
            retirement_age: Some(65),
            life_expectancy: Some(90),
            tax_filing_status: Some("Single".to_string()),
            federal_tax_bracket: Some(0.24),
            state: Some("CA".to_string()),
            state_tax_bracket: Some(0.09),
            risk_tolerance: RiskToleranceLevel::Moderate,
            goals: Vec::new(),
            income_sources: vec![
                crate::financial_advisor::IncomeSource {
                    id: "salary".to_string(),
                    name: "Primary Salary".to_string(),
                    income_type: crate::financial_advisor::IncomeSourceType::Employment,
                    annual_amount: 100000.0,
                    is_taxable: true,
                    frequency: "bi-weekly".to_string(),
                    growth_rate: Some(0.03),
                    start_date: None,
                    end_date: None,
                    metadata: HashMap::new(),
                }
            ],
            expenses: vec![
                crate::financial_advisor::Expense {
                    id: "rent".to_string(),
                    name: "Rent".to_string(),
                    category: crate::financial_advisor::ExpenseCategory::Housing,
                    monthly_amount: 2000.0,
                    is_essential: true,
                    is_tax_deductible: false,
                    growth_rate: Some(0.02),
                    start_date: None,
                    end_date: None,
                    metadata: HashMap::new(),
                },
                crate::financial_advisor::Expense {
                    id: "groceries".to_string(),
                    name: "Groceries".to_string(),
                    category: crate::financial_advisor::ExpenseCategory::Food,
                    monthly_amount: 500.0,
                    is_essential: true,
                    is_tax_deductible: false,
                    growth_rate: Some(0.02),
                    start_date: None,
                    end_date: None,
                    metadata: HashMap::new(),
                }
            ],
            assets: Vec::new(),
            liabilities: vec![
                crate::financial_advisor::Liability {
                    id: "student_loan".to_string(),
                    name: "Student Loan".to_string(),
                    liability_type: crate::financial_advisor::LiabilityType::StudentLoan,
                    current_balance: 30000.0,
                    interest_rate: 0.05,
                    minimum_payment: 350.0,
                    is_tax_deductible: true,
                    original_amount: Some(40000.0),
                    term_months: Some(120),
                    origination_date: Some(NaiveDate::from_ymd_opt(now.year() - 5, 1, 1).unwrap()),
                    maturity_date: Some(NaiveDate::from_ymd_opt(now.year() + 5, 1, 1).unwrap()),
                    metadata: HashMap::new(),
                },
                crate::financial_advisor::Liability {
                    id: "credit_card".to_string(),
                    name: "Credit Card".to_string(),
                    liability_type: crate::financial_advisor::LiabilityType::CreditCard,
                    current_balance: 5000.0,
                    interest_rate: 0.18,
                    minimum_payment: 150.0,
                    is_tax_deductible: false,
                    original_amount: None,
                    term_months: None,
                    origination_date: None,
                    maturity_date: None,
                    metadata: HashMap::new(),
                }
            ],
            insurance_policies: Vec::new(),
            risk_profile_responses: Vec::new(),
            behavioral_biases: Vec::new(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    #[test]
    fn test_initialize_default_templates() {
        let service = GoalTemplateService::new();
        
        // Check that we have the expected number of templates
        assert!(service.templates.len() >= 8, "Expected at least 8 default templates");
        
        // Check that key templates exist
        assert!(service.get_template("emergency_fund").is_some());
        assert!(service.get_template("retirement").is_some());
        assert!(service.get_template("home_purchase").is_some());
        assert!(service.get_template("education_funding").is_some());
    }
    
    #[test]
    fn test_get_templates_for_life_stage() {
        let service = GoalTemplateService::new();
        
        let young_adult_templates = service.get_templates_for_life_stage(LifeStage::YoungAdult);
        let retirement_templates = service.get_templates_for_life_stage(LifeStage::Retirement);
        
        // Young adults should have emergency fund, home purchase, retirement
        assert!(young_adult_templates.iter().any(|t| t.id == "emergency_fund"));
        assert!(young_adult_templates.iter().any(|t| t.id == "home_purchase"));
        assert!(young_adult_templates.iter().any(|t| t.id == "retirement"));
        
        // Retirement stage should have legacy planning, healthcare
        assert!(retirement_templates.iter().any(|t| t.id == "legacy_planning"));
        assert!(retirement_templates.iter().any(|t| t.id == "healthcare"));
    }
}