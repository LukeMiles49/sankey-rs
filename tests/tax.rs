use sankey::{Sankey, SankeyStyle};

#[test]
fn tax() {
	let mut sankey = Sankey::new();
	
	let salary = sankey.node(Some(50000.0), Some("Salary".into()), None);
	let bonus = sankey.node(Some(5000.0), Some("Bonus".into()), None);
	let employer = sankey.node(None, Some("Employer".into()), None);
	let government = sankey.node(None, Some("Government".into()), None);
	
	let income = sankey.node(None, Some("Income".into()), None);
	sankey.edge(salary, income, sankey.remaining_output(salary), None, None);
	sankey.edge(bonus, income, sankey.remaining_output(bonus), None, None);
	
	let pension = sankey.node(None, Some("Pension".into()), None);
	sankey.edge(income, pension, sankey.required_output(income) * 0.1, None, None);
	sankey.edge(employer, pension, sankey.current_input(pension), None, None);
	
	let taxable = sankey.node(None, Some("Taxable".into()), None);
	sankey.edge(income, taxable, sankey.remaining_output(income), None, None);
	
	let tax = sankey.node(None, Some("Tax".into()), None);
	sankey.edge(taxable, tax, apply_tax(sankey.required_output(taxable), &TAX_BANDS), None, None);
	
	let national_insurance = sankey.node(Some(apply_tax(sankey.required_output(taxable), &NATIONAL_INSURANCE_BANDS)), Some("National Insurance".into()), None);
	sankey.edge(taxable, national_insurance, apply_tax(sankey.required_output(taxable), &NATIONAL_INSURANCE_CONTRIBUTION_BANDS), None, None);
	sankey.edge(employer, national_insurance, sankey.remaining_input(national_insurance), None, None);
	
	let student_loan = sankey.node(None, Some("Student Loan".into()), None);
	sankey.edge(taxable, student_loan, apply_tax(sankey.required_output(taxable), &STUDENT_LOAN_BANDS), None, None);
	
	let rent = sankey.node(Some(8400.0), Some("Rent".into()), None);
	sankey.edge(taxable, rent, sankey.required_input(rent), None, None);
	
	let bills = sankey.node(Some(1000.0), Some("Bills".into()), None);
	sankey.edge(taxable, bills, sankey.required_input(bills), None, None);
	
	let food = sankey.node(Some(2500.0), Some("Food".into()), None);
	sankey.edge(taxable, food, sankey.required_input(food), None, None);
	
	let travel = sankey.node(Some(5000.0), Some("Travel".into()), None);
	sankey.edge(taxable, travel, sankey.required_input(travel), None, None);
	
	let other = sankey.node(Some(2000.0), Some("Other".into()), None);
	sankey.edge(taxable, other, sankey.required_input(other), None, None);
	
	if sankey.remaining_output(taxable) > 0.0 {
		let lisa = sankey.node(None, Some("LISA".into()), None);
		sankey.edge(taxable, lisa, f64::min(sankey.remaining_output(taxable), 4000.0), None, None);
		sankey.edge(government, lisa, sankey.current_input(lisa) * 0.25, None, None);
	}
	
	if sankey.remaining_output(taxable) > 0.0 {
		let isa = sankey.node(None, Some("ISA".into()), None);
		sankey.edge(taxable, isa, f64::min(sankey.remaining_output(taxable), 16000.0), None, None);
	}
	
	if sankey.remaining_output(taxable) > 0.0 {
		let savings = sankey.node(None, Some("Savings".into()), None);
		sankey.edge(taxable, savings, sankey.remaining_output(taxable), None, None);
	}
	
	let style = SankeyStyle {
		number_format: Some(|x| format!("£{:.2}", x)),
		node_separation: None,
		node_width: None,
	};
	
	let svg = sankey.draw(512.0, 512.0, style);
	
	svg::save("./target/tax.svg", &svg).unwrap();
}

struct Band {
	max: f64,
	rate: f64,
}

const TAX_BANDS: [Band; 6] = [
	Band { max: 12570.0, rate: 0.0 },
	Band { max: 50270.0, rate: 0.2 },
	Band { max: 100000.0, rate: 0.4 },
	Band { max: 125140.0, rate: 0.6 },
	Band { max: 150000.0, rate: 0.4 },
	Band { max: f64::INFINITY, rate: 0.45 },
];

const NATIONAL_INSURANCE_CONTRIBUTION_BANDS: [Band; 3] = [
	Band { max: 12570.0, rate: 0.0 },
	Band { max: 50270.0, rate: 0.1325 },
	Band { max: f64::INFINITY, rate: 0.0325 },
];

const NATIONAL_INSURANCE_BANDS: [Band; 2] = [
	Band { max: 9100.0, rate: 0.0 },
	Band { max: f64::INFINITY, rate: 0.1505 },
];

const STUDENT_LOAN_BANDS: [Band; 2] = [
	Band { max: 27295.0, rate: 0.0 },
	Band { max: f64::INFINITY, rate: 0.09 },
];

fn apply_tax(mut income: f64, bands: &[Band]) -> f64 {
	let mut tax = 0.0;
	for &Band { max, rate } in bands {
		tax += rate * f64::min(income, max);
		income -= max;
		if income <= 0.0 {
			break;
		}
	}
	tax
}