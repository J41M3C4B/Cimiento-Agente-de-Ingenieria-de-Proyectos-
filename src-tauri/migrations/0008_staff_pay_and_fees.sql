-- Pay per position and stay fees per group, to total the payroll and what the fees bring in.
-- Never a name: staff stay anonymous lines and the people served stay grouped.

ALTER TABLE staff_group ADD COLUMN monthly_salary_mxn INTEGER CHECK (monthly_salary_mxn IS NULL OR monthly_salary_mxn >= 0);
ALTER TABLE staff_group ADD COLUMN contract TEXT CHECK (contract IS NULL OR contract IN ('permanent','temporary','fees'));
ALTER TABLE staff_group ADD COLUMN start_year INTEGER;
ALTER TABLE staff_group ADD COLUMN notes TEXT;

ALTER TABLE population_group ADD COLUMN paying_count INTEGER CHECK (paying_count IS NULL OR paying_count >= 0);
ALTER TABLE population_group ADD COLUMN monthly_fee_mxn INTEGER CHECK (monthly_fee_mxn IS NULL OR monthly_fee_mxn >= 0);
