pub enum EnumDriverMailType {
    AccountVerification,
    PasswordReset,
    AccountChangement,
    MonthlyReports
}

impl EnumDriverMailType {
    pub fn as_id(&self) -> i32 {
        match self {
            EnumDriverMailType::AccountVerification => 1,
            EnumDriverMailType::PasswordReset => 2,
            EnumDriverMailType::AccountChangement => 3,
            EnumDriverMailType::MonthlyReports => 4,
        }
    }
}