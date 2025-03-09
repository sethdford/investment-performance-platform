use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use regex::Regex;

/// Financial query intent types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FinancialQueryIntent {
    // Portfolio and Investment Management
    /// Portfolio performance inquiry
    PortfolioPerformance,
    
    /// Asset allocation inquiry
    AssetAllocation,
    
    /// Investment recommendation request
    InvestmentRecommendation,
    
    /// Risk assessment inquiry
    RiskAssessment,
    
    /// Market information inquiry
    MarketInformation,
    
    /// ESG/Sustainable investing inquiry
    SustainableInvesting,
    
    /// Alternative investments inquiry
    AlternativeInvestments,
    
    /// International investing inquiry
    InternationalInvesting,
    
    /// Sector or thematic investing inquiry
    ThematicInvesting,
    
    // Retirement Planning
    /// Retirement planning inquiry
    RetirementPlanning,
    
    /// Retirement income strategies
    RetirementIncomeStrategies,
    
    /// Social Security optimization
    SocialSecurityOptimization,
    
    /// Medicare and health costs in retirement
    RetirementHealthcare,
    
    /// Required Minimum Distributions (RMDs)
    RequiredMinimumDistributions,
    
    /// Pension and defined benefit plans
    PensionPlanning,
    
    /// Early retirement planning
    EarlyRetirement,
    
    // Wealth Management
    /// Estate planning inquiry
    EstatePlanning,
    
    /// Trust services and planning
    TrustPlanning,
    
    /// Charitable giving inquiry
    CharitableGiving,
    
    /// Family wealth transfer
    WealthTransfer,
    
    /// Business succession planning
    BusinessSuccession,
    
    /// Private banking services
    PrivateBanking,
    
    /// Executive compensation strategies
    ExecutiveCompensation,
    
    // Tax Planning
    /// Tax optimization inquiry
    TaxOptimization,
    
    /// Tax-loss harvesting
    TaxLossHarvesting,
    
    /// Tax-efficient investing
    TaxEfficientInvesting,
    
    /// Tax planning for specific situations
    SpecializedTaxPlanning,
    
    // Financial Planning
    /// Goal progress inquiry
    GoalProgress,
    
    /// Cash flow analysis inquiry
    CashFlowAnalysis,
    
    /// Budget analysis inquiry
    BudgetAnalysis,
    
    /// Debt management inquiry
    DebtManagement,
    
    /// Insurance analysis inquiry
    InsuranceAnalysis,
    
    /// Financial education inquiry
    FinancialEducation,
    
    // Account Management
    /// Account information inquiry
    AccountInformation,
    
    /// Transaction history inquiry
    TransactionHistory,
    
    /// Account setup and maintenance
    AccountSetup,
    
    /// Account transfers and rollovers
    AccountTransfers,
    
    // Life Events
    /// Marriage financial planning
    MarriagePlanning,
    
    /// Divorce financial planning
    DivorcePlanning,
    
    /// New child or grandchild planning
    ChildPlanning,
    
    /// Education funding (college, private school)
    EducationPlanning,
    
    /// Home purchase or refinancing
    HomePurchase,
    
    /// Career change or job transition
    CareerTransition,
    
    /// Inheritance planning
    InheritancePlanning,
    
    /// Sudden wealth management
    SuddenWealth,
    
    /// Health crisis planning
    HealthCrisisPlanning,
    
    /// Long-term care planning
    LongTermCarePlanning,
    
    /// Relocation planning
    RelocationPlanning,
    
    // Life Stages
    /// Early career planning (20s-30s)
    EarlyCareerPlanning,
    
    /// Family formation planning (30s-40s)
    FamilyFormationPlanning,
    
    /// Peak earning years planning (40s-50s)
    PeakEarningsPlanning,
    
    /// Pre-retirement planning (50s-60s)
    PreRetirementPlanning,
    
    /// Retirement transition planning
    RetirementTransitionPlanning,
    
    /// Later retirement planning (70s+)
    LaterRetirementPlanning,
    
    // Behavioral Finance
    /// Risk tolerance reassessment
    RiskToleranceReassessment,
    
    /// Behavioral coaching during market volatility
    MarketVolatilityCoaching,
    
    /// Financial decision-making support
    DecisionMakingSupport,
    
    /// Financial wellness and mindfulness
    FinancialWellness,
    
    /// Greeting intent
    Greeting,
    
    /// Farewell intent
    Farewell,
    
    /// Help intent
    Help,
    
    /// Unknown intent
    Unknown,
}

impl std::fmt::Display for FinancialQueryIntent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinancialQueryIntent::PortfolioPerformance => write!(f, "PortfolioPerformance"),
            FinancialQueryIntent::AssetAllocation => write!(f, "AssetAllocation"),
            FinancialQueryIntent::InvestmentRecommendation => write!(f, "InvestmentRecommendation"),
            FinancialQueryIntent::RiskAssessment => write!(f, "RiskAssessment"),
            FinancialQueryIntent::MarketInformation => write!(f, "MarketInformation"),
            FinancialQueryIntent::SustainableInvesting => write!(f, "SustainableInvesting"),
            FinancialQueryIntent::AlternativeInvestments => write!(f, "AlternativeInvestments"),
            FinancialQueryIntent::InternationalInvesting => write!(f, "InternationalInvesting"),
            FinancialQueryIntent::ThematicInvesting => write!(f, "ThematicInvesting"),
            FinancialQueryIntent::RetirementPlanning => write!(f, "RetirementPlanning"),
            FinancialQueryIntent::RetirementIncomeStrategies => write!(f, "RetirementIncomeStrategies"),
            FinancialQueryIntent::SocialSecurityOptimization => write!(f, "SocialSecurityOptimization"),
            FinancialQueryIntent::RetirementHealthcare => write!(f, "RetirementHealthcare"),
            FinancialQueryIntent::RequiredMinimumDistributions => write!(f, "RequiredMinimumDistributions"),
            FinancialQueryIntent::PensionPlanning => write!(f, "PensionPlanning"),
            FinancialQueryIntent::EarlyRetirement => write!(f, "EarlyRetirement"),
            FinancialQueryIntent::EstatePlanning => write!(f, "EstatePlanning"),
            FinancialQueryIntent::TrustPlanning => write!(f, "TrustPlanning"),
            FinancialQueryIntent::CharitableGiving => write!(f, "CharitableGiving"),
            FinancialQueryIntent::WealthTransfer => write!(f, "WealthTransfer"),
            FinancialQueryIntent::BusinessSuccession => write!(f, "BusinessSuccession"),
            FinancialQueryIntent::PrivateBanking => write!(f, "PrivateBanking"),
            FinancialQueryIntent::ExecutiveCompensation => write!(f, "ExecutiveCompensation"),
            FinancialQueryIntent::TaxOptimization => write!(f, "TaxOptimization"),
            FinancialQueryIntent::TaxLossHarvesting => write!(f, "TaxLossHarvesting"),
            FinancialQueryIntent::TaxEfficientInvesting => write!(f, "TaxEfficientInvesting"),
            FinancialQueryIntent::SpecializedTaxPlanning => write!(f, "SpecializedTaxPlanning"),
            FinancialQueryIntent::GoalProgress => write!(f, "GoalProgress"),
            FinancialQueryIntent::CashFlowAnalysis => write!(f, "CashFlowAnalysis"),
            FinancialQueryIntent::BudgetAnalysis => write!(f, "BudgetAnalysis"),
            FinancialQueryIntent::DebtManagement => write!(f, "DebtManagement"),
            FinancialQueryIntent::InsuranceAnalysis => write!(f, "InsuranceAnalysis"),
            FinancialQueryIntent::FinancialEducation => write!(f, "FinancialEducation"),
            FinancialQueryIntent::AccountInformation => write!(f, "AccountInformation"),
            FinancialQueryIntent::TransactionHistory => write!(f, "TransactionHistory"),
            FinancialQueryIntent::AccountSetup => write!(f, "AccountSetup"),
            FinancialQueryIntent::AccountTransfers => write!(f, "AccountTransfers"),
            FinancialQueryIntent::MarriagePlanning => write!(f, "MarriagePlanning"),
            FinancialQueryIntent::DivorcePlanning => write!(f, "DivorcePlanning"),
            FinancialQueryIntent::ChildPlanning => write!(f, "ChildPlanning"),
            FinancialQueryIntent::EducationPlanning => write!(f, "EducationPlanning"),
            FinancialQueryIntent::HomePurchase => write!(f, "HomePurchase"),
            FinancialQueryIntent::CareerTransition => write!(f, "CareerTransition"),
            FinancialQueryIntent::InheritancePlanning => write!(f, "InheritancePlanning"),
            FinancialQueryIntent::SuddenWealth => write!(f, "SuddenWealth"),
            FinancialQueryIntent::HealthCrisisPlanning => write!(f, "HealthCrisisPlanning"),
            FinancialQueryIntent::LongTermCarePlanning => write!(f, "LongTermCarePlanning"),
            FinancialQueryIntent::RelocationPlanning => write!(f, "RelocationPlanning"),
            FinancialQueryIntent::EarlyCareerPlanning => write!(f, "EarlyCareerPlanning"),
            FinancialQueryIntent::FamilyFormationPlanning => write!(f, "FamilyFormationPlanning"),
            FinancialQueryIntent::PeakEarningsPlanning => write!(f, "PeakEarningsPlanning"),
            FinancialQueryIntent::PreRetirementPlanning => write!(f, "PreRetirementPlanning"),
            FinancialQueryIntent::RetirementTransitionPlanning => write!(f, "RetirementTransitionPlanning"),
            FinancialQueryIntent::LaterRetirementPlanning => write!(f, "LaterRetirementPlanning"),
            FinancialQueryIntent::RiskToleranceReassessment => write!(f, "RiskToleranceReassessment"),
            FinancialQueryIntent::MarketVolatilityCoaching => write!(f, "MarketVolatilityCoaching"),
            FinancialQueryIntent::DecisionMakingSupport => write!(f, "DecisionMakingSupport"),
            FinancialQueryIntent::FinancialWellness => write!(f, "FinancialWellness"),
            FinancialQueryIntent::Greeting => write!(f, "Greeting"),
            FinancialQueryIntent::Farewell => write!(f, "Farewell"),
            FinancialQueryIntent::Help => write!(f, "Help"),
            FinancialQueryIntent::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Entity types that can be extracted from financial queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    // Time and Date Entities
    /// Time period (e.g., "last month", "year to date", "since inception")
    TimePeriod,
    
    /// Date (e.g., "January 1st", "next year")
    Date,
    
    /// Age (e.g., "65 years old", "at age 70")
    Age,
    
    /// Life stage (e.g., "retirement", "early career", "family formation")
    LifeStage,
    
    // Account and Product Entities
    /// Account type (e.g., "401k", "IRA", "brokerage")
    AccountType,
    
    /// Account identifier (e.g., account number, nickname)
    AccountIdentifier,
    
    /// Financial product (e.g., "annuity", "CD", "money market")
    FinancialProduct,
    
    // Investment Entities
    /// Asset class (e.g., "stocks", "bonds", "real estate")
    AssetClass,
    
    /// Security (e.g., "AAPL", "S&P 500")
    Security,
    
    /// Sector (e.g., "technology", "healthcare", "energy")
    Sector,
    
    /// Investment style (e.g., "growth", "value", "income")
    InvestmentStyle,
    
    /// ESG preference (e.g., "environmental", "social", "governance")
    EsgPreference,
    
    // Financial Goal Entities
    /// Financial goal (e.g., "retirement", "college fund")
    Goal,
    
    /// Goal timeframe (e.g., "short-term", "long-term")
    GoalTimeframe,
    
    /// Goal priority (e.g., "essential", "important", "aspirational")
    GoalPriority,
    
    // Monetary Entities
    /// Amount (e.g., "$5000", "5%")
    Amount,
    
    /// Income (e.g., "salary", "pension", "social security")
    Income,
    
    /// Expense (e.g., "housing", "transportation")
    Expense,
    
    /// Asset (e.g., "home", "investment portfolio", "business")
    Asset,
    
    /// Debt (e.g., "mortgage", "student loan")
    Debt,
    
    // Risk and Performance Entities
    /// Risk level (e.g., "conservative", "aggressive")
    RiskLevel,
    
    /// Financial metric (e.g., "return", "volatility", "Sharpe ratio")
    Metric,
    
    /// Benchmark (e.g., "S&P 500", "Barclays Aggregate Bond Index")
    Benchmark,
    
    // Tax Entities
    /// Tax type (e.g., "capital gains", "income tax")
    TaxType,
    
    /// Tax strategy (e.g., "tax-loss harvesting", "Roth conversion")
    TaxStrategy,
    
    /// Tax status (e.g., "tax-exempt", "tax-deferred", "taxable")
    TaxStatus,
    
    // Insurance Entities
    /// Insurance type (e.g., "life insurance", "health insurance")
    Insurance,
    
    /// Insurance coverage (e.g., "term", "whole life", "universal life")
    InsuranceCoverage,
    
    // Life Event Entities
    /// Life event (e.g., "marriage", "divorce", "birth of child")
    LifeEvent,
    
    /// Family member (e.g., "spouse", "child", "parent")
    FamilyMember,
    
    // Location Entities
    /// Location (e.g., "New York", "international", "domestic")
    Location,
    
    // Behavioral Entities
    /// Behavioral bias (e.g., "loss aversion", "recency bias")
    BehavioralBias,
    
    /// Emotional state (e.g., "worried", "confident", "uncertain")
    EmotionalState,
    
    // Miscellaneous Entities
    /// Professional (e.g., "CPA", "estate attorney", "insurance agent")
    Professional,
    
    /// Document (e.g., "will", "trust", "beneficiary form")
    Document,
    
    /// Regulation (e.g., "SECURE Act", "TCJA", "Reg BI")
    Regulation,
}

/// Extracted entity from a financial query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity type
    pub entity_type: EntityType,
    
    /// Entity value
    pub value: String,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    
    /// Start position in the original text
    pub start_pos: usize,
    
    /// End position in the original text
    pub end_pos: usize,
}

/// Processed financial query with intent and entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedQuery {
    /// Original query text
    pub original_text: String,
    
    /// Recognized intent
    pub intent: FinancialQueryIntent,
    
    /// Intent confidence score (0.0 to 1.0)
    pub intent_confidence: f64,
    
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    
    /// Normalized query text (lowercase, removed punctuation)
    pub normalized_text: String,
}

/// Natural Language Processing service for financial queries
pub struct FinancialNlpService {
    /// Intent patterns
    intent_patterns: HashMap<FinancialQueryIntent, Vec<Regex>>,
    
    /// Entity extraction patterns
    entity_patterns: HashMap<EntityType, Vec<Regex>>,
}

impl FinancialNlpService {
    /// Create a new financial NLP service
    pub fn new() -> Self {
        let mut service = Self {
            intent_patterns: HashMap::new(),
            entity_patterns: HashMap::new(),
        };
        
        service.initialize_intent_patterns();
        service.initialize_entity_patterns();
        
        service
    }
    
    /// Initialize intent recognition patterns
    fn initialize_intent_patterns(&mut self) {
        // Portfolio performance patterns
        self.add_intent_patterns(
            FinancialQueryIntent::PortfolioPerformance,
            vec![
                r"(?i)how (is|has) my portfolio (doing|performing)",
                r"(?i)what (is|are) my (returns|gains|losses)",
                r"(?i)portfolio (performance|return|growth)",
                r"(?i)how (much|well) (has|have) my investments (grown|increased|decreased)",
                r"(?i)what (is|was) my (return|performance) (in|over|during|for)",
                r"(?i)how (did|are) my investments (doing|performing)",
                r"(?i)what's my portfolio return",
                r"(?i)how well have my investments done",
                r"(?i)what was my performance",
            ],
        );
        
        // Greeting patterns
        self.add_intent_patterns(
            FinancialQueryIntent::Greeting,
            vec![
                r"(?i)^(hi|hello|hey|greetings|good morning|good afternoon|good evening)(\s.*)?$",
                r"(?i)^(how are you|how's it going|how do you do|what's up)(\s.*)?$",
                r"(?i)^(nice to meet you|pleased to meet you)(\s.*)?$",
            ],
        );
        
        // Asset allocation patterns
        self.add_intent_patterns(
            FinancialQueryIntent::AssetAllocation,
            vec![
                r"(?i)what (is|are) my (current )?asset (allocation|mix)",
                r"(?i)how (is|are) my (portfolio|investments|assets) (allocated|distributed)",
                r"(?i)show me my (asset|portfolio) (allocation|breakdown|distribution)",
                r"(?i)(allocation|exposure) (to|in) (stocks|bonds|cash|equities)",
                r"(?i)am I (too|properly) (exposed|allocated|invested) (to|in)",
            ],
        );
        
        // Goal progress patterns
        self.add_intent_patterns(
            FinancialQueryIntent::GoalProgress,
            vec![
                r"(?i)how (am I|are we) (doing|progressing) (on|towards|with) my (goal|retirement|college)",
                r"(?i)(progress|status) (of|on|towards) my (financial )?(goal|plan|target)",
                r"(?i)will I (have enough|meet my goal|reach my target) for",
                r"(?i)am I on track (for|to|with) my",
                r"(?i)what (is|are) the (chances|probability|likelihood) (of|that) (meeting|achieving|reaching) my",
                r"(?i)am I on track for retirement",
                r"(?i)how (is|are) my (retirement|financial|savings) goal (doing|progressing)",
                r"(?i)retirement goal",
                r"(?i)goal (progress|tracking|status)",
            ],
        );
        
        // Tax optimization patterns
        self.add_intent_patterns(
            FinancialQueryIntent::TaxOptimization,
            vec![
                r"(?i)how can I (reduce|lower|minimize) (my )?taxes",
                r"(?i)tax (optimization|strategy|planning|efficiency)",
                r"(?i)(opportunities|strategies) for tax (loss harvesting|deferral|reduction)",
                r"(?i)should I (do a Roth conversion|harvest losses|realize gains)",
                r"(?i)what (is|are) my (tax implications|tax liability|tax situation)",
            ],
        );
        
        // Retirement planning patterns
        self.add_intent_patterns(
            FinancialQueryIntent::RetirementPlanning,
            vec![
                r"(?i)when can I retire",
                r"(?i)how much (do I need|should I save) (for|to) retire",
                r"(?i)retirement (planning|strategy|income|withdrawal)",
                r"(?i)will (my money|I) (last|have enough) (through|in|during) retirement",
                r"(?i)(social security|medicare|RMD|required minimum distribution)",
            ],
        );
        
        // Cash flow analysis patterns
        self.add_intent_patterns(
            FinancialQueryIntent::CashFlowAnalysis,
            vec![
                r"(?i)what (is|are) my (monthly|annual) (cash flow|income|expenses)",
                r"(?i)how much (am I|are we) (spending|saving|earning)",
                r"(?i)(analyze|review|show) my (spending|income|cash flow|budget)",
                r"(?i)where (is|does) my money (going|come from)",
                r"(?i)how much (can|should) I (save|spend|invest) (each|per) (month|year)",
            ],
        );
        
        // Investment recommendation patterns
        self.add_intent_patterns(
            FinancialQueryIntent::InvestmentRecommendation,
            vec![
                r"(?i)what should I invest in",
                r"(?i)(recommend|suggest) (investments|stocks|bonds|funds)",
                r"(?i)how should I invest (my|the) (money|funds|savings)",
                r"(?i)should I (buy|sell|hold) (this|these|my) (investment|stock|bond|fund)",
                r"(?i)what (changes|adjustments) should I make to my (portfolio|investments)",
                r"(?i)I want to invest in",
                r"(?i)invest in (a )?(mix of )?(stocks|bonds|equities|fixed income)",
                r"(?i)investing in (stocks|bonds|funds|ETFs)",
                r"(?i)investment (advice|recommendation|suggestion)",
                r"(?i)best (way|approach) to invest",
            ],
        );
        
        // Risk assessment patterns
        self.add_intent_patterns(
            FinancialQueryIntent::RiskAssessment,
            vec![
                r"(?i)how (risky|volatile) (is|are) my (portfolio|investments)",
                r"(?i)what (is|are) my (risk|volatility|downside)",
                r"(?i)am I taking (too much|enough|the right amount of) risk",
                r"(?i)(assess|evaluate|measure) my (risk tolerance|risk capacity)",
                r"(?i)how would my portfolio (perform|do) (in|during) a (market crash|recession|downturn)",
            ],
        );
        
        // Market information patterns
        self.add_intent_patterns(
            FinancialQueryIntent::MarketInformation,
            vec![
                r"(?i)how (is|are) the (market|markets) (doing|performing)",
                r"(?i)what (is|are) (happening|going on) (in|with) the (market|economy)",
                r"(?i)(tell|inform) me about (market|economic) (conditions|trends|outlook)",
                r"(?i)what (is|are) the (forecast|outlook|prediction) for (stocks|bonds|interest rates)",
                r"(?i)should I be (concerned|worried) about (inflation|recession|interest rates)",
            ],
        );
        
        // Financial education patterns
        self.add_intent_patterns(
            FinancialQueryIntent::FinancialEducation,
            vec![
                r"(?i)(explain|what is|tell me about) (dollar cost averaging|diversification|asset allocation)",
                r"(?i)how (does|do) (compound interest|tax-loss harvesting|Roth IRA) work",
                r"(?i)what (is|are) the (difference|benefits|drawbacks) (between|of)",
                r"(?i)help me understand (how|why|when)",
                r"(?i)can you (teach|educate) me (about|on)",
            ],
        );
        
        // Sustainable investing patterns
        self.add_intent_patterns(
            FinancialQueryIntent::SustainableInvesting,
            vec![
                r"(?i)(ESG|sustainable|responsible|ethical|impact) investing",
                r"(?i)how (can|do) I invest (according to|based on|aligned with) my (values|ethics|beliefs)",
                r"(?i)(environmental|social|governance) (factors|considerations|criteria) in my portfolio",
                r"(?i)(carbon|climate|green) (footprint|impact|investing)",
                r"(?i)(socially responsible|sustainable) (funds|ETFs|investments)",
                r"(?i)invest in ESG",
                r"(?i)ESG funds",
                r"(?i)sustainable (portfolio|investment|strategy)",
                r"(?i)ethical (investment|portfolio|fund)",
                r"(?i)invest (ethically|sustainably|responsibly)",
            ],
        );
        
        // Retirement income strategies patterns
        self.add_intent_patterns(
            FinancialQueryIntent::RetirementIncomeStrategies,
            vec![
                r"(?i)how (should|can|do) I (generate|create|plan for) (income|cash flow) in retirement",
                r"(?i)(withdrawal|distribution) (strategy|plan|rate) (for|in|during) retirement",
                r"(?i)(sustainable|safe) withdrawal rate",
                r"(?i)how (long|much) will my (money|savings|nest egg) last in retirement",
                r"(?i)(sequence of returns|longevity|inflation) risk in retirement",
            ],
        );
        
        // Social Security optimization patterns
        self.add_intent_patterns(
            FinancialQueryIntent::SocialSecurityOptimization,
            vec![
                r"(?i)when should I (claim|take|start) (my )?social security",
                r"(?i)(maximize|optimize|increase) (my )?social security (benefits|payments)",
                r"(?i)social security (strategy|planning|optimization)",
                r"(?i)how (much|many) (will|can) I (get|receive) from social security",
                r"(?i)(spousal|survivor|divorced) (benefits|strategy) (for|with) social security",
                r"(?i)social security benefits",
                r"(?i)claim (my )?social security",
                r"(?i)when (to|should I) (start|begin|take) (receiving|collecting) (my )?(social security|SS|SSI)",
                r"(?i)optimal (age|time) (for|to) (claim|take|start) social security",
            ],
        );
        
        // Estate planning patterns
        self.add_intent_patterns(
            FinancialQueryIntent::EstatePlanning,
            vec![
                r"(?i)(estate|legacy) planning",
                r"(?i)(will|trust|power of attorney|healthcare directive)",
                r"(?i)how (do|can|should) I (pass on|transfer|leave) (my|assets|wealth|money) to (my|family|heirs|beneficiaries)",
                r"(?i)(estate|inheritance|death) (tax|taxes)",
                r"(?i)(probate|executor|beneficiary) (process|designation|planning)",
            ],
        );
        
        // Charitable giving patterns
        self.add_intent_patterns(
            FinancialQueryIntent::CharitableGiving,
            vec![
                r"(?i)(charitable|philanthropy|donation) (giving|strategy|planning)",
                r"(?i)(donor advised fund|charitable trust|qualified charitable distribution)",
                r"(?i)how (can|do|should) I (give|donate) (to|money|assets) to charity",
                r"(?i)(tax benefits|deduction|advantages) of (charitable|giving|donating)",
                r"(?i)(legacy|impact) (giving|philanthropy)",
            ],
        );
        
        // Life event patterns - Marriage
        self.add_intent_patterns(
            FinancialQueryIntent::MarriagePlanning,
            vec![
                r"(?i)(getting married|marriage|wedding) (finances|planning|money)",
                r"(?i)(prenuptial|prenup) agreement",
                r"(?i)how (should|do|can) (we|I) (manage|handle|combine) (finances|money|accounts) (after|when) (getting married|marriage)",
                r"(?i)(joint|separate) (finances|accounts|money) (in|for|with) (marriage|spouse)",
                r"(?i)financial (implications|considerations|planning) (of|for) (marriage|getting married)",
                r"(?i)getting married next year",
                r"(?i)married (finances|money|financial planning)",
                r"(?i)(I'm|I am) getting married",
                r"(?i)marriage (and|&) (money|finances|financial planning)",
                r"(?i)combine (finances|money|accounts) (with|after) marriage",
            ],
        );
        
        // Life event patterns - Divorce
        self.add_intent_patterns(
            FinancialQueryIntent::DivorcePlanning,
            vec![
                r"(?i)(divorce|separation) (finances|planning|money)",
                r"(?i)how (will|does|can) divorce (affect|impact) (my|our) (finances|retirement|investments)",
                r"(?i)(dividing|splitting) (assets|property|accounts|retirement) in divorce",
                r"(?i)(alimony|child support|maintenance) (payments|planning|tax)",
                r"(?i)financial (recovery|planning|strategy) after divorce",
                r"(?i)getting divorced",
                r"(?i)I('m| am) getting divorced",
                r"(?i)divorce (next|this) (year|month)",
                r"(?i)adjust (my|financial|money) (plan|situation) (after|during|for) divorce",
                r"(?i)divorce financial (implications|considerations|planning)",
            ],
        );
        
        // Life event patterns - Education Planning
        self.add_intent_patterns(
            FinancialQueryIntent::EducationPlanning,
            vec![
                r"(?i)(college|education|school) (savings|planning|fund)",
                r"(?i)(529|education savings|college savings) (plan|account)",
                r"(?i)how (much|should) (to|I) save for (college|education|school)",
                r"(?i)(FAFSA|financial aid|student loans|scholarships)",
                r"(?i)(saving|paying) for (my|child's|grandchild's) (education|college|school)",
                r"(?i)save for (my|a) child('s)? college",
                r"(?i)college education",
                r"(?i)how much (should|do) I (need to|have to) save for (my|a) child('s)? (college|education)",
                r"(?i)child('s)? (college|education) (fund|savings|cost)",
            ],
        );
        
        // Life stage patterns - Early Career
        self.add_intent_patterns(
            FinancialQueryIntent::EarlyCareerPlanning,
            vec![
                r"(?i)(starting|early|beginning) (career|job|work) (finances|planning|money)",
                r"(?i)(student loan|education debt) (repayment|strategy|planning)",
                r"(?i)how (should|do|can) I (save|invest|budget) (in|during) (my|the) (20s|30s|early career)",
                r"(?i)(emergency fund|first home|first job) (savings|planning)",
                r"(?i)financial (priorities|goals|planning) for (young adults|new graduates|early career)",
            ],
        );
        
        // Life stage patterns - Family Formation
        self.add_intent_patterns(
            FinancialQueryIntent::FamilyFormationPlanning,
            vec![
                r"(?i)(starting a family|having children|family planning) (finances|costs|planning)",
                r"(?i)(childcare|daycare|education) (costs|expenses|planning)",
                r"(?i)financial (planning|implications|preparation) (for|of) (having|raising) (children|a family)",
                r"(?i)(life|health|disability) insurance (for|with) (family|children)",
                r"(?i)(saving|budgeting|planning) (with|for) (young|growing) (family|children)",
            ],
        );
        
        // Life stage patterns - Pre-Retirement
        self.add_intent_patterns(
            FinancialQueryIntent::PreRetirementPlanning,
            vec![
                r"(?i)(pre-retirement|retirement preparation|retirement transition) (planning|strategy)",
                r"(?i)(catch-up contributions|maximizing retirement savings|retirement readiness)",
                r"(?i)how (should|do|can) I (prepare|plan|save) (for|before) retirement (in|during) (my|the) (50s|60s)",
                r"(?i)(retirement income|withdrawal|distribution) (planning|strategy|projection)",
                r"(?i)(healthcare costs|long-term care|Medicare) (in|before|planning for) retirement",
            ],
        );
        
        // Behavioral finance patterns
        self.add_intent_patterns(
            FinancialQueryIntent::MarketVolatilityCoaching,
            vec![
                r"(?i)(worried|concerned|anxious|nervous) about (market|volatility|investments|losses)",
                r"(?i)should I (sell|get out|move to cash) (during|because of) (market volatility|market crash|downturn)",
                r"(?i)how (to|should I) (handle|manage|deal with) (market|investment) (volatility|uncertainty|stress)",
                r"(?i)(staying the course|long-term perspective|emotional investing)",
                r"(?i)(fear|greed|panic) (in|about|regarding) (investing|markets|portfolio)",
                r"(?i)worried about (the )?market volatility",
                r"(?i)market volatility",
                r"(?i)change (my|our) investments (due to|because of) (volatility|market conditions)",
                r"(?i)(scared|afraid|fearful) (of|about) (the )?(market|stock|investment) (crash|drop|decline)",
                r"(?i)should I (change|adjust|modify) my investments",
            ],
        );
        
        // Financial wellness patterns
        self.add_intent_patterns(
            FinancialQueryIntent::FinancialWellness,
            vec![
                r"(?i)(financial|money) (wellness|wellbeing|health|mindfulness)",
                r"(?i)(stress|anxiety|worry) (about|regarding|concerning) (money|finances|financial situation)",
                r"(?i)how (to|can I) (improve|enhance|better) my (relationship|mindset|attitude) (with|towards|about) money",
                r"(?i)(work-life balance|quality of life|life satisfaction) and (money|finances|wealth)",
                r"(?i)(values-based|purpose-driven|meaningful) (financial planning|money management|wealth)",
            ],
        );
        
        // Add more intent patterns for other intents as needed
    }
    
    /// Initialize entity extraction patterns
    fn initialize_entity_patterns(&mut self) {
        // Time period patterns
        self.add_entity_patterns(
            EntityType::TimePeriod,
            vec![
                r"(?i)(last|past|previous) (month|year|quarter|week)",
                r"(?i)(year to date|YTD)",
                r"(?i)(since inception|all time)",
                r"(?i)(trailing|rolling) (12|24|36|60) months",
                r"(?i)(1|3|5|10|20) (year|month)s?",
            ],
        );
        
        // Account type patterns
        self.add_entity_patterns(
            EntityType::AccountType,
            vec![
                r"(?i)(401k|403b|457|IRA|Roth IRA|Traditional IRA)",
                r"(?i)(brokerage|taxable|investment) account",
                r"(?i)(checking|savings) account",
                r"(?i)(HSA|health savings account)",
                r"(?i)(529|education savings|college savings) (plan|account)",
                r"(?i)(trust|UTMA|UGMA) account",
            ],
        );
        
        // Asset class patterns
        self.add_entity_patterns(
            EntityType::AssetClass,
            vec![
                r"(?i)(stocks|equities|shares)",
                r"(?i)(bonds|fixed income|treasuries)",
                r"(?i)(cash|money market|liquid assets)",
                r"(?i)(real estate|REIT|property)",
                r"(?i)(commodities|gold|silver|precious metals)",
                r"(?i)(alternatives|hedge funds|private equity)",
                r"(?i)(international|emerging markets|developed markets)",
                r"(?i)(large cap|mid cap|small cap)",
                r"(?i)(growth|value|blend)",
            ],
        );
        
        // Goal patterns
        self.add_entity_patterns(
            EntityType::Goal,
            vec![
                r"(?i)(retirement|retiring)",
                r"(?i)(college|education|school) (fund|savings|planning)",
                r"(?i)(home|house) (purchase|down payment|buying)",
                r"(?i)(emergency|rainy day) fund",
                r"(?i)(vacation|travel) (fund|savings|planning)",
                r"(?i)(wedding|marriage) (fund|savings|planning)",
                r"(?i)(car|vehicle) (purchase|buying)",
                r"(?i)(business|startup) (funding|capital|investment)",
                r"(?i)(legacy|inheritance|estate) (planning|fund)",
            ],
        );
        
        // Amount patterns
        self.add_entity_patterns(
            EntityType::Amount,
            vec![
                r"(?i)\$\d+(?:,\d+)*(?:\.\d+)?",
                r"(?i)\d+(?:,\d+)*(?:\.\d+)? (dollars|USD)",
                r"(?i)\d+(?:\.\d+)?%",
                r"(?i)\d+(?:,\d+)*(?:\.\d+)? (thousand|million|billion)",
                r"(?i)(a |one |two |three |four |five |ten |twenty |fifty |hundred |thousand |million |billion )(dollars|USD)",
            ],
        );
        
        // Date patterns
        self.add_entity_patterns(
            EntityType::Date,
            vec![
                r"(?i)(January|February|March|April|May|June|July|August|September|October|November|December) \d{1,2}(?:st|nd|rd|th)?,? \d{4}",
                r"(?i)\d{1,2}/\d{1,2}/\d{2,4}",
                r"(?i)(next|this|last) (year|month|week)",
                r"(?i)in \d{1,2} (years|months)",
                r"(?i)(20\d\d)",
                r"(?i)(today|tomorrow|yesterday)",
            ],
        );
        
        // Age patterns
        self.add_entity_patterns(
            EntityType::Age,
            vec![
                r"(?i)(\d{1,2}) (years old|year old)",
                r"(?i)age (\d{1,2})",
                r"(?i)at (\d{1,2})",
                r"(?i)(in|by|when I am|when I'm) (\d{1,2})",
            ],
        );
        
        // Life stage patterns
        self.add_entity_patterns(
            EntityType::LifeStage,
            vec![
                r"(?i)(early career|just starting|first job)",
                r"(?i)(family formation|having children|raising kids)",
                r"(?i)(peak earning years|mid-career|established career)",
                r"(?i)(pre-retirement|near retirement|approaching retirement)",
                r"(?i)(retirement|retired|post-career)",
                r"(?i)(later retirement|elderly|advanced age)",
            ],
        );
        
        // Risk level patterns
        self.add_entity_patterns(
            EntityType::RiskLevel,
            vec![
                r"(?i)(conservative|low risk|risk averse)",
                r"(?i)(moderate|balanced|medium risk)",
                r"(?i)(aggressive|high risk|growth oriented)",
                r"(?i)(very conservative|capital preservation)",
                r"(?i)(very aggressive|maximum growth)",
                r"(?i)risk (tolerance|capacity|profile|appetite)",
            ],
        );
        
        // Financial metric patterns
        self.add_entity_patterns(
            EntityType::Metric,
            vec![
                r"(?i)(return|performance|yield)",
                r"(?i)(volatility|standard deviation|variance)",
                r"(?i)(Sharpe ratio|risk-adjusted return)",
                r"(?i)(alpha|beta|R-squared)",
                r"(?i)(expense ratio|fee|cost)",
                r"(?i)(dividend|interest|income)",
                r"(?i)(capital gain|appreciation)",
                r"(?i)(drawdown|loss|decline)",
            ],
        );
        
        // Security patterns
        self.add_entity_patterns(
            EntityType::Security,
            vec![
                r"(?i)([A-Z]{1,5})",  // Stock ticker symbols
                r"(?i)(S&P 500|Dow Jones|NASDAQ|Russell 2000)",
                r"(?i)(index fund|ETF|mutual fund)",
                r"(?i)(Treasury|municipal|corporate) (bond|note|bill)",
                r"(?i)(stock|share|equity) of ([A-Z][a-zA-Z ]+)",
            ],
        );
        
        // Benchmark patterns
        self.add_entity_patterns(
            EntityType::Benchmark,
            vec![
                r"(?i)(S&P 500|Dow Jones|NASDAQ|Russell 2000)",
                r"(?i)(Barclays|Bloomberg) (Aggregate|Municipal|Treasury|Corporate) Bond Index",
                r"(?i)(MSCI|FTSE) (World|All Country|Emerging Markets|EAFE)",
                r"(?i)(benchmark|index|market average)",
                r"(?i)(60/40|traditional|balanced) portfolio",
            ],
        );
        
        // Tax type patterns
        self.add_entity_patterns(
            EntityType::TaxType,
            vec![
                r"(?i)(income|ordinary income) tax",
                r"(?i)(capital gains|long-term capital gains|short-term capital gains)",
                r"(?i)(dividend|qualified dividend|ordinary dividend)",
                r"(?i)(estate|inheritance|gift) tax",
                r"(?i)(property|real estate) tax",
                r"(?i)(FICA|Social Security|Medicare) tax",
                r"(?i)(state|local|federal) tax",
                r"(?i)(AMT|alternative minimum tax)",
                r"(?i)(NIIT|net investment income tax)",
            ],
        );
        
        // Tax strategy patterns
        self.add_entity_patterns(
            EntityType::TaxStrategy,
            vec![
                r"(?i)(tax-loss harvesting|harvest losses)",
                r"(?i)(Roth conversion|convert to Roth)",
                r"(?i)(tax-gain harvesting|harvest gains)",
                r"(?i)(asset location|tax location)",
                r"(?i)(charitable giving|qualified charitable distribution|QCD)",
                r"(?i)(tax deferral|defer taxes|tax-deferred)",
                r"(?i)(tax-exempt|municipal bonds)",
                r"(?i)(step-up in basis|basis step-up)",
                r"(?i)(tax bracket management|bracket planning)",
            ],
        );
        
        // Income patterns
        self.add_entity_patterns(
            EntityType::Income,
            vec![
                r"(?i)(salary|wages|compensation|pay)",
                r"(?i)(bonus|commission|overtime)",
                r"(?i)(dividend|interest|investment) income",
                r"(?i)(rental|real estate) income",
                r"(?i)(pension|retirement) income",
                r"(?i)(Social Security|SSI|SSDI) (benefits|income)",
                r"(?i)(business|self-employment) income",
                r"(?i)(alimony|child support|maintenance) (payments|income)",
                r"(?i)(royalty|licensing|intellectual property) income",
            ],
        );
        
        // Expense patterns
        self.add_entity_patterns(
            EntityType::Expense,
            vec![
                r"(?i)(housing|mortgage|rent)",
                r"(?i)(transportation|car|vehicle) (payment|expense|cost)",
                r"(?i)(food|groceries|dining)",
                r"(?i)(healthcare|medical|insurance) (expense|cost)",
                r"(?i)(utilities|electric|gas|water|internet)",
                r"(?i)(education|tuition|student loan) (payment|expense|cost)",
                r"(?i)(childcare|daycare) (expense|cost)",
                r"(?i)(entertainment|recreation|leisure) (expense|cost)",
                r"(?i)(debt|loan|credit card) (payment|expense|cost)",
                r"(?i)(tax|taxes) (payment|expense|cost)",
            ],
        );
        
        // Asset patterns
        self.add_entity_patterns(
            EntityType::Asset,
            vec![
                r"(?i)(home|house|primary residence|real estate)",
                r"(?i)(investment|brokerage|retirement) (account|portfolio)",
                r"(?i)(business|company|ownership stake)",
                r"(?i)(car|vehicle|boat)",
                r"(?i)(cash|savings|emergency fund)",
                r"(?i)(collectibles|art|jewelry)",
                r"(?i)(intellectual property|patents|copyrights)",
                r"(?i)(life insurance|cash value|permanent insurance)",
            ],
        );
        
        // Debt patterns
        self.add_entity_patterns(
            EntityType::Debt,
            vec![
                r"(?i)(mortgage|home loan|HELOC)",
                r"(?i)(student|education) loan",
                r"(?i)(auto|car|vehicle) loan",
                r"(?i)(credit card|revolving) debt",
                r"(?i)(personal|unsecured) loan",
                r"(?i)(business|commercial) loan",
                r"(?i)(medical|healthcare) debt",
                r"(?i)(tax|IRS) debt",
            ],
        );
        
        // Insurance patterns
        self.add_entity_patterns(
            EntityType::Insurance,
            vec![
                r"(?i)(life|term|whole|universal) insurance",
                r"(?i)(health|medical) insurance",
                r"(?i)(disability|long-term disability|short-term disability) insurance",
                r"(?i)(long-term care|LTC) insurance",
                r"(?i)(homeowners|renters) insurance",
                r"(?i)(auto|car|vehicle) insurance",
                r"(?i)(umbrella|liability) insurance",
                r"(?i)(business|professional) insurance",
            ],
        );
        
        // Life event patterns
        self.add_entity_patterns(
            EntityType::LifeEvent,
            vec![
                r"(?i)(marriage|wedding|getting married)",
                r"(?i)(divorce|separation|splitting up)",
                r"(?i)(birth|having|adopting) (a|of) (child|baby)",
                r"(?i)(death|passing|loss) of (spouse|partner|parent)",
                r"(?i)(buying|purchasing) (a|home|house)",
                r"(?i)(job|career) (change|transition|loss)",
                r"(?i)(retirement|retiring)",
                r"(?i)(inheritance|receiving money|windfall)",
                r"(?i)(health crisis|illness|disability)",
                r"(?i)(relocation|moving|relocating)",
            ],
        );
        
        // Behavioral bias patterns
        self.add_entity_patterns(
            EntityType::BehavioralBias,
            vec![
                r"(?i)(loss aversion|fear of loss)",
                r"(?i)(recency bias|recent events)",
                r"(?i)(overconfidence|too confident)",
                r"(?i)(herd mentality|following the crowd)",
                r"(?i)(confirmation bias|confirming beliefs)",
                r"(?i)(anchoring|fixating on numbers)",
                r"(?i)(mental accounting|separate buckets)",
                r"(?i)(status quo bias|resistance to change)",
                r"(?i)(home bias|favoring domestic)",
                r"(?i)(present bias|immediate gratification)",
            ],
        );
        
        // Emotional state patterns
        self.add_entity_patterns(
            EntityType::EmotionalState,
            vec![
                r"(?i)(worried|anxious|nervous|concerned)",
                r"(?i)(confident|optimistic|positive)",
                r"(?i)(uncertain|unsure|confused)",
                r"(?i)(fearful|scared|afraid)",
                r"(?i)(overwhelmed|stressed|pressured)",
                r"(?i)(regretful|remorseful|disappointed)",
                r"(?i)(hopeful|encouraged|reassured)",
                r"(?i)(frustrated|annoyed|irritated)",
            ],
        );
        
        // Add more entity patterns as needed
    }
    
    /// Add intent patterns to the service
    fn add_intent_patterns(&mut self, intent: FinancialQueryIntent, patterns: Vec<&str>) {
        let compiled_patterns: Vec<Regex> = patterns
            .into_iter()
            .map(|p| Regex::new(p).expect("Invalid regex pattern"))
            .collect();
        
        self.intent_patterns.insert(intent, compiled_patterns);
    }
    
    /// Add entity patterns to the service
    fn add_entity_patterns(&mut self, entity_type: EntityType, patterns: Vec<&str>) {
        let compiled_patterns: Vec<Regex> = patterns
            .into_iter()
            .map(|p| Regex::new(p).expect("Invalid regex pattern"))
            .collect();
        
        self.entity_patterns.insert(entity_type, compiled_patterns);
    }
    
    /// Process a financial query
    pub fn process_query(&self, query: &str) -> Result<ProcessedQuery> {
        let normalized_text = self.normalize_text(query);
        
        // Recognize intent
        let (intent, intent_confidence) = self.recognize_intent(&normalized_text);
        
        // Extract entities
        let entities = self.extract_entities(query);
        
        Ok(ProcessedQuery {
            original_text: query.to_string(),
            intent,
            intent_confidence,
            entities,
            normalized_text,
        })
    }
    
    /// Normalize text for processing
    fn normalize_text(&self, text: &str) -> String {
        text.to_lowercase()
    }
    
    /// Recognize the intent of a query
    fn recognize_intent(&self, text: &str) -> (FinancialQueryIntent, f64) {
        let mut best_intent = FinancialQueryIntent::Unknown;
        let mut best_confidence = 0.0;
        
        for (intent, patterns) in &self.intent_patterns {
            for pattern in patterns {
                if pattern.is_match(text) {
                    // In a real implementation, we would use a more sophisticated
                    // confidence calculation based on multiple matching patterns,
                    // pattern specificity, and possibly machine learning models.
                    // For now, we'll use a simple approach.
                    let confidence = 0.8;  // Fixed confidence for regex matches
                    
                    if confidence > best_confidence {
                        best_intent = intent.clone();
                        best_confidence = confidence;
                    }
                }
            }
        }
        
        (best_intent, best_confidence)
    }
    
    /// Extract entities from a query
    fn extract_entities(&self, text: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();
        
        for (entity_type, patterns) in &self.entity_patterns {
            for pattern in patterns {
                for capture in pattern.captures_iter(text) {
                    if let Some(m) = capture.get(0) {
                        entities.push(ExtractedEntity {
                            entity_type: entity_type.clone(),
                            value: m.as_str().to_string(),
                            confidence: 0.8,  // Fixed confidence for regex matches
                            start_pos: m.start(),
                            end_pos: m.end(),
                        });
                    }
                }
            }
        }
        
        entities
    }
    
    /// Get response for a processed query
    pub fn generate_response(&self, processed_query: &ProcessedQuery) -> String {
        // In a real implementation, this would use a more sophisticated
        // response generation system, possibly with templates, a language model,
        // or a combination of approaches. For now, we'll use a simple approach.
        
        match processed_query.intent {
            FinancialQueryIntent::Greeting => {
                "Hello! I'm your financial advisor assistant. How can I help you with your financial planning today?".to_string()
            },
            FinancialQueryIntent::PortfolioPerformance => {
                "Your portfolio has performed well recently. Would you like to see the detailed performance breakdown?".to_string()
            },
            FinancialQueryIntent::AssetAllocation => {
                "Your current asset allocation is 60% stocks, 30% bonds, and 10% cash equivalents. Would you like to make any adjustments?".to_string()
            },
            FinancialQueryIntent::GoalProgress => {
                "You're making good progress toward your goals. Your retirement goal is 75% funded, and your emergency fund is fully funded.".to_string()
            },
            FinancialQueryIntent::TaxOptimization => {
                "I've identified several tax optimization opportunities for you, including tax-loss harvesting and Roth conversion strategies.".to_string()
            },
            FinancialQueryIntent::RetirementPlanning => {
                "Based on your current savings rate and portfolio, you're on track to retire at age 67 with your desired income.".to_string()
            },
            FinancialQueryIntent::CashFlowAnalysis => {
                "Your monthly cash flow shows $5,000 in income and $4,200 in expenses, giving you $800 to save or invest each month.".to_string()
            },
            FinancialQueryIntent::InvestmentRecommendation => {
                "Based on your risk profile and goals, I recommend increasing your allocation to international equities and adding some exposure to REITs.".to_string()
            },
            FinancialQueryIntent::RiskAssessment => {
                "Your portfolio has a moderate risk level with an expected volatility of 12% annually. This aligns with your stated risk tolerance.".to_string()
            },
            FinancialQueryIntent::MarketInformation => {
                "The markets have been volatile recently due to inflation concerns and central bank policies. However, economic fundamentals remain strong.".to_string()
            },
            FinancialQueryIntent::FinancialEducation => {
                "I'd be happy to explain that concept. Would you like a basic overview or a more detailed explanation?".to_string()
            },
            FinancialQueryIntent::Unknown => {
                // Improved response for unknown intents or personal questions
                if processed_query.original_text.to_lowercase().contains("my name") {
                    "I don't have access to your personal information like your name. I'm designed to help with financial planning and investment questions. How can I assist you with your financial needs today?".to_string()
                } else {
                    "I'm not sure I understand your question. I'm designed to help with financial planning and investment questions. Could you please rephrase your question or ask something about your finances or investments?".to_string()
                }
            },
            _ => {
                "I'm not sure I understand your question. Could you please rephrase it or provide more details about your financial inquiry?".to_string()
            }
        }
    }
}

// Tests have been moved to tests/nlp_tests.rs 