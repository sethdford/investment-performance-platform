// -------------------- Risk Profiling Questionnaire --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileQuestion {
    pub id: Uuid,
    pub text: String,
    pub category: String, // e.g., "Risk Capacity", "Risk Preference", "Behavioral"
    pub answer_options: Vec<RiskProfileAnswerOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileAnswerOption {
    pub id: Uuid,
    pub text: String,
    pub risk_score: i32, // Higher score indicates higher risk tolerance
    pub behavioral_bias: Option<BehavioralBias>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileQuestionnaire {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub questions: Vec<RiskProfileQuestion>,
    pub version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileResponse {
    pub question_id: Uuid,
    pub selected_option_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct RiskProfilingService {
    questionnaires: HashMap<Uuid, RiskProfileQuestionnaire>,
}

impl RiskProfilingService {
    pub fn new() -> Self {
        let mut service = Self {
            questionnaires: HashMap::new(),
        };
        service.initialize_default_questionnaire();
        service
    }

    fn initialize_default_questionnaire(&mut self) {
        let questionnaire = RiskProfileQuestionnaire {
            id: Uuid::new_v4(),
            name: "Standard Risk Profiling Questionnaire".to_string(),
            description: "Assesses risk tolerance based on behavioral finance principles".to_string(),
            questions: vec![
                // Example risk capacity questions
                RiskProfileQuestion {
                    id: Uuid::new_v4(),
                    text: "How many years until you plan to start withdrawing money from your investments?".to_string(),
                    category: "Risk Capacity".to_string(),
                    answer_options: vec![
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "Less than 1 year".to_string(),
                            risk_score: 1,
                            behavioral_bias: None,
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "1-3 years".to_string(),
                            risk_score: 2,
                            behavioral_bias: None,
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "3-7 years".to_string(),
                            risk_score: 3,
                            behavioral_bias: None,
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "7-15 years".to_string(),
                            risk_score: 4,
                            behavioral_bias: None,
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "More than 15 years".to_string(),
                            risk_score: 5,
                            behavioral_bias: None,
                        },
                    ],
                },
                // Example risk preference question
                RiskProfileQuestion {
                    id: Uuid::new_v4(),
                    text: "When the market declines significantly, what would you most likely do?".to_string(),
                    category: "Risk Preference".to_string(),
                    answer_options: vec![
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "Sell all investments and move to cash".to_string(),
                            risk_score: 1,
                            behavioral_bias: Some(BehavioralBias::LossAversion),
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "Sell some investments to reduce risk".to_string(),
                            risk_score: 2,
                            behavioral_bias: Some(BehavioralBias::Recency),
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "Hold steady and make no changes".to_string(),
                            risk_score: 3,
                            behavioral_bias: Some(BehavioralBias::StatusQuo),
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "Buy a small amount of additional investments".to_string(),
                            risk_score: 4,
                            behavioral_bias: None,
                        },
                        RiskProfileAnswerOption {
                            id: Uuid::new_v4(),
                            text: "Significantly increase investments to buy at lower prices".to_string(),
                            risk_score: 5,
                            behavioral_bias: Some(BehavioralBias::Overconfidence),
                        },
                    ],
                },
                // Add more questions as needed
            ],
            version: "1.0".to_string(),
            created_at: Utc::now(),
        };

        self.questionnaires.insert(questionnaire.id, questionnaire);
    }

    pub fn calculate_risk_tolerance(&self, responses: &[RiskProfileResponse]) -> RiskToleranceLevel {
        // Find the questionnaire (assuming only one exists)
        let questionnaire = self.questionnaires.values().next().unwrap();
        
        // Map to track question ID to its question object
        let questions_map: HashMap<Uuid, &RiskProfileQuestion> = questionnaire.questions
            .iter()
            .map(|q| (q.id, q))
            .collect();
        
        // Calculate total score
        let mut total_score = 0;
        let mut total_possible = 0;
        
        for response in responses {
            if let Some(question) = questions_map.get(&response.question_id) {
                if let Some(option) = question.answer_options.iter()
                    .find(|o| o.id == response.selected_option_id) {
                    total_score += option.risk_score;
                }
                
                // Assume max score for each question is the highest risk_score of its options
                total_possible += question.answer_options.iter()
                    .map(|o| o.risk_score)
                    .max()
                    .unwrap_or(0);
            }
        }
        
        // Calculate percentage of max score
        let percentage = if total_possible > 0 {
            (total_score as f64 / total_possible as f64) * 100.0
        } else {
            0.0
        };
        
        // Map percentage to risk tolerance level
        match percentage {
            p if p < 20.0 => RiskToleranceLevel::VeryConservative,
            p if p < 40.0 => RiskToleranceLevel::Conservative,
            p if p < 60.0 => RiskToleranceLevel::Moderate,
            p if p < 80.0 => RiskToleranceLevel::Aggressive,
            _ => RiskToleranceLevel::VeryAggressive,
        }
    }

    pub fn detect_behavioral_biases(&self, responses: &[RiskProfileResponse]) -> HashSet<BehavioralBias> {
        let questionnaire = self.questionnaires.values().next().unwrap();
        let questions_map: HashMap<Uuid, &RiskProfileQuestion> = questionnaire.questions
            .iter()
            .map(|q| (q.id, q))
            .collect();
        
        let mut detected_biases = HashSet::new();
        
        for response in responses {
            if let Some(question) = questions_map.get(&response.question_id) {
                if let Some(option) = question.answer_options.iter()
                    .find(|o| o.id == response.selected_option_id) {
                    if let Some(bias) = &option.behavioral_bias {
                        detected_biases.insert(bias.clone());
                    }
                }
            }
        }
        
        detected_biases
    }
}

// -------------------- Goal Template System --------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifeStage {
    EarlyCareer,
    MidCareer,
    PreRetirement,
    Retirement,
    LateRetirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTemplate {
    pub id: Uuid,
    pub name: String,
    pub goal_type: GoalType,
    pub description: String,
    pub suggested_priority: GoalPriority,
    pub typical_time_horizon: TimeHorizon,
    pub applicable_life_stages: Vec<LifeStage>,
    pub default_target_percentage: Option<f64>, // % of income or net worth
    pub suggested_milestones: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GoalTemplateService {
    templates: Vec<GoalTemplate>,
}

impl GoalTemplateService {
    pub fn new() -> Self {
        let mut service = Self {
            templates: Vec::new(),
        };
        service.initialize_default_templates();
        service
    }

    fn initialize_default_templates(&mut self) {
        self.templates = vec![
            // Emergency Fund
            GoalTemplate {
                id: Uuid::new_v4(),
                name: "Emergency Fund".to_string(),
                goal_type: GoalType::EmergencyFund,
                description: "Build a financial safety net to cover unexpected expenses or income loss".to_string(),
                suggested_priority: GoalPriority::Essential,
                typical_time_horizon: TimeHorizon::Short,
                applicable_life_stages: vec![
                    LifeStage::EarlyCareer,
                    LifeStage::MidCareer,
                    LifeStage::PreRetirement,
                ],
                default_target_percentage: Some(30.0), // 3-6 months of expenses
                suggested_milestones: vec![
                    "1 month of expenses".to_string(),
                    "3 months of expenses".to_string(),
                    "6 months of expenses".to_string(),
                ],
            },
            // Retirement
            GoalTemplate {
                id: Uuid::new_v4(),
                name: "Retirement Planning".to_string(),
                goal_type: GoalType::Retirement,
                description: "Save and invest for financial independence in retirement".to_string(),
                suggested_priority: GoalPriority::Essential,
                typical_time_horizon: TimeHorizon::VeryLong,
                applicable_life_stages: vec![
                    LifeStage::EarlyCareer,
                    LifeStage::MidCareer,
                    LifeStage::PreRetirement,
                ],
                default_target_percentage: Some(15.0), // 15% of income
                suggested_milestones: vec![
                    "Start contributing to retirement accounts".to_string(),
                    "Maximize employer match".to_string(),
                    "Reach 1x annual salary by age 30".to_string(),
                    "Reach 3x annual salary by age 40".to_string(),
                    "Reach 6x annual salary by age 50".to_string(),
                    "Reach 8x annual salary by age 60".to_string(),
                ],
            },
            // Add more templates as needed
        ];
    }

    pub fn create_goal_from_template(
        &self, 
        template_id: &Uuid, 
        target_amount: f64,
        target_date: DateTime<Utc>
    ) -> Option<FinancialGoal> {
        self.templates.iter()
            .find(|t| t.id == *template_id)
            .map(|template| {
                FinancialGoal {
                    id: Uuid::new_v4(),
                    name: template.name.clone(),
                    goal_type: template.goal_type.clone(),
                    description: template.description.clone(),
                    target_amount,
                    current_amount: 0.0,
                    target_date,
                    priority: template.suggested_priority.clone(),
                    time_horizon: template.typical_time_horizon.clone(),
                    status: GoalStatus::NotStarted,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }
            })
    }

    pub fn get_templates_for_life_stage(&self, life_stage: &LifeStage) -> Vec<&GoalTemplate> {
        self.templates.iter()
            .filter(|template| template.applicable_life_stages.contains(life_stage))
            .collect()
    }
}

// -------------------- Goal Prioritization --------------------

pub struct GoalPrioritizationService;

impl GoalPrioritizationService {
    pub fn new() -> Self {
        Self
    }
    
    pub fn prioritize_goals(&self, goals: &mut [FinancialGoal], profile: &ClientProfile) -> Vec<&FinancialGoal> {
        // First, sort goals by their explicit priority
        let mut prioritized_goals: Vec<&FinancialGoal> = goals.iter().collect();
        
        prioritized_goals.sort_by(|a, b| {
            // Sort by priority first (Essential is highest)
            let priority_order = b.priority.cmp(&a.priority);
            if priority_order != std::cmp::Ordering::Equal {
                return priority_order;
            }
            
            // If same priority, consider time horizon (shorter is more urgent)
            let time_horizon_a = match a.time_horizon {
                TimeHorizon::VeryShort => 1,
                TimeHorizon::Short => 2,
                TimeHorizon::Medium => 3,
                TimeHorizon::Long => 4,
                TimeHorizon::VeryLong => 5,
            };
            
            let time_horizon_b = match b.time_horizon {
                TimeHorizon::VeryShort => 1,
                TimeHorizon::Short => 2,
                TimeHorizon::Medium => 3,
                TimeHorizon::Long => 4,
                TimeHorizon::VeryLong => 5,
            };
            
            // For time horizon, lower number means higher priority
            time_horizon_a.cmp(&time_horizon_b)
        });
        
        prioritized_goals
    }
    
    // Helper method to recommend goals that may be missing from a client's profile
    pub fn recommend_missing_goals(&self, profile: &ClientProfile, template_service: &GoalTemplateService) -> Vec<GoalTemplate> {
        // Determine client's life stage based on age and other factors
        let life_stage = self.determine_life_stage(profile);
        
        // Get templates applicable to this life stage
        let applicable_templates = template_service.get_templates_for_life_stage(&life_stage);
        
        // Filter out templates for goals the client already has
        let existing_goal_types: HashSet<&GoalType> = profile.financial_goals
            .iter()
            .map(|g| &g.goal_type)
            .collect();
        
        applicable_templates
            .into_iter()
            .filter(|template| !existing_goal_types.contains(&template.goal_type))
            .cloned()
            .collect()
    }
    
    fn determine_life_stage(&self, profile: &ClientProfile) -> LifeStage {
        // Calculate age from date of birth
        let now = Utc::now();
        let dob = profile.date_of_birth;
        let age = now.year() - dob.year();
        
        // Determine retirement status
        let retirement_age = profile.retirement_age.unwrap_or(65);
        
        if age >= retirement_age + 15 {
            LifeStage::LateRetirement
        } else if age >= retirement_age {
            LifeStage::Retirement
        } else if age >= retirement_age - 10 {
            LifeStage::PreRetirement
        } else if age >= 35 {
            LifeStage::MidCareer
        } else {
            LifeStage::EarlyCareer
        }
    }
}

// -------------------- Financial Analysis Services --------------------

pub struct FinancialBalanceSheetAnalysis;

impl FinancialBalanceSheetAnalysis {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze_balance_sheet(&self, profile: &ClientProfile) -> HashMap<String, f64> {
        let mut analysis = HashMap::new();
        
        // Calculate total assets
        let total_assets: f64 = profile.assets.iter().map(|a| a.value).sum();
        analysis.insert("total_assets".to_string(), total_assets);
        
        // Calculate total liabilities
        let total_liabilities: f64 = profile.liabilities.iter().map(|l| l.current_balance).sum();
        analysis.insert("total_liabilities".to_string(), total_liabilities);
        
        // Calculate net worth
        let net_worth = total_assets - total_liabilities;
        analysis.insert("net_worth".to_string(), net_worth);
        
        // Calculate debt-to-asset ratio
        if total_assets > 0.0 {
            let debt_to_asset = total_liabilities / total_assets;
            analysis.insert("debt_to_asset_ratio".to_string(), debt_to_asset);
        }
        
        // Calculate asset allocation
        let mut asset_allocation = HashMap::new();
        for asset in &profile.assets {
            let asset_type = format!("{:?}", asset.asset_type);
            let current_value = asset_allocation.get(&asset_type).unwrap_or(&0.0);
            asset_allocation.insert(asset_type, current_value + asset.value);
        }
        
        for (asset_type, value) in asset_allocation {
            if total_assets > 0.0 {
                let percentage = (value / total_assets) * 100.0;
                analysis.insert(format!("asset_allocation_{}", asset_type), percentage);
            }
        }
        
        analysis
    }
    
    pub fn generate_balance_sheet_recommendations(&self, profile: &ClientProfile) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Analyze the balance sheet
        let analysis = self.analyze_balance_sheet(profile);
        
        // Check debt-to-asset ratio
        if let Some(debt_to_asset) = analysis.get("debt_to_asset_ratio") {
            if *debt_to_asset > 0.5 {
                recommendations.push("Your debt-to-asset ratio is high. Consider focusing on debt reduction.".to_string());
            }
        }
        
        // Check emergency fund
        let has_emergency_fund = profile.financial_goals.iter()
            .any(|g| matches!(g.goal_type, GoalType::EmergencyFund));
            
        if !has_emergency_fund {
            recommendations.push("You don't have an emergency fund goal. Consider establishing one with 3-6 months of expenses.".to_string());
        }
        
        // Check asset diversification
        let asset_types: HashSet<String> = profile.assets.iter()
            .map(|a| format!("{:?}", a.asset_type))
            .collect();
            
        if asset_types.len() < 3 {
            recommendations.push("Your assets appear to lack diversification. Consider expanding into different asset classes.".to_string());
        }
        
        recommendations
    }
}

pub struct CashFlowAnalysis;

impl CashFlowAnalysis {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze_cash_flow(&self, profile: &ClientProfile) -> HashMap<String, f64> {
        let mut analysis = HashMap::new();
        
        // Calculate total monthly income
        let monthly_income: f64 = profile.income_sources.iter()
            .map(|i| {
                match i.frequency.as_str() {
                    "Monthly" => i.amount,
                    "Bi-weekly" => i.amount * 26.0 / 12.0,
                    "Weekly" => i.amount * 52.0 / 12.0,
                    "Annually" => i.amount / 12.0,
                    "Semi-monthly" => i.amount * 24.0 / 12.0,
                    _ => i.amount, // Default to assuming monthly
                }
            })
            .sum();
        
        analysis.insert("monthly_income".to_string(), monthly_income);
        
        // Calculate total monthly expenses
        let monthly_expenses: f64 = profile.expenses.iter()
            .map(|e| {
                match e.frequency.as_str() {
                    "Monthly" => e.amount,
                    "Bi-weekly" => e.amount * 26.0 / 12.0,
                    "Weekly" => e.amount * 52.0 / 12.0,
                    "Annually" => e.amount / 12.0,
                    "Semi-monthly" => e.amount * 24.0 / 12.0,
                    _ => e.amount, // Default to assuming monthly
                }
            })
            .sum();
        
        analysis.insert("monthly_expenses".to_string(), monthly_expenses);
        
        // Calculate monthly net cash flow
        let net_cash_flow = monthly_income - monthly_expenses;
        analysis.insert("net_cash_flow".to_string(), net_cash_flow);
        
        // Calculate savings rate
        if monthly_income > 0.0 {
            let savings_rate = (net_cash_flow / monthly_income) * 100.0;
            analysis.insert("savings_rate".to_string(), savings_rate);
        }
        
        // Categorize expenses
        let mut expense_categories = HashMap::new();
        for expense in &profile.expenses {
            let category = format!("{:?}", expense.category);
            let monthly_amount = match expense.frequency.as_str() {
                "Monthly" => expense.amount,
                "Bi-weekly" => expense.amount * 26.0 / 12.0,
                "Weekly" => expense.amount * 52.0 / 12.0,
                "Annually" => expense.amount / 12.0,
                "Semi-monthly" => expense.amount * 24.0 / 12.0,
                _ => expense.amount, // Default to assuming monthly
            };
            
            let current_value = expense_categories.get(&category).unwrap_or(&0.0);
            expense_categories.insert(category, current_value + monthly_amount);
        }
        
        for (category, amount) in expense_categories {
            analysis.insert(format!("expense_category_{}", category), amount);
            
            // Calculate percentage of total expenses
            if monthly_expenses > 0.0 {
                let percentage = (amount / monthly_expenses) * 100.0;
                analysis.insert(format!("expense_percentage_{}", category), percentage);
            }
        }
        
        analysis
    }
    
    pub fn generate_cash_flow_recommendations(&self, profile: &ClientProfile) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Analyze cash flow
        let analysis = self.analyze_cash_flow(profile);
        
        // Check if expenses exceed income
        if let (Some(income), Some(expenses)) = (
            analysis.get("monthly_income"),
            analysis.get("monthly_expenses")
        ) {
            if expenses > income {
                recommendations.push("Your expenses exceed your income. Consider reducing expenses or finding ways to increase income.".to_string());
            }
        }
        
        // Check savings rate
        if let Some(savings_rate) = analysis.get("savings_rate") {
            if *savings_rate < 10.0 {
                recommendations.push("Your savings rate is below 10%. Aim to save at least 15-20% of your income for long-term goals.".to_string());
            }
        }
        
        // Check high expense categories
        for (key, value) in &analysis {
            if key.starts_with("expense_percentage_") {
                if *value > 30.0 && !key.contains("Housing") {
                    let category = key.replace("expense_percentage_", "");
                    recommendations.push(format!("Your {} expenses are relatively high at {}% of total expenses. Consider ways to reduce this category.", category, value.round()));
                }
            }
        }
        
        recommendations
    }
} 