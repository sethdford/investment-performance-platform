use chrono::{NaiveDate, Utc};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents different types of charitable vehicles used for donations
#[derive(Debug, Clone, PartialEq)]
enum CharitableVehicleType {
    /// Donor-advised fund allows for immediate tax deduction and later distribution
    DonorAdvisedFund,
    /// Private foundation for larger-scale charitable giving
    PrivateFoundation,
    /// Trust that provides income to beneficiaries with remainder to charity
    CharitableRemainder,
    /// Distribution from retirement account directly to charity
    QualifiedCharitableDistribution,
}

/// Represents a charitable organization
#[derive(Debug, Clone)]
struct Charity {
    id: String,
    name: String,
    ein: Option<String>,
    mission: String,
    category: String,
    created_at: NaiveDate,
}

/// Represents a vehicle used for charitable giving
#[derive(Debug, Clone)]
struct CharitableVehicle {
    id: String,
    name: String,
    vehicle_type: CharitableVehicleType,
    balance: f64,
    created_at: NaiveDate,
}

/// Types of donations that can be made to charities
#[derive(Debug, Clone)]
enum DonationType {
    /// Direct cash donation
    Cash,
    /// Donation of securities (stocks, bonds, etc.)
    Securities,
    /// Donation of physical goods or services
    InKind,
    /// Distribution from retirement account directly to charity
    QualifiedCharitableDistribution,
}

/// Represents a donation made to a charity
#[derive(Debug, Clone)]
struct CharitableDonation {
    id: String,
    charity_id: String,
    amount: f64,
    donation_type: DonationType,
    donation_date: NaiveDate,
    vehicle_id: Option<String>,
    security_id: Option<String>,
    tax_year: i32,
    receipt_received: bool,
    notes: Option<String>,
}

/// Tax impact of a charitable donation
#[derive(Debug, Clone)]
struct CharitableTaxImpact {
    donation_id: String,
    tax_year: i32,
    federal_deduction: f64,
    state_deduction: f64,
    estimated_tax_savings: f64,
}

/// Strategy for optimizing charitable giving
#[derive(Debug, Clone)]
struct DonationStrategy {
    id: String,
    description: String,
    estimated_tax_savings: f64,
    priority: i32,
}

/// Plan for charitable giving over a period
#[derive(Debug, Clone)]
struct CharitableGivingPlan {
    annual_target: f64,
    strategies: Vec<DonationStrategy>,
    tax_impact: f64,
}

/// Report summarizing charitable giving activity
#[derive(Debug, Clone)]
struct CharitableGivingReport {
    total_donations: f64,
    donations_by_charity: HashMap<String, f64>,
    tax_impact: f64,
    strategies: Vec<DonationStrategy>,
}

/// Simplified household structure for charitable giving analysis
#[derive(Debug, Clone)]
struct SimplifiedHousehold {
    id: String,
    name: String,
    charities: Vec<Charity>,
    charitable_vehicles: Vec<CharitableVehicle>,
    donations: Vec<CharitableDonation>,
    tax_bracket: f64,
    state_tax_rate: f64,
}

impl SimplifiedHousehold {
    /// Creates a new household with default tax rates
    fn new(name: &str) -> Self {
        if name.trim().is_empty() {
            panic!("Household name cannot be empty");
        }
        
        SimplifiedHousehold {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            charities: Vec::new(),
            charitable_vehicles: Vec::new(),
            donations: Vec::new(),
            tax_bracket: 0.35, // Default to 35% federal tax bracket
            state_tax_rate: 0.05, // Default to 5% state tax rate
        }
    }

    /// Adds a charity to the household and returns its ID
    fn add_charity(&mut self, name: &str, ein: Option<&str>, mission: &str, category: &str) -> String {
        // Validate inputs
        if name.trim().is_empty() {
            panic!("Charity name cannot be empty");
        }
        
        if mission.trim().is_empty() {
            panic!("Charity mission cannot be empty");
        }
        
        if category.trim().is_empty() {
            panic!("Charity category cannot be empty");
        }
        
        // Validate EIN format if provided
        if let Some(ein_str) = ein {
            if !ein_str.contains('-') || ein_str.len() != 10 {
                panic!("EIN should be in the format XX-XXXXXXX");
            }
        }
        
        let charity = Charity {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            ein: ein.map(|s| s.to_string()),
            mission: mission.to_string(),
            category: category.to_string(),
            created_at: Utc::now().date_naive(),
        };
        
        let id = charity.id.clone();
        self.charities.push(charity);
        id
    }

    /// Adds a charitable vehicle to the household and returns its ID
    fn add_charitable_vehicle(&mut self, name: &str, vehicle_type: CharitableVehicleType, balance: f64) -> String {
        // Validate inputs
        if name.trim().is_empty() {
            panic!("Vehicle name cannot be empty");
        }
        
        if balance < 0.0 {
            panic!("Vehicle balance cannot be negative");
        }
        
        let vehicle = CharitableVehicle {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            vehicle_type,
            balance,
            created_at: Utc::now().date_naive(),
        };
        
        let id = vehicle.id.clone();
        self.charitable_vehicles.push(vehicle);
        id
    }

    /// Records a donation to a charity and returns the donation ID
    fn record_donation(
        &mut self, 
        charity_id: &str, 
        amount: f64, 
        donation_type: DonationType, 
        donation_date: NaiveDate,
        vehicle_id: Option<&str>,
        security_id: Option<&str>,
        tax_year: i32,
        receipt_received: bool,
        notes: Option<&str>
    ) -> String {
        // Validate inputs
        if charity_id.trim().is_empty() {
            panic!("Charity ID cannot be empty");
        }
        
        if amount <= 0.0 {
            panic!("Donation amount must be positive");
        }
        
        if tax_year < 1900 || tax_year > 2100 {
            panic!("Tax year must be reasonable (between 1900 and 2100)");
        }
        
        // Verify charity exists
        if !self.charities.iter().any(|c| c.id == charity_id) {
            panic!("Charity with ID {} does not exist", charity_id);
        }
        
        // Verify vehicle exists if provided
        if let Some(v_id) = vehicle_id {
            if !self.charitable_vehicles.iter().any(|v| v.id == v_id) {
                panic!("Charitable vehicle with ID {} does not exist", v_id);
            }
        }
        
        let donation = CharitableDonation {
            id: Uuid::new_v4().to_string(),
            charity_id: charity_id.to_string(),
            amount,
            donation_type,
            donation_date,
            vehicle_id: vehicle_id.map(|s| s.to_string()),
            security_id: security_id.map(|s| s.to_string()),
            tax_year,
            receipt_received,
            notes: notes.map(|s| s.to_string()),
        };
        
        let id = donation.id.clone();
        self.donations.push(donation);
        id
    }

    /// Analyzes the tax impact of all donations
    fn analyze_tax_impact(&self) -> Vec<CharitableTaxImpact> {
        self.donations.iter().map(|donation| {
            let federal_deduction = match donation.donation_type {
                DonationType::Cash => donation.amount, // Simplified - removed AGI limitation
                DonationType::Securities => donation.amount,
                DonationType::InKind => donation.amount * 0.8,
                DonationType::QualifiedCharitableDistribution => 0.0,
            };
            
            let state_deduction = federal_deduction * 0.9; // State often has lower limits
            
            let estimated_tax_savings = 
                (federal_deduction * self.tax_bracket) + 
                (state_deduction * self.state_tax_rate);
                
            CharitableTaxImpact {
                donation_id: donation.id.clone(),
                tax_year: donation.tax_year,
                federal_deduction,
                state_deduction,
                estimated_tax_savings,
            }
        }).collect()
    }
    
    /// Calculates the total amount of all donations
    fn total_donations(&self) -> f64 {
        self.donations.iter().map(|d| d.amount).sum()
    }
    
    /// Groups donations by charity and returns a map of charity ID to total donation amount
    fn donations_by_charity(&self) -> HashMap<String, f64> {
        let mut result = HashMap::new();
        
        for donation in &self.donations {
            *result.entry(donation.charity_id.clone()).or_insert(0.0) += donation.amount;
        }
        
        result
    }
    
    /// Generates donation strategies based on the household's donation patterns
    fn generate_donation_strategies(&self) -> Vec<DonationStrategy> {
        let mut strategies = Vec::new();
        
        // Strategy 1: Bunch donations
        if self.total_donations() > 10000.0 {
            strategies.push(DonationStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Consider bunching donations in alternate years to exceed standard deduction threshold".to_string(),
                estimated_tax_savings: self.total_donations() * 0.05,
                priority: 1,
            });
        }
        
        // Strategy 2: Donate appreciated securities
        strategies.push(DonationStrategy {
            id: Uuid::new_v4().to_string(),
            description: "Donate appreciated securities instead of cash to avoid capital gains tax".to_string(),
            estimated_tax_savings: self.total_donations() * 0.10,
            priority: 2,
        });
        
        // Strategy 3: Qualified Charitable Distribution
        strategies.push(DonationStrategy {
            id: Uuid::new_v4().to_string(),
            description: "Use Qualified Charitable Distributions from IRAs if over 70½ to exclude from income".to_string(),
            estimated_tax_savings: self.total_donations() * 0.15,
            priority: 3,
        });
        
        // Strategy 4: Donor Advised Fund
        if self.total_donations() > 25000.0 {
            strategies.push(DonationStrategy {
                id: Uuid::new_v4().to_string(),
                description: "Establish a Donor Advised Fund for tax-efficient giving and simplified record-keeping".to_string(),
                estimated_tax_savings: self.total_donations() * 0.08,
                priority: 2,
            });
        }
        
        strategies
    }
    
    /// Generates a charitable giving plan based on an annual target
    fn generate_charitable_giving_plan(&self, annual_target: f64) -> CharitableGivingPlan {
        if annual_target <= 0.0 {
            panic!("Annual target must be positive");
        }
        
        let strategies = self.generate_donation_strategies();
        let tax_impact = strategies.iter().map(|s| s.estimated_tax_savings).sum();
        
        CharitableGivingPlan {
            annual_target,
            strategies,
            tax_impact,
        }
    }
    
    /// Generates a report summarizing charitable giving activity
    fn generate_charitable_giving_report(&self) -> CharitableGivingReport {
        let total_donations = self.total_donations();
        let donations_by_charity = self.donations_by_charity();
        let tax_impacts = self.analyze_tax_impact();
        let tax_impact = tax_impacts.iter().map(|t| t.estimated_tax_savings).sum();
        let strategies = self.generate_donation_strategies();
        
        CharitableGivingReport {
            total_donations,
            donations_by_charity,
            tax_impact,
            strategies,
        }
    }
    
    /// Gets a charity by ID
    fn get_charity_by_id(&self, charity_id: &str) -> Option<&Charity> {
        self.charities.iter().find(|c| c.id == charity_id)
    }

    /// Gets a charitable vehicle by ID
    fn get_vehicle_by_id(&self, vehicle_id: &str) -> Option<&CharitableVehicle> {
        self.charitable_vehicles.iter().find(|v| v.id == vehicle_id)
    }
}

/// Example demonstrating charitable giving analysis
fn main() {
    // Create a household
    let mut household = SimplifiedHousehold::new("Smith Family");
    
    // Set tax rates
    household.tax_bracket = 0.35; // 35% federal tax bracket
    household.state_tax_rate = 0.06; // 6% state tax rate
    
    // Add charities - store IDs instead of references
    let red_cross_id = household.add_charity(
        "American Red Cross", 
        Some("53-0196605"), 
        "Provides emergency assistance, disaster relief, and disaster preparedness education", 
        "Humanitarian"
    );
    
    let wwf_id = household.add_charity(
        "World Wildlife Fund", 
        Some("52-1693387"), 
        "International organization working on issues regarding the conservation of the environment", 
        "Environmental"
    );
    
    let food_bank_id = household.add_charity(
        "Local Food Bank", 
        Some("12-3456789"), 
        "Provides food to those in need in the local community", 
        "Hunger Relief"
    );
    
    let art_museum_id = household.add_charity(
        "City Art Museum",
        Some("45-6789123"),
        "Preserves and exhibits art for public education and enjoyment",
        "Arts & Culture"
    );
    
    // Add charitable vehicles - store IDs instead of references
    let daf_id = household.add_charitable_vehicle(
        "Family Donor Advised Fund", 
        CharitableVehicleType::DonorAdvisedFund, 
        50000.0
    );
    
    let qcd_id = household.add_charitable_vehicle(
        "IRA Qualified Charitable Distribution", 
        CharitableVehicleType::QualifiedCharitableDistribution, 
        100000.0
    );
    
    let private_foundation_id = household.add_charitable_vehicle(
        "Smith Family Foundation",
        CharitableVehicleType::PrivateFoundation,
        250000.0
    );
    
    let charitable_remainder_id = household.add_charitable_vehicle(
        "Smith Charitable Remainder Trust",
        CharitableVehicleType::CharitableRemainder,
        75000.0
    );
    
    // Record donations
    // Cash donation directly to charity
    household.record_donation(
        &red_cross_id,
        5000.0,
        DonationType::Cash,
        NaiveDate::from_ymd_opt(2023, 3, 15).unwrap(),
        None,
        None,
        2023,
        true,
        Some("Annual donation")
    );
    
    // Securities donation through DAF
    household.record_donation(
        &wwf_id,
        10000.0,
        DonationType::Securities,
        NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
        Some(&daf_id),
        Some("AAPL"),
        2023,
        true,
        Some("Appreciated stock donation")
    );
    
    // QCD from IRA
    household.record_donation(
        &food_bank_id,
        8000.0,
        DonationType::QualifiedCharitableDistribution,
        NaiveDate::from_ymd_opt(2023, 11, 30).unwrap(),
        Some(&qcd_id),
        None,
        2023,
        true,
        Some("Annual QCD from IRA")
    );
    
    // In-kind donation of artwork
    household.record_donation(
        &art_museum_id,
        15000.0,
        DonationType::InKind,
        NaiveDate::from_ymd_opt(2023, 10, 15).unwrap(),
        None,
        None,
        2023,
        false, // Receipt not yet received
        Some("Donation of original artwork")
    );
    
    // Donation through private foundation
    household.record_donation(
        &food_bank_id,
        20000.0,
        DonationType::Cash,
        NaiveDate::from_ymd_opt(2023, 12, 15).unwrap(),
        Some(&private_foundation_id),
        None,
        2023,
        true,
        Some("Year-end grant from family foundation")
    );
    
    // Donation through charitable remainder trust
    household.record_donation(
        &art_museum_id,
        5000.0,
        DonationType::Cash,
        NaiveDate::from_ymd_opt(2023, 9, 1).unwrap(),
        Some(&charitable_remainder_id),
        None,
        2023,
        true,
        Some("Donation from charitable remainder trust")
    );
    
    // Analyze tax impact
    let tax_impacts = household.analyze_tax_impact();
    println!("\n=== Charitable Donation Tax Impact ===");
    for impact in &tax_impacts {
        // Find donation for reference
        let donation = household.donations.iter()
            .find(|d| d.id == impact.donation_id)
            .expect("Donation should exist");
            
        // Find charity name
        let charity_name = household.get_charity_by_id(&donation.charity_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| donation.charity_id.clone());
            
        println!("Donation to {}: ${:.2}", charity_name, donation.amount);
        println!("  Donation Type: {:?}", donation.donation_type);
        println!("  Tax Year: {}", impact.tax_year);
        println!("  Federal Deduction: ${:.2}", impact.federal_deduction);
        println!("  State Deduction: ${:.2}", impact.state_deduction);
        println!("  Estimated Tax Savings: ${:.2}", impact.estimated_tax_savings);
        
        // Show vehicle if applicable
        if let Some(vehicle_id) = &donation.vehicle_id {
            if let Some(vehicle) = household.get_vehicle_by_id(vehicle_id) {
                println!("  Through Vehicle: {} ({})", 
                    vehicle.name, 
                    format!("{:?}", vehicle.vehicle_type)
                );
            }
        }
        println!("");
    }
    
    // Generate donation strategies
    let strategies = household.generate_donation_strategies();
    println!("\n=== Recommended Donation Strategies ===");
    for strategy in &strategies {
        println!("Priority {}: {}", strategy.priority, strategy.description);
        println!("  Estimated Tax Savings: ${:.2}", strategy.estimated_tax_savings);
    }
    
    // Generate charitable giving plan
    let plan = household.generate_charitable_giving_plan(60000.0);
    println!("\n=== Charitable Giving Plan ===");
    println!("Annual Target: ${:.2}", plan.annual_target);
    println!("Estimated Tax Impact: ${:.2}", plan.tax_impact);
    println!("Recommended Strategies:");
    for strategy in &plan.strategies {
        println!("- {}", strategy.description);
    }
    
    // Generate charitable giving report
    let report = household.generate_charitable_giving_report();
    println!("\n=== Charitable Giving Report ===");
    println!("Total Donations: ${:.2}", report.total_donations);
    println!("Tax Impact: ${:.2}", report.tax_impact);
    println!("Donations by Charity:");
    for (charity_id, amount) in &report.donations_by_charity {
        // Find charity name
        let charity_name = household.get_charity_by_id(charity_id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| charity_id.clone());
            
        println!("  {}: ${:.2}", charity_name, amount);
    }
    
    // Donation type breakdown
    println!("\nDonations by Type:");
    let mut cash_total = 0.0;
    let mut securities_total = 0.0;
    let mut inkind_total = 0.0;
    let mut qcd_total = 0.0;
    
    for donation in &household.donations {
        match donation.donation_type {
            DonationType::Cash => cash_total += donation.amount,
            DonationType::Securities => securities_total += donation.amount,
            DonationType::InKind => inkind_total += donation.amount,
            DonationType::QualifiedCharitableDistribution => qcd_total += donation.amount,
        }
    }
    
    println!("  Cash: ${:.2}", cash_total);
    println!("  Securities: ${:.2}", securities_total);
    println!("  In-Kind: ${:.2}", inkind_total);
    println!("  QCD: ${:.2}", qcd_total);
    
    println!("\nCharitable giving optimization completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_household_creation() {
        let household = SimplifiedHousehold::new("Test Household");
        assert_eq!(household.name, "Test Household");
        assert_eq!(household.charities.len(), 0);
        assert_eq!(household.charitable_vehicles.len(), 0);
        assert_eq!(household.donations.len(), 0);
        assert_eq!(household.tax_bracket, 0.35);
        assert_eq!(household.state_tax_rate, 0.05);
    }

    #[test]
    #[should_panic(expected = "Household name cannot be empty")]
    fn test_household_creation_empty_name() {
        SimplifiedHousehold::new("");
    }

    #[test]
    fn test_add_charity() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        assert_eq!(household.charities.len(), 1);
        assert_eq!(household.charities[0].name, "Test Charity");
        assert_eq!(household.charities[0].ein, Some("12-3456789".to_string()));
        assert_eq!(household.charities[0].mission, "Test Mission");
        assert_eq!(household.charities[0].category, "Test Category");
        assert_eq!(household.charities[0].id, charity_id);
    }

    #[test]
    #[should_panic(expected = "Charity name cannot be empty")]
    fn test_add_charity_empty_name() {
        let mut household = SimplifiedHousehold::new("Test Household");
        household.add_charity("", Some("12-3456789"), "Test Mission", "Test Category");
    }

    #[test]
    #[should_panic(expected = "EIN should be in the format XX-XXXXXXX")]
    fn test_add_charity_invalid_ein() {
        let mut household = SimplifiedHousehold::new("Test Household");
        household.add_charity("Test Charity", Some("123456789"), "Test Mission", "Test Category");
    }

    #[test]
    fn test_add_charitable_vehicle() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let vehicle_id = household.add_charitable_vehicle(
            "Test Vehicle",
            CharitableVehicleType::DonorAdvisedFund,
            10000.0
        );
        
        assert_eq!(household.charitable_vehicles.len(), 1);
        assert_eq!(household.charitable_vehicles[0].name, "Test Vehicle");
        assert_eq!(household.charitable_vehicles[0].vehicle_type, CharitableVehicleType::DonorAdvisedFund);
        assert_eq!(household.charitable_vehicles[0].balance, 10000.0);
        assert_eq!(household.charitable_vehicles[0].id, vehicle_id);
    }

    #[test]
    #[should_panic(expected = "Vehicle balance cannot be negative")]
    fn test_add_charitable_vehicle_negative_balance() {
        let mut household = SimplifiedHousehold::new("Test Household");
        household.add_charitable_vehicle(
            "Test Vehicle",
            CharitableVehicleType::DonorAdvisedFund,
            -100.0
        );
    }

    #[test]
    fn test_record_donation() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        let donation_id = household.record_donation(
            &charity_id,
            1000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            Some("Test donation")
        );
        
        assert_eq!(household.donations.len(), 1);
        assert_eq!(household.donations[0].charity_id, charity_id);
        assert_eq!(household.donations[0].amount, 1000.0);
        assert_eq!(household.donations[0].id, donation_id);
        
        // Test with vehicle
        let vehicle_id = household.add_charitable_vehicle(
            "Test Vehicle",
            CharitableVehicleType::DonorAdvisedFund,
            10000.0
        );
        
        let donation_id2 = household.record_donation(
            &charity_id,
            2000.0,
            DonationType::Securities,
            NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
            Some(&vehicle_id),
            Some("AAPL"),
            2023,
            true,
            Some("Test donation with vehicle")
        );
        
        assert_eq!(household.donations.len(), 2);
        assert_eq!(household.donations[1].charity_id, charity_id);
        assert_eq!(household.donations[1].amount, 2000.0);
        assert_eq!(household.donations[1].vehicle_id, Some(vehicle_id));
        assert_eq!(household.donations[1].security_id, Some("AAPL".to_string()));
        assert_eq!(household.donations[1].id, donation_id2);
    }

    #[test]
    #[should_panic(expected = "Donation amount must be positive")]
    fn test_record_donation_zero_amount() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        household.record_donation(
            &charity_id,
            0.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            Some("Test donation")
        );
    }

    #[test]
    #[should_panic(expected = "Charity with ID invalid-id does not exist")]
    fn test_record_donation_invalid_charity() {
        let mut household = SimplifiedHousehold::new("Test Household");
        
        household.record_donation(
            "invalid-id",
            1000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            Some("Test donation")
        );
    }

    #[test]
    fn test_analyze_tax_impact() {
        let mut household = SimplifiedHousehold::new("Test Household");
        household.tax_bracket = 0.30;
        household.state_tax_rate = 0.05;
        
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        household.record_donation(
            &charity_id,
            1000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        let tax_impacts = household.analyze_tax_impact();
        
        assert_eq!(tax_impacts.len(), 1);
        assert_eq!(tax_impacts[0].federal_deduction, 1000.0);
        assert_eq!(tax_impacts[0].state_deduction, 900.0);
        assert_eq!(tax_impacts[0].estimated_tax_savings, 345.0); // 1000 * 0.3 + 900 * 0.05
    }

    #[test]
    fn test_total_donations() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        household.record_donation(
            &charity_id,
            1000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        household.record_donation(
            &charity_id,
            2000.0,
            DonationType::Securities,
            NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        assert_eq!(household.total_donations(), 3000.0);
    }

    #[test]
    fn test_donations_by_charity() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity1_id = household.add_charity(
            "Charity 1",
            Some("12-3456789"),
            "Mission 1",
            "Category 1"
        );
        
        let charity2_id = household.add_charity(
            "Charity 2",
            Some("98-7654321"),
            "Mission 2",
            "Category 2"
        );
        
        household.record_donation(
            &charity1_id,
            1000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        household.record_donation(
            &charity1_id,
            2000.0,
            DonationType::Securities,
            NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        household.record_donation(
            &charity2_id,
            3000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 3, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        let donations_by_charity = household.donations_by_charity();
        
        assert_eq!(donations_by_charity.len(), 2);
        assert_eq!(donations_by_charity.get(&charity1_id), Some(&3000.0));
        assert_eq!(donations_by_charity.get(&charity2_id), Some(&3000.0));
    }

    #[test]
    fn test_generate_donation_strategies() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        // No donations yet
        let strategies = household.generate_donation_strategies();
        assert_eq!(strategies.len(), 2); // Should have 2 default strategies
        
        // Add donations
        household.record_donation(
            &charity_id,
            15000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        let strategies = household.generate_donation_strategies();
        assert_eq!(strategies.len(), 3); // Should have 3 strategies now
        
        // Add more donations to trigger the DAF strategy
        household.record_donation(
            &charity_id,
            15000.0,
            DonationType::Securities,
            NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        let strategies = household.generate_donation_strategies();
        assert_eq!(strategies.len(), 4); // Should have 4 strategies now
    }

    #[test]
    fn test_generate_charitable_giving_plan() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        household.record_donation(
            &charity_id,
            10000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        let plan = household.generate_charitable_giving_plan(20000.0);
        
        assert_eq!(plan.annual_target, 20000.0);
        assert!(plan.tax_impact > 0.0);
        // The number of strategies depends on the donation amount
        // With $10,000 in donations, we should have 2 strategies
        assert!(plan.strategies.len() >= 2);
    }

    #[test]
    #[should_panic(expected = "Annual target must be positive")]
    fn test_generate_charitable_giving_plan_zero_target() {
        let household = SimplifiedHousehold::new("Test Household");
        household.generate_charitable_giving_plan(0.0);
    }

    #[test]
    fn test_generate_charitable_giving_report() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        household.record_donation(
            &charity_id,
            10000.0,
            DonationType::Cash,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        let report = household.generate_charitable_giving_report();
        
        assert_eq!(report.total_donations, 10000.0);
        assert_eq!(report.donations_by_charity.len(), 1);
        assert_eq!(report.donations_by_charity.get(&charity_id), Some(&10000.0));
        assert!(report.tax_impact > 0.0);
        // With $10,000 in donations, we should have at least 2 strategies
        assert!(report.strategies.len() >= 2);
    }

    #[test]
    fn test_get_charity_by_id() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        let charity = household.get_charity_by_id(&charity_id);
        assert!(charity.is_some());
        assert_eq!(charity.unwrap().name, "Test Charity");
        
        let nonexistent_charity = household.get_charity_by_id("nonexistent-id");
        assert!(nonexistent_charity.is_none());
    }

    #[test]
    fn test_get_vehicle_by_id() {
        let mut household = SimplifiedHousehold::new("Test Household");
        let vehicle_id = household.add_charitable_vehicle(
            "Test Vehicle",
            CharitableVehicleType::DonorAdvisedFund,
            10000.0
        );
        
        let vehicle = household.get_vehicle_by_id(&vehicle_id);
        assert!(vehicle.is_some());
        assert_eq!(vehicle.unwrap().name, "Test Vehicle");
        
        let nonexistent_vehicle = household.get_vehicle_by_id("nonexistent-id");
        assert!(nonexistent_vehicle.is_none());
    }
    
    #[test]
    fn test_inkind_donation() {
        let mut household = SimplifiedHousehold::new("Test Household");
        household.tax_bracket = 0.30;
        household.state_tax_rate = 0.05;
        
        let charity_id = household.add_charity(
            "Test Charity",
            Some("12-3456789"),
            "Test Mission",
            "Test Category"
        );
        
        household.record_donation(
            &charity_id,
            1000.0,
            DonationType::InKind,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            None,
            None,
            2023,
            true,
            None
        );
        
        let tax_impacts = household.analyze_tax_impact();
        
        assert_eq!(tax_impacts.len(), 1);
        assert_eq!(tax_impacts[0].federal_deduction, 800.0); // 80% of 1000
        assert_eq!(tax_impacts[0].state_deduction, 720.0); // 90% of 800
        assert_eq!(tax_impacts[0].estimated_tax_savings, 276.0); // 800 * 0.3 + 720 * 0.05
    }
} 