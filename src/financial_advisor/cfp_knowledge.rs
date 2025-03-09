use std::collections::HashMap;

/// Represents the CFP knowledge base with categories and information
pub struct CfpKnowledgeBase {
    categories: HashMap<String, Vec<CfpKnowledgeItem>>,
}

/// Represents a single knowledge item in the CFP knowledge base
pub struct CfpKnowledgeItem {
    pub title: String,
    pub content: String,
    pub source: String,
    pub keywords: Vec<String>,
}

impl CfpKnowledgeBase {
    /// Create a new CFP knowledge base with predefined information
    pub fn new() -> Self {
        let mut kb = Self {
            categories: HashMap::new(),
        };
        
        kb.initialize();
        kb
    }
    
    /// Initialize the knowledge base with CFP information
    fn initialize(&mut self) {
        // Add CFP exam topics
        self.add_category("exam_topics", vec![
            CfpKnowledgeItem {
                title: "CFP Exam Topics".to_string(),
                content: "The CFP exam covers eight principal knowledge domains: Professional Conduct and Regulation (7%), General Financial Planning Principles (17%), Education Planning (6%), Risk Management and Insurance Planning (12%), Investment Planning (17%), Tax Planning (14%), Retirement Savings and Income Planning (17%), and Estate Planning (10%).".to_string(),
                source: "CFP Board".to_string(),
                keywords: vec!["exam", "topics", "domains", "certification"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
        
        // Add ethics standards
        self.add_category("ethics", vec![
            CfpKnowledgeItem {
                title: "CFP Code of Ethics".to_string(),
                content: "The CFP Board's Code of Ethics includes: 1) Act with honesty, integrity, competence, and diligence. 2) Act in the client's best interests. 3) Exercise due care. 4) Avoid or disclose and manage conflicts of interest. 5) Maintain the confidentiality and protect the privacy of client information. 6) Act in a manner that reflects positively on the financial planning profession and CFP certification.".to_string(),
                source: "CFP Board Code of Ethics".to_string(),
                keywords: vec!["ethics", "standards", "conduct", "fiduciary"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
        
        // Add financial planning process
        self.add_category("planning_process", vec![
            CfpKnowledgeItem {
                title: "Financial Planning Process".to_string(),
                content: "The financial planning process consists of: 1) Understanding the client's personal and financial circumstances. 2) Identifying and selecting goals. 3) Analyzing the client's current course of action and potential alternative course(s) of action. 4) Developing the financial planning recommendation(s). 5) Presenting the financial planning recommendation(s). 6) Implementing the financial planning recommendation(s). 7) Monitoring progress and updating.".to_string(),
                source: "CFP Board Practice Standards".to_string(),
                keywords: vec!["process", "planning", "steps", "methodology"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
        
        // Add retirement planning
        self.add_category("retirement", vec![
            CfpKnowledgeItem {
                title: "SECURE 2.0 Act Key Provisions".to_string(),
                content: "The SECURE 2.0 Act of 2022 includes several key provisions: 1) Automatic enrollment in 401(k) and 403(b) plans. 2) Increased age for required minimum distributions (RMDs) to 73 in 2023 and 75 in 2033. 3) Higher catch-up contribution limits for those aged 60-63. 4) Emergency withdrawals up to $1,000 without penalty. 5) Employer matching for student loan payments. 6) Creation of starter 401(k) plans for small businesses.".to_string(),
                source: "SECURE 2.0 Act of 2022".to_string(),
                keywords: vec!["retirement", "SECURE Act", "RMD", "401k", "legislation"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
        
        // Add tax planning
        self.add_category("tax_planning", vec![
            CfpKnowledgeItem {
                title: "Tax Cuts and Jobs Act".to_string(),
                content: "The Tax Cuts and Jobs Act (TCJA) made significant changes to the tax code, including: 1) Reduced individual tax rates. 2) Increased standard deduction. 3) Limited state and local tax (SALT) deductions to $10,000. 4) Limited mortgage interest deduction. 5) Eliminated personal exemptions. 6) Expanded child tax credit. 7) Created a 20% qualified business income deduction for pass-through entities. Many individual provisions are scheduled to expire after 2025.".to_string(),
                source: "Tax Cuts and Jobs Act".to_string(),
                keywords: vec!["tax", "TCJA", "deduction", "credit", "legislation"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
        
        // Add investment planning
        self.add_category("investments", vec![
            CfpKnowledgeItem {
                title: "Investment Risk Types".to_string(),
                content: "Key investment risks include: 1) Market risk - potential for investments to lose value due to market factors. 2) Interest rate risk - impact of interest rate changes on investment values. 3) Inflation risk - possibility that investment returns won't keep pace with inflation. 4) Credit/default risk - risk that a borrower will fail to repay. 5) Liquidity risk - difficulty selling an investment at a fair price. 6) Concentration risk - inadequate diversification. 7) Currency risk - impact of exchange rate fluctuations on foreign investments.".to_string(),
                source: "CFP Board Investment Planning Curriculum".to_string(),
                keywords: vec!["investment", "risk", "market", "diversification"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
        
        // Add estate planning
        self.add_category("estate_planning", vec![
            CfpKnowledgeItem {
                title: "Estate Planning Fundamentals".to_string(),
                content: "Essential estate planning documents include: 1) Will - directs distribution of assets and names guardians for minor children. 2) Revocable living trust - allows assets to pass outside of probate and provides management in case of incapacity. 3) Durable power of attorney - appoints someone to handle financial affairs if incapacitated. 4) Healthcare power of attorney - designates someone to make medical decisions. 5) Living will/advance directive - specifies end-of-life care preferences. 6) HIPAA authorization - allows access to medical information.".to_string(),
                source: "CFP Board Estate Planning Curriculum".to_string(),
                keywords: vec!["estate", "will", "trust", "probate", "power of attorney"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
        
        // Add risk management
        self.add_category("risk_management", vec![
            CfpKnowledgeItem {
                title: "Insurance Planning Principles".to_string(),
                content: "Key insurance planning principles include: 1) Risk identification - recognizing potential financial risks. 2) Risk evaluation - assessing potential impact and likelihood. 3) Risk management techniques - avoidance, reduction, retention, and transfer. 4) Insurance types - life, health, disability, long-term care, property & casualty. 5) Policy analysis - evaluating coverage, exclusions, limitations, and costs. 6) Company selection - considering financial strength, claims payment history, and service quality.".to_string(),
                source: "CFP Board Risk Management Curriculum".to_string(),
                keywords: vec!["insurance", "risk", "policy", "coverage", "protection"].iter().map(|s| s.to_string()).collect(),
            },
        ]);
    }
    
    /// Add a category with knowledge items to the knowledge base
    fn add_category(&mut self, category: &str, items: Vec<CfpKnowledgeItem>) {
        self.categories.insert(category.to_string(), items);
    }
    
    /// Retrieve knowledge items by category
    pub fn get_by_category(&self, category: &str) -> Option<&Vec<CfpKnowledgeItem>> {
        self.categories.get(category)
    }
    
    /// Search for knowledge items by keywords
    pub fn search(&self, query: &str) -> Vec<&CfpKnowledgeItem> {
        let query = query.to_lowercase();
        let mut results = Vec::new();
        
        for items in self.categories.values() {
            for item in items {
                // Check if query matches title, content, or keywords
                if item.title.to_lowercase().contains(&query) || 
                   item.content.to_lowercase().contains(&query) ||
                   item.keywords.iter().any(|k| k.to_lowercase().contains(&query)) {
                    results.push(item);
                }
            }
        }
        
        results
    }
    
    /// Get all categories
    pub fn get_categories(&self) -> Vec<&String> {
        self.categories.keys().collect()
    }
    
    /// Get a summary of the knowledge base
    pub fn get_summary(&self) -> String {
        let mut summary = String::from("CFP Knowledge Base Summary:\n");
        
        for (category, items) in &self.categories {
            summary.push_str(&format!("- {} ({} items)\n", category, items.len()));
        }
        
        summary
    }
}

/// Provides a formatted string with CFP knowledge for a specific query
pub fn get_cfp_knowledge_for_query(query: &str) -> String {
    let kb = CfpKnowledgeBase::new();
    let results = kb.search(query);
    
    if results.is_empty() {
        return format!("No specific CFP knowledge found for '{}'.", query);
    }
    
    let mut response = format!("CFP Knowledge related to '{}':\n\n", query);
    
    for (i, item) in results.iter().enumerate() {
        response.push_str(&format!("{}. {}\n", i+1, item.title));
        response.push_str(&format!("   {}\n", item.content));
        response.push_str(&format!("   Source: {}\n\n", item.source));
    }
    
    response
} 