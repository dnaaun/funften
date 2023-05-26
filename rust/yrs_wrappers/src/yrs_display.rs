use yrs::GetString;
use yrs::ReadTxn;

use crate::yrs_wrapper_error::YrsResult;

pub trait YrsDisplay {
    fn fmt(&self, txn: &impl ReadTxn) -> YrsResult<String>;
}

impl YrsDisplay for yrs::TextRef {
    fn fmt(&self, txn: &impl ReadTxn) -> YrsResult<String> {
        Ok(self.get_string(txn))
    }
}
