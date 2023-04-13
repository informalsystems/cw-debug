use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub message: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    SetMessage { message: String, sudo: bool },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetMessage returns the current message
    #[returns(GetMessageResponse)]
    GetMessage {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetMessageResponse {
    pub message: String,
    pub setter: String,
}
