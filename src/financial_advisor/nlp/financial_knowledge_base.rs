use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader};
use std::path::Path;
use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use super::knowledge_retriever::{KnowledgeItem, KnowledgeSourceType, KnowledgeRetriever};
use super::rule_based::FinancialQueryIntent;

/// Financial knowledge category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FinancialKnowledgeCategory {
    /// Retirement planning
    RetirementPlanning,
    
    /// Tax planning
    TaxPlanning,
    
    /// Investment strategies
    InvestmentStrategies,
    
    /// Estate planning
    EstatePlanning,
    
    /// Insurance planning
    InsurancePlanning,
    
    /// Education planning
    EducationPlanning,
    
    /// Financial concepts
    FinancialConcepts,
    
    /// Market data
    MarketData,
    
    /// Regulatory information
    RegulatoryInformation,
}

/// Financial knowledge source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialKnowledgeSource {
    /// Source name
    pub name: String,
    
    /// Source URL (if available)
    pub url: Option<String>,
    
    /// Source description
    pub description: String,
    
    /// Last updated date (ISO 8601 format)
    pub last_updated: String,
}

/// Financial knowledge entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialKnowledgeEntry {
    /// Entry ID
    pub id: String,
    
    /// Title
    pub title: String,
    
    /// Content
    pub content: String,
    
    /// Category
    pub category: FinancialKnowledgeCategory,
    
    /// Tags
    pub tags: Vec<String>,
    
    /// Related intents
    pub related_intents: Vec<FinancialQueryIntent>,
    
    /// Source
    pub source: FinancialKnowledgeSource,
    
    /// Relevance score (0.0 to 1.0)
    pub relevance_score: f32,
}

/// Financial knowledge base configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialKnowledgeBaseConfig {
    /// Base directory for knowledge files
    pub base_directory: Option<String>,
    
    /// Auto-update interval in seconds (0 means no auto-update)
    pub auto_update_interval: u64,
    
    /// Default relevance score threshold
    pub default_relevance_threshold: f32,
    
    /// Maximum entries to load per category
    pub max_entries_per_category: usize,
}

impl Default for FinancialKnowledgeBaseConfig {
    fn default() -> Self {
        Self {
            base_directory: None,
            auto_update_interval: 0,
            default_relevance_threshold: 0.7,
            max_entries_per_category: 1000,
        }
    }
}

/// Financial knowledge base
#[derive(Debug)]
pub struct FinancialKnowledgeBase {
    /// Knowledge entries by category
    entries_by_category: HashMap<FinancialKnowledgeCategory, Vec<FinancialKnowledgeEntry>>,
    
    /// Knowledge entries by ID
    entries_by_id: HashMap<String, FinancialKnowledgeEntry>,
    
    /// Configuration
    config: FinancialKnowledgeBaseConfig,
    
    /// Last update timestamp
    last_update: std::time::SystemTime,
}

impl FinancialKnowledgeBase {
    /// Create a new financial knowledge base
    pub fn new(config: FinancialKnowledgeBaseConfig) -> Self {
        Self {
            entries_by_category: HashMap::new(),
            entries_by_id: HashMap::new(),
            config,
            last_update: std::time::SystemTime::now(),
        }
    }
    
    /// Initialize the knowledge base with default entries
    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing financial knowledge base with default entries");
        
        // Add retirement planning entries
        self.add_retirement_planning_entries()?;
        
        // Add tax planning entries
        self.add_tax_planning_entries()?;
        
        // Add investment strategies entries
        self.add_investment_strategies_entries()?;
        
        // Add financial concepts entries
        self.add_financial_concepts_entries()?;
        
        info!("Financial knowledge base initialized with {} entries", self.entries_by_id.len());
        
        Ok(())
    }
    
    /// Add retirement planning entries
    fn add_retirement_planning_entries(&mut self) -> Result<()> {
        let entries = vec![
            FinancialKnowledgeEntry {
                id: "retirement-001".to_string(),
                title: "4% Rule for Retirement Withdrawals".to_string(),
                content: "The 4% rule is a guideline used to determine how much a retiree should withdraw from a retirement account each year. This rule seeks to provide a steady income stream to the retiree while also maintaining an account balance that keeps income flowing through retirement. The rule states that you should withdraw 4% of your retirement portfolio in the first year of retirement, then adjust that amount for inflation each subsequent year.".to_string(),
                category: FinancialKnowledgeCategory::RetirementPlanning,
                tags: vec!["retirement", "withdrawal", "4% rule", "income"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::RetirementPlanning, FinancialQueryIntent::CashFlowAnalysis],
                source: FinancialKnowledgeSource {
                    name: "Financial Planning Association".to_string(),
                    url: Some("https://www.fpa.org".to_string()),
                    description: "Professional association for financial planners".to_string(),
                    last_updated: "2023-01-15".to_string(),
                },
                relevance_score: 0.9,
            },
            FinancialKnowledgeEntry {
                id: "retirement-002".to_string(),
                title: "Required Minimum Distributions (RMDs)".to_string(),
                content: "Required Minimum Distributions (RMDs) are the minimum amounts that a retirement plan account owner must withdraw annually, generally starting with the year that they reach age 73 (as of 2023 under SECURE 2.0 Act). The RMD rules apply to traditional IRAs, SEP IRAs, SIMPLE IRAs, 401(k) plans, 403(b) plans, 457(b) plans, profit-sharing plans, and other defined contribution plans. They do not apply to Roth IRAs during the owner's lifetime.".to_string(),
                category: FinancialKnowledgeCategory::RetirementPlanning,
                tags: vec!["retirement", "RMD", "IRA", "401(k)", "tax"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::RetirementPlanning, FinancialQueryIntent::TaxOptimization],
                source: FinancialKnowledgeSource {
                    name: "Internal Revenue Service".to_string(),
                    url: Some("https://www.irs.gov".to_string()),
                    description: "U.S. government agency responsible for tax collection and tax law enforcement".to_string(),
                    last_updated: "2023-02-10".to_string(),
                },
                relevance_score: 0.95,
            },
            FinancialKnowledgeEntry {
                id: "retirement-003".to_string(),
                title: "Social Security Claiming Strategies".to_string(),
                content: "When to claim Social Security benefits is one of the most important retirement decisions. You can start receiving benefits as early as age 62, but your benefit amount will be reduced. If you wait until your full retirement age (between 66 and 67 depending on birth year), you'll receive 100% of your benefit. If you delay claiming until age 70, your benefit will increase by 8% per year beyond full retirement age. For married couples, coordinating claiming strategies can maximize lifetime benefits.".to_string(),
                category: FinancialKnowledgeCategory::RetirementPlanning,
                tags: vec!["retirement", "social security", "benefits", "claiming"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::RetirementPlanning, FinancialQueryIntent::SocialSecurityOptimization],
                source: FinancialKnowledgeSource {
                    name: "Social Security Administration".to_string(),
                    url: Some("https://www.ssa.gov".to_string()),
                    description: "U.S. government agency that administers Social Security programs".to_string(),
                    last_updated: "2023-03-05".to_string(),
                },
                relevance_score: 0.9,
            },
        ];
        
        for entry in entries {
            self.add_entry(entry)?;
        }
        
        Ok(())
    }
    
    /// Add tax planning entries
    fn add_tax_planning_entries(&mut self) -> Result<()> {
        let entries = vec![
            FinancialKnowledgeEntry {
                id: "tax-001".to_string(),
                title: "Tax-Loss Harvesting".to_string(),
                content: "Tax-loss harvesting is the practice of selling a security that has experienced a loss. By realizing a loss, investors can offset taxes on both gains and income. The sold security is replaced by a similar one, maintaining the optimal asset allocation and expected returns. To comply with the wash-sale rule, investors must avoid purchasing a 'substantially identical' security within 30 days before or after the sale.".to_string(),
                category: FinancialKnowledgeCategory::TaxPlanning,
                tags: vec!["tax", "investment", "loss harvesting", "wash sale"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::TaxOptimization, FinancialQueryIntent::InvestmentRecommendation],
                source: FinancialKnowledgeSource {
                    name: "Journal of Financial Planning".to_string(),
                    url: Some("https://www.financialplanningassociation.org/journal".to_string()),
                    description: "Peer-reviewed journal for financial planning professionals".to_string(),
                    last_updated: "2023-01-20".to_string(),
                },
                relevance_score: 0.85,
            },
            FinancialKnowledgeEntry {
                id: "tax-002".to_string(),
                title: "Roth Conversion Strategies".to_string(),
                content: "A Roth conversion involves transferring retirement funds from a traditional IRA, SEP IRA, SIMPLE IRA, or retirement plan like a 401(k) to a Roth IRA. The conversion amount is generally subject to income tax in the year of conversion, but qualified withdrawals from the Roth IRA in the future are tax-free. Roth conversions can be particularly advantageous during low-income years, in anticipation of higher future tax rates, or to reduce future RMDs.".to_string(),
                category: FinancialKnowledgeCategory::TaxPlanning,
                tags: vec!["tax", "retirement", "Roth", "conversion", "IRA"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::TaxOptimization, FinancialQueryIntent::RetirementPlanning],
                source: FinancialKnowledgeSource {
                    name: "Internal Revenue Service".to_string(),
                    url: Some("https://www.irs.gov".to_string()),
                    description: "U.S. government agency responsible for tax collection and tax law enforcement".to_string(),
                    last_updated: "2023-02-15".to_string(),
                },
                relevance_score: 0.9,
            },
        ];
        
        for entry in entries {
            self.add_entry(entry)?;
        }
        
        Ok(())
    }
    
    /// Add investment strategies entries
    fn add_investment_strategies_entries(&mut self) -> Result<()> {
        let entries = vec![
            FinancialKnowledgeEntry {
                id: "invest-001".to_string(),
                title: "Asset Allocation Fundamentals".to_string(),
                content: "Asset allocation is an investment strategy that aims to balance risk and reward by apportioning a portfolio's assets according to an individual's goals, risk tolerance, and investment horizon. The three main asset classes - equities, fixed-income, and cash and equivalents - have different levels of risk and return, so each will behave differently over time. A well-diversified portfolio tends to be more stable and less susceptible to market volatility.".to_string(),
                category: FinancialKnowledgeCategory::InvestmentStrategies,
                tags: vec!["investment", "asset allocation", "diversification", "portfolio"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::AssetAllocation, FinancialQueryIntent::InvestmentRecommendation],
                source: FinancialKnowledgeSource {
                    name: "CFA Institute".to_string(),
                    url: Some("https://www.cfainstitute.org".to_string()),
                    description: "Global association of investment professionals".to_string(),
                    last_updated: "2023-01-25".to_string(),
                },
                relevance_score: 0.9,
            },
            FinancialKnowledgeEntry {
                id: "invest-002".to_string(),
                title: "Factor Investing".to_string(),
                content: "Factor investing is an investment approach that involves targeting specific drivers of return across asset classes. Common factors include value, size, momentum, quality, and low volatility. These factors have been shown to deliver higher risk-adjusted returns over time. Factor investing can be implemented through individual securities selection or through factor-based ETFs and mutual funds.".to_string(),
                category: FinancialKnowledgeCategory::InvestmentStrategies,
                tags: vec!["investment", "factor", "smart beta", "portfolio"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::InvestmentRecommendation, FinancialQueryIntent::PortfolioPerformance],
                source: FinancialKnowledgeSource {
                    name: "Journal of Portfolio Management".to_string(),
                    url: Some("https://jpm.pm-research.com".to_string()),
                    description: "Academic journal focused on portfolio management".to_string(),
                    last_updated: "2023-03-10".to_string(),
                },
                relevance_score: 0.85,
            },
        ];
        
        for entry in entries {
            self.add_entry(entry)?;
        }
        
        Ok(())
    }
    
    /// Add financial concepts entries
    fn add_financial_concepts_entries(&mut self) -> Result<()> {
        let entries = vec![
            FinancialKnowledgeEntry {
                id: "concept-001".to_string(),
                title: "Time Value of Money".to_string(),
                content: "The time value of money is the concept that money available now is worth more than the same amount in the future due to its potential earning capacity. This core principle of finance holds that provided money can earn interest, any amount of money is worth more the sooner it is received. The time value of money is calculated using various formulas including present value, future value, and discounted cash flow analysis.".to_string(),
                category: FinancialKnowledgeCategory::FinancialConcepts,
                tags: vec!["finance", "time value", "present value", "future value"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::FinancialEducation, FinancialQueryIntent::InvestmentRecommendation],
                source: FinancialKnowledgeSource {
                    name: "Financial Analysts Journal".to_string(),
                    url: Some("https://www.cfainstitute.org/en/research/financial-analysts-journal".to_string()),
                    description: "Publication of investment knowledge for financial analysts".to_string(),
                    last_updated: "2023-02-05".to_string(),
                },
                relevance_score: 0.8,
            },
            FinancialKnowledgeEntry {
                id: "concept-002".to_string(),
                title: "Risk-Return Tradeoff".to_string(),
                content: "The risk-return tradeoff is the principle that potential return rises with an increase in risk. Low levels of uncertainty (low risk) are associated with low potential returns, whereas high levels of uncertainty (high risk) are associated with high potential returns. This relationship is central to modern portfolio theory and guides investment decisions based on an investor's risk tolerance and financial goals.".to_string(),
                category: FinancialKnowledgeCategory::FinancialConcepts,
                tags: vec!["finance", "risk", "return", "investment"].iter().map(|s| s.to_string()).collect(),
                related_intents: vec![FinancialQueryIntent::FinancialEducation, FinancialQueryIntent::RiskAssessment],
                source: FinancialKnowledgeSource {
                    name: "Journal of Finance".to_string(),
                    url: Some("https://afajof.org".to_string()),
                    description: "Academic journal published by the American Finance Association".to_string(),
                    last_updated: "2023-01-30".to_string(),
                },
                relevance_score: 0.85,
            },
        ];
        
        for entry in entries {
            self.add_entry(entry)?;
        }
        
        Ok(())
    }
    
    /// Add an entry to the knowledge base
    pub fn add_entry(&mut self, entry: FinancialKnowledgeEntry) -> Result<()> {
        // Check if entry with this ID already exists
        if self.entries_by_id.contains_key(&entry.id) {
            return Err(anyhow!("Entry with ID {} already exists", entry.id));
        }
        
        // Add to category map
        self.entries_by_category
            .entry(entry.category.clone())
            .or_insert_with(Vec::new)
            .push(entry.clone());
        
        // Add to ID map
        self.entries_by_id.insert(entry.id.clone(), entry);
        
        Ok(())
    }
    
    /// Get an entry by ID
    pub fn get_entry_by_id(&self, id: &str) -> Option<&FinancialKnowledgeEntry> {
        self.entries_by_id.get(id)
    }
    
    /// Get entries by category
    pub fn get_entries_by_category(&self, category: &FinancialKnowledgeCategory) -> Vec<&FinancialKnowledgeEntry> {
        if let Some(entries) = self.entries_by_category.get(category) {
            entries.iter().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Search entries by query
    pub fn search(&self, query: &str) -> Vec<&FinancialKnowledgeEntry> {
        let query = query.to_lowercase();
        let mut results = Vec::new();
        
        for entry in self.entries_by_id.values() {
            // Check if query matches title, content, or tags
            if entry.title.to_lowercase().contains(&query) || 
               entry.content.to_lowercase().contains(&query) ||
               entry.tags.iter().any(|tag| tag.to_lowercase().contains(&query)) {
                results.push(entry);
            }
        }
        
        // Sort by relevance score (descending)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        results
    }
    
    /// Search entries by intent
    pub fn search_by_intent(&self, intent: &FinancialQueryIntent) -> Vec<&FinancialKnowledgeEntry> {
        let mut results = Vec::new();
        
        for entry in self.entries_by_id.values() {
            if entry.related_intents.contains(intent) {
                results.push(entry);
            }
        }
        
        // Sort by relevance score (descending)
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        
        results
    }
    
    /// Load entries from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let file = File::open(path).context("Failed to open knowledge base file")?;
        let reader = BufReader::new(file);
        
        let entries: Vec<FinancialKnowledgeEntry> = serde_json::from_reader(reader)
            .context("Failed to parse knowledge base file")?;
        
        info!("Loading {} entries from file", entries.len());
        
        for entry in entries {
            match self.add_entry(entry) {
                Ok(_) => {},
                Err(e) => warn!("Failed to add entry: {}", e),
            }
        }
        
        self.last_update = std::time::SystemTime::now();
        
        Ok(())
    }
    
    /// Save entries to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let entries: Vec<&FinancialKnowledgeEntry> = self.entries_by_id.values().collect();
        let file = File::create(path).context("Failed to create knowledge base file")?;
        
        serde_json::to_writer_pretty(file, &entries)
            .context("Failed to write knowledge base file")?;
        
        Ok(())
    }
    
    /// Get the number of entries in the knowledge base
    pub fn len(&self) -> usize {
        self.entries_by_id.len()
    }
    
    /// Check if the knowledge base is empty
    pub fn is_empty(&self) -> bool {
        self.entries_by_id.is_empty()
    }
    
    /// Get all categories
    pub fn get_categories(&self) -> Vec<FinancialKnowledgeCategory> {
        self.entries_by_category.keys().cloned().collect()
    }
    
    /// Convert to KnowledgeItems for use with KnowledgeRetriever
    pub fn to_knowledge_items(&self) -> Vec<KnowledgeItem> {
        self.entries_by_id.values()
            .map(|entry| {
                let source_type = match entry.category {
                    FinancialKnowledgeCategory::RetirementPlanning => KnowledgeSourceType::RetirementPlanning,
                    FinancialKnowledgeCategory::TaxPlanning => KnowledgeSourceType::TaxRules,
                    FinancialKnowledgeCategory::InvestmentStrategies => KnowledgeSourceType::InvestmentStrategy,
                    FinancialKnowledgeCategory::EstatePlanning => KnowledgeSourceType::EstatePlanning,
                    FinancialKnowledgeCategory::InsurancePlanning => KnowledgeSourceType::InsurancePlanning,
                    FinancialKnowledgeCategory::EducationPlanning => KnowledgeSourceType::EducationPlanning,
                    FinancialKnowledgeCategory::FinancialConcepts => KnowledgeSourceType::FinancialConcept,
                    FinancialKnowledgeCategory::MarketData => KnowledgeSourceType::MarketData,
                    FinancialKnowledgeCategory::RegulatoryInformation => KnowledgeSourceType::RegulatoryInfo,
                };
                
                KnowledgeItem {
                    id: entry.id.clone(),
                    title: entry.title.clone(),
                    content: entry.content.clone(),
                    source_type,
                    tags: entry.tags.clone(),
                    related_intents: entry.related_intents.clone(),
                    embedding: None,
                }
            })
            .collect()
    }
    
    /// Populate a KnowledgeRetriever with entries from this knowledge base
    pub fn populate_knowledge_retriever(&self, retriever: &mut KnowledgeRetriever) -> Result<()> {
        let items = self.to_knowledge_items();
        
        info!("Populating knowledge retriever with {} items", items.len());
        
        for item in items {
            retriever.add_item(item)?;
        }
        
        Ok(())
    }
}

/// Create a default financial knowledge base
pub fn create_default_knowledge_base() -> Result<FinancialKnowledgeBase> {
    let config = FinancialKnowledgeBaseConfig::default();
    let mut kb = FinancialKnowledgeBase::new(config);
    
    kb.initialize()?;
    
    Ok(kb)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_knowledge_base() {
        let kb = create_default_knowledge_base().unwrap();
        assert!(!kb.is_empty());
    }
    
    #[test]
    fn test_search() {
        let kb = create_default_knowledge_base().unwrap();
        let results = kb.search("retirement");
        assert!(!results.is_empty());
    }
    
    #[test]
    fn test_search_by_intent() {
        let kb = create_default_knowledge_base().unwrap();
        let results = kb.search_by_intent(&FinancialQueryIntent::RetirementPlanning);
        assert!(!results.is_empty());
    }
    
    #[test]
    fn test_to_knowledge_items() {
        let kb = create_default_knowledge_base().unwrap();
        let items = kb.to_knowledge_items();
        assert_eq!(items.len(), kb.len());
    }
} 